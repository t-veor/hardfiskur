use hardfiskur_core::board::Move;
use hardfiskur_core::move_gen::MoveVec;

use crate::{
    evaluation::evaluate,
    move_ordering::MovePicker,
    parameters::{IIR_MIN_DEPTH, LMR_BASE, LMR_DIVISOR, LMR_MIN_DEPTH, LMR_MIN_MOVES_PLAYED},
    score::Score,
    search::forward_pruning::MovePruning,
    transposition_table::{TranspositionEntry, TranspositionFlag},
};

use super::{
    node_types::{NodeType, NonPV},
    SearchContext,
};

impl<'a> SearchContext<'a> {
    pub fn negamax<NT: NodeType>(
        &mut self,
        mut depth: i16,
        ply_from_root: u16,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.consistency_check();
        debug_assert!(NT::IS_PV || beta - alpha == Score(1));

        // Repetition & 50-move-rule handling
        if self
            .board
            .current_position_repeated_at_least(if ply_from_root >= 2 { 1 } else { 2 })
            || self.board.halfmove_clock() >= 100
        {
            return Score(0);
        }

        let (legal_moves, move_gen_result) = self.board.legal_moves_and_meta();

        // Handle checkmate/stalemate
        let in_check = move_gen_result.checker_count > 0;
        if legal_moves.is_empty() {
            return if in_check {
                // Checkmate
                -Score::mate_in_plies(ply_from_root)
            } else {
                // Stalemate
                Score(0)
            };
        }

        if depth <= 0 {
            return self.quiescence(ply_from_root, alpha, beta);
        }

        // Increment stats (after quiescence search, so we don't count the same
        // node twice)
        self.stats.nodes_searched += 1;
        self.stats.sel_depth = self.stats.sel_depth.max(ply_from_root);

        // Transposition table lookup
        let tt_entry = if let Some(entry) = self.tt.get(self.board.zobrist_hash()) {
            // TODO: If this is a beta cutoff, it needs to do killer/history
            // updates etc.
            if !NT::IS_PV && Self::should_cutoff(&entry, depth, ply_from_root, alpha, beta) {
                self.stats.tt_hits += 1;

                // Sanity check
                assert!(!NT::IS_ROOT);

                return entry.get_score(ply_from_root);
            }

            Some(entry)
        } else {
            None
        };

        // Internal Iterative Reductions
        if depth >= IIR_MIN_DEPTH
            && tt_entry
                .as_ref()
                .map_or(true, |entry| entry.depth + 4 <= depth)
        {
            depth -= 1;
        }

        let static_eval = match tt_entry.as_ref() {
            None if in_check => -Score::INF,
            Some(entry) => entry.get_score(ply_from_root),
            None => evaluate(self.board),
        };

        // Forward pruning
        if !NT::IS_ROOT && !NT::IS_PV && !in_check {
            if let Some(score) =
                self.forward_pruning(depth, ply_from_root, static_eval, alpha, beta)
            {
                return score;
            }
        }

        let mut ordered_moves =
            MovePicker::new(legal_moves, tt_entry.and_then(|entry| entry.best_move));

        let mut best_score = -Score::INF;
        let mut best_move = None;
        let original_alpha = alpha;
        let mut previously_played_quiets = MoveVec::new();

        let mut moves_played = 0;
        'move_loop: while let Some(m) =
            ordered_moves.next_move(self.board, ply_from_root, &self.killers, self.history)
        {
            // Move forward pruning. Don't perform if we're in the root, not
            // played any moves yet, or possibly losing to a mating attack
            if !NT::IS_ROOT && moves_played > 0 && !best_score.is_mate_for_them() {
                match self.move_forward_pruning::<NT>(
                    m,
                    depth,
                    in_check,
                    static_eval,
                    alpha,
                    previously_played_quiets.len(),
                ) {
                    MovePruning::None => (),
                    MovePruning::SkipMove => continue 'move_loop,
                    MovePruning::Stop => break 'move_loop,
                }
            }

            let prev_total_nodes = self.stats.nodes_searched;

            let next_hash = self.board.zobrist_hash_after(Some(m));
            self.tt.prefetch(next_hash);

            self.board.push_move_unchecked(m);
            moves_played += 1;

            debug_assert_eq!(self.board.zobrist_hash(), next_hash);

            let eval = if moves_played == 1 {
                -self.negamax::<NT::Next>(depth - 1, ply_from_root + 1, -beta, -alpha)
            } else {
                let reduction =
                    self.calculate_late_move_reduction(m, depth, moves_played, in_check);
                self.principal_variation_search::<NT>(depth, ply_from_root, reduction, alpha, beta)
            };

            self.board.pop_move();

            if NT::IS_ROOT {
                let subtree_nodes = self.stats.nodes_searched - prev_total_nodes;
                self.effort.log_effort(m, subtree_nodes);
            }

            // Out of time, stop searching!
            if depth > 1 && self.should_exit_search() {
                return best_score;
            }

            best_score = best_score.max(eval);

            if eval > alpha {
                alpha = eval;
                best_move = Some(m);

                if NT::IS_ROOT {
                    self.best_root_move = Some(m);
                }

                if eval >= beta {
                    // Beta cutoff!
                    break;
                }
            }

            if !m.is_capture() {
                previously_played_quiets.push(m);
            }
        }

        let tt_flag = Self::determine_tt_flag(best_score, original_alpha, beta);
        if tt_flag == TranspositionFlag::Lowerbound {
            self.stats.beta_cutoffs += 1;

            // Getting a beta-cutoff should always mean we have a best move
            if let Some(best_move) = best_move {
                self.update_beta_cutoff_heuristics(
                    depth,
                    ply_from_root,
                    best_move,
                    &previously_played_quiets,
                );
            } else {
                #[cfg(debug_assertions)]
                panic!("tt_flag was lowerbound but best_move is None?");
            }
        }

        self.tt.set(
            self.board.zobrist_hash(),
            TranspositionEntry::new(tt_flag, depth, best_score, best_move, ply_from_root),
        );

        best_score
    }

    fn principal_variation_search<NT: NodeType>(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        reduction: i16,
        alpha: Score,
        beta: Score,
    ) -> Score {
        // Try a null-window search with late move reudction
        let mut score =
            -self.negamax::<NonPV>(depth - 1 - reduction, ply_from_root + 1, -alpha - 1, -alpha);

        // If the search fails (and there was a reduction), re-search with a
        // null window but with full depth
        if alpha < score && reduction > 0 {
            score = -self.negamax::<NonPV>(depth - 1, ply_from_root + 1, -alpha - 1, -alpha);
        }

        // If the search fails again, we have to do a full width search
        if NT::IS_PV && alpha < score && score < beta {
            // Note -- the null window search fails if the score is >= alpha.
            // However, we can skip the research if it also happens that the
            // score is >= beta, because we would cause a cutoff in the outer
            // loop anyway.
            score = -self.negamax::<NT::Next>(depth - 1, ply_from_root + 1, -beta, -alpha)
        }

        score
    }

    fn determine_tt_flag(
        best_score: Score,
        original_alpha: Score,
        beta: Score,
    ) -> TranspositionFlag {
        if best_score <= original_alpha {
            TranspositionFlag::Upperbound
        } else if best_score >= beta {
            TranspositionFlag::Lowerbound
        } else {
            TranspositionFlag::Exact
        }
    }

    fn should_cutoff(
        entry: &TranspositionEntry,
        depth: i16,
        ply_from_root: u16,
        alpha: Score,
        beta: Score,
    ) -> bool {
        match entry.flag {
            _ if depth > entry.depth => false,
            TranspositionFlag::Exact => true,
            TranspositionFlag::Lowerbound => entry.get_score(ply_from_root) >= beta,
            TranspositionFlag::Upperbound => entry.get_score(ply_from_root) <= alpha,
        }
    }

    fn calculate_late_move_reduction(
        &self,
        m: Move,
        depth: i16,
        moves: usize,
        in_check: bool,
    ) -> i16 {
        if m.is_capture() || moves < LMR_MIN_MOVES_PLAYED || depth < LMR_MIN_DEPTH {
            return 0;
        }

        let mut reduction = LMR_BASE + (depth as f64).ln() * (moves as f64).ln() / LMR_DIVISOR;

        if in_check {
            reduction -= 1.0;
        }

        (reduction as i16).clamp(0, depth)
    }
}
