use hardfiskur_core::move_gen::{MoveGenFlags, MoveVec};

use crate::{evaluation::evaluate, score::Score};

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub fn quiescence(&mut self, ply_from_root: u16, mut alpha: Score, beta: Score) -> Score {
        self.stats.nodes_searched += 1;
        self.stats.quiescence_nodes += 1;

        let stand_pat_score = evaluate(self.board);

        if stand_pat_score >= beta {
            self.stats.beta_cutoffs += 1;
            return stand_pat_score;
        }

        let mut best_score = stand_pat_score;
        alpha = alpha.max(stand_pat_score);

        let mut capturing_moves = MoveVec::new();
        self.board
            .legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut capturing_moves);

        self.move_orderer
            .order_moves(self.board, ply_from_root, None, &mut capturing_moves);

        let mut best_move_idx = None;

        for (move_idx, m) in capturing_moves.into_iter().enumerate() {
            self.board.push_move_unchecked(m);
            let eval = -self.quiescence(ply_from_root + 1, -beta, -alpha);
            self.board.pop_move();

            if eval > best_score {
                best_score = eval;
                best_move_idx = Some(move_idx);

                if eval >= beta {
                    self.stats.beta_cutoffs += 1;
                    self.stats.move_ordering.record_beta_cutoff(move_idx);

                    // Update killer moves
                    self.move_orderer.store_killer(ply_from_root, m);
                    break;
                }

                if eval > alpha {
                    alpha = eval;
                }
            }
        }

        if let Some(i) = best_move_idx {
            self.stats.move_ordering.record_best_move(i);
        }

        return best_score;
    }
}
