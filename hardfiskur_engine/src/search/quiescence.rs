use hardfiskur_core::move_gen::{MoveGenFlags, MoveVec};

use crate::{evaluation::evaluate, score::Score};

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub fn quiescence(&mut self, ply_from_root: u16, mut alpha: Score, beta: Score) -> Score {
        self.consistency_check();

        // Increment stats
        self.stats.nodes_searched += 1;
        self.stats.quiescence_nodes += 1;

        // Score from standing pat.
        let mut best_score = evaluate(self.board);

        if best_score >= beta {
            // Beta cutoff!
            self.stats.beta_cutoffs += 1;
            return best_score;
        }

        alpha = alpha.max(best_score);

        let capturing_moves = {
            let mut moves = MoveVec::new();
            self.board
                .legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut moves);

            moves
        };

        for m in capturing_moves {
            if !m.is_capture() {
                continue;
            }

            self.board.push_move_unchecked(m);

            let eval = -self.quiescence(ply_from_root + 1, -beta, -alpha);

            self.board.pop_move();

            if eval > best_score {
                best_score = eval;

                if eval >= beta {
                    // Beta cutoff!
                    self.stats.beta_cutoffs += 1;
                    break;
                }
            }

            alpha = alpha.max(eval);
        }

        best_score
    }
}
