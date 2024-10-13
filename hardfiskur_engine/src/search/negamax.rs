use crate::{
    score::Score,
    transposition_table::{TranspositionEntry, TranspositionFlag},
};

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub fn negamax<const ROOT: bool>(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.consistency_check();

        // Repetition & 50-move-rule handling
        if self
            .board
            .current_position_repeated_at_least(if ply_from_root >= 2 { 1 } else { 2 })
            || self.board.halfmove_clock() >= 100
        {
            return Score(0);
        }

        let (mut legal_moves, move_gen_result) = self.board.legal_moves_and_meta();

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

        // Transposition table lookup
        let tt_move = if let Some(entry) = self.tt.get(self.board.zobrist_hash()) {
            // TODO: If this is a beta cutoff, it needs to do killer/history
            // updates etc.
            if Self::should_cutoff(&entry, depth, ply_from_root, alpha, beta) {
                self.stats.tt_hits += 1;

                // TODO: This is temporarily here to prevent the search from not
                // returning a move when the root node finds an exact score.
                // Once PVS is implemented, TT cutoffs should no longer happen
                // on PV nodes (including the root), so this will be unnecessary
                // and can be removed at that point.
                if ROOT {
                    self.best_root_move = entry.best_move;
                }

                return entry.get_score(ply_from_root);
            }

            entry.best_move
        } else {
            None
        };

        self.move_orderer
            .order_moves(self.board, ply_from_root, tt_move, &mut legal_moves);

        let mut best_score = -Score::INF;
        let mut best_move = None;
        let original_alpha = alpha;

        for m in legal_moves {
            self.board.push_move_unchecked(m);

            let eval = -self.negamax::<false>(depth - 1, ply_from_root + 1, -beta, -alpha);

            self.board.pop_move();

            // Out of time, stop searching!
            if depth > 1 && self.should_exit_search() {
                return best_score;
            }

            best_score = best_score.max(eval);

            if eval > alpha {
                alpha = eval;
                best_move = Some(m);

                if ROOT {
                    self.best_root_move = Some(m);
                }

                if eval >= beta {
                    // Beta cutoff!
                    break;
                }
            }
        }

        let tt_flag = Self::determine_tt_flag(best_score, original_alpha, beta);
        if tt_flag == TranspositionFlag::Lowerbound {
            self.stats.beta_cutoffs += 1;
        }

        self.tt.set(
            self.board.zobrist_hash(),
            TranspositionEntry::new(tt_flag, depth, best_score, best_move, ply_from_root),
        );

        best_score
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
}
