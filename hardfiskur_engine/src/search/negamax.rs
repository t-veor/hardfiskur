use hardfiskur_core::board::UCIMove;

use crate::{
    score::Score,
    transposition_table::{TranspositionEntry, TranspositionFlag},
};

use super::SearchContext;

macro_rules! diag_outer {
    ($indent:expr, $($t:tt)*) => {
        if false {
            for _ in 0..$indent {
                eprint!("  ");
            }
            eprintln!($($t)*);
        }
    };
}

impl<'a> SearchContext<'a> {
    pub fn negamax<const ROOT: bool>(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        mut alpha: Score,
        mut beta: Score,
        extension_count: i16,
    ) -> Score {
        macro_rules! diag_header {
            ($($t:tt)*) => {
                diag_outer!(ply_from_root, $($t)*)
            };
        }

        macro_rules! diag {
            ($($t:tt)*) => {
                diag_outer!(ply_from_root + 1, $($t)*)
            };
        }

        diag_header!(
            "> negamax depth={depth} alpha={alpha} beta={beta} ({:?})",
            self.board.zobrist_hash()
        );

        self.board.consistency_check();

        self.stats.nodes_searched += 1;
        self.stats.sel_depth = self.stats.sel_depth.max(ply_from_root);

        // handle repetitions & fifty-move rule
        // this needs to go before the tt lookup, as otherwise entries in the table
        // may confuse it into thinking a repetition has a non-drawn score.
        if self
            .board
            .current_position_repeated_at_least(if ply_from_root >= 2 { 1 } else { 2 })
            || self.board.halfmove_clock() >= 100
        {
            diag!("=> {} (Draw by repetition or 50-move)", Score(0));
            return Score(0);
        }

        let mut tt_move = None;
        if let Some(entry) = self.tt.get(self.board.zobrist_hash()) {
            tt_move = entry.best_move;
            diag!(
                "TT hit: depth={} score={} flag={:?} best_move={}",
                entry.depth,
                entry.get_score(ply_from_root),
                entry.flag,
                if let Some(m) = entry.best_move {
                    format!("{}", UCIMove::from(m))
                } else {
                    "none".to_string()
                }
            );

            if entry.depth >= depth {
                self.stats.tt_hits += 1;

                let score = entry.get_score(ply_from_root);
                match entry.flag {
                    TranspositionFlag::Exact => {
                        if ROOT {
                            self.best_root_move = entry.best_move;
                        }
                        diag!("=> {score} (Exact score entry in TT)");
                        return score;
                    }
                    TranspositionFlag::Upperbound => {
                        beta = beta.min(score);
                        diag!("beta := {beta} (upper bound)");
                    }
                    TranspositionFlag::Lowerbound => {
                        alpha = alpha.max(score);
                        diag!("alpha := {alpha} (lower bound)");
                    }
                }

                // Caused a cutoff? Return immediately
                if alpha >= beta {
                    self.stats.beta_cutoffs += 1;

                    if ROOT {
                        self.best_root_move = entry.best_move;
                    }
                    diag!("=> {score} (TT entry caused cutoff)");

                    return score;
                }
            }
        }
        let mut tt_flag = TranspositionFlag::Upperbound;

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

        // TODO: Try not transitioning into the quiescence search if in check
        if depth <= 0 {
            let score = self.quiescence(ply_from_root, alpha, beta);
            diag!("=> {score} (quiescence search)");
            return score;
        }

        self.move_orderer
            .order_moves(self.board, ply_from_root, tt_move, &mut legal_moves);

        let mut best_score = -Score::INF;
        let mut best_move = None;
        let mut best_move_idx = None;

        for (move_idx, m) in legal_moves.into_iter().enumerate() {
            diag!("play {}", UCIMove::from(m));

            self.board.push_move_unchecked(m);

            let extension = Self::extensions(in_check, extension_count);
            if extension > 0 {
                diag!("apply extension={extension}");
            }

            let eval = -self.negamax::<false>(
                depth - 1 + extension,
                ply_from_root + 1,
                -beta,
                -alpha,
                extension_count + extension,
            );

            self.board.pop_move();

            // Out of time, stop searching!
            if depth > 1 && self.should_exit_search() {
                diag!("=> {best_score} (Out of time!)");
                return best_score;
            }

            diag!("move {} returned {eval}", UCIMove::from(m));

            if eval > best_score {
                diag!("- raised best_score (prev_best_score={best_score})");
                best_score = eval;
                best_move = Some(m);
                best_move_idx = Some(move_idx);

                if ROOT {
                    self.best_root_move = Some(m);
                }
            }

            if eval >= beta {
                tt_flag = TranspositionFlag::Lowerbound;
                self.stats.beta_cutoffs += 1;
                self.stats.move_ordering.record_beta_cutoff(move_idx);

                // Update killer moves
                if !m.is_capture() {
                    self.move_orderer.store_killer(ply_from_root, m);
                }

                diag!("- caused beta cutoff! (beta={beta})");

                break;
            }

            if eval > alpha {
                diag!("- raised alpha (prev_alpha={alpha})");

                tt_flag = TranspositionFlag::Exact;
                alpha = eval;
            }
        }

        self.tt.set(
            self.board.zobrist_hash(),
            TranspositionEntry::new(tt_flag, depth, best_score, best_move, ply_from_root),
        );

        if let Some(i) = best_move_idx {
            self.stats.move_ordering.record_best_move(i);
        }

        match tt_flag {
            TranspositionFlag::Exact => {
                diag!("=> {best_score} (PV-NODE)");
            }
            TranspositionFlag::Lowerbound => {
                diag!("=> {best_score} (FAIL-HIGH)");
            }
            TranspositionFlag::Upperbound => {
                diag!("=> {best_score} (FAIL-LOW)");
            }
        }

        best_score
    }
}
