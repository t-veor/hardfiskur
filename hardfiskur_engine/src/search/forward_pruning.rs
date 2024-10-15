use crate::{
    parameters::{NMP_MIN_DEPTH, NMP_REDUCTION, RFP_MARGIN, RFP_MAX_DEPTH},
    score::Score,
};

use super::{node_types::NonPV, SearchContext};

impl<'a> SearchContext<'a> {
    pub fn forward_pruning(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        static_eval: Score,
        _alpha: Score,
        beta: Score,
    ) -> Option<Score> {
        // Reverse Futility Pruning
        if depth <= RFP_MAX_DEPTH && (static_eval - RFP_MARGIN * depth as i32) > beta {
            return Some(static_eval);
        }

        // Null Move Pruning
        if let Some(score) = self.null_move_pruning(depth, ply_from_root, beta, static_eval) {
            return Some(score);
        }

        None
    }

    fn null_move_pruning(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        beta: Score,
        static_eval: Score,
    ) -> Option<Score> {
        if depth >= NMP_MIN_DEPTH
            && static_eval > beta
            && self.board.last_move().is_some()
            && !self.board.is_king_and_pawn_endgame()
        {
            self.board.push_null_move();

            let score =
                -self.negamax::<NonPV>(depth - NMP_REDUCTION, ply_from_root + 1, -beta, -beta + 1);

            self.board.pop_move();

            return if score.is_mate_for_us() {
                Some(beta)
            } else if score >= beta {
                Some(score)
            } else {
                None
            };
        }

        None
    }
}
