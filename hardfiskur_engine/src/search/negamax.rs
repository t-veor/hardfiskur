use crate::score::Score;

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

        // Increment stats
        self.stats.nodes_searched += 1;
        self.stats.sel_depth = self.stats.sel_depth.max(ply_from_root);

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

        self.move_orderer
            .order_moves(self.board, ply_from_root, None, &mut legal_moves);

        let mut best_score = -Score::INF;

        for m in legal_moves {
            self.board.push_move_unchecked(m);

            let eval = -self.negamax::<false>(depth - 1, ply_from_root + 1, -beta, -alpha);

            self.board.pop_move();

            // Out of time, stop searching!
            if depth > 1 && self.should_exit_search() {
                return best_score;
            }

            if eval > best_score {
                best_score = eval;

                if eval >= beta {
                    // Beta cutoff!
                    self.stats.beta_cutoffs += 1;
                    break;
                }

                if eval > alpha {
                    alpha = eval;

                    if ROOT {
                        self.best_root_move = Some(m);
                    }
                }
            }
        }

        best_score
    }
}
