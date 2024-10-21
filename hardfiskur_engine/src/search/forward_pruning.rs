use hardfiskur_core::board::Move;

use crate::{
    parameters::{
        FP_MARGIN, FP_MARGIN_BASE, FP_MAX_DEPTH, LMP_MARGIN, LMP_MAX_DEPTH, NMP_MIN_DEPTH,
        NMP_REDUCTION, RFP_MARGIN, RFP_MAX_DEPTH,
    },
    score::Score,
};

use super::{
    node_types::{NodeType, NonPV},
    SearchContext,
};

/// Enum used for move pruning within the move loop. Represents whether the move
/// be searched, skipped, or if we should stop trying moves for this node
/// entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MovePruning {
    /// Continue with searching this move.
    None,

    /// Skip this move, but continue searching later moves.
    SkipMove,

    /// Stop searching moves entirely for this node.
    Stop,
}

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

    pub fn move_forward_pruning<NT: NodeType>(
        &self,
        m: Move,
        depth: i16,
        in_check: bool,
        static_eval: Score,
        alpha: Score,
        quiets_played: usize,
    ) -> MovePruning {
        // Futility Pruning. If the score is far enough below alpha, later moves
        // in the move ordering are unlikely to recover a score in very few
        // moves.
        if !NT::IS_PV
            && !in_check
            && !m.is_capture()
            && depth <= FP_MAX_DEPTH
            && static_eval + FP_MARGIN * (depth as i32) + FP_MARGIN_BASE < alpha
        {
            return MovePruning::Stop;
        }

        // Late Move Pruning. Stop searching further moves after trying enough
        // quiet moves without a cutoff.
        if !m.is_capture()
            && depth <= LMP_MAX_DEPTH
            && quiets_played as i32 > LMP_MARGIN + (depth as i32).pow(2) / 2
        {
            return MovePruning::Stop;
        }

        MovePruning::None
    }
}
