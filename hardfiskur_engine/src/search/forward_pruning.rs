use crate::{
    parameters::{RFP_DEPTH, RFP_MARGIN},
    score::Score,
};

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub fn forward_pruning(
        &mut self,
        depth: i16,
        _ply_from_root: u16,
        static_eval: Score,
        _alpha: Score,
        beta: Score,
    ) -> Option<Score> {
        // Reverse Futility Pruning
        if depth < RFP_DEPTH && static_eval - RFP_MARGIN * depth as i32 > beta {
            return Some(static_eval);
        }

        None
    }
}
