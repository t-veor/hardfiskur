use crate::score::Score;

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub fn quiescence(&mut self, _ply_from_root: u16, _alpha: Score, _beta: Score) -> Score {
        self.consistency_check();

        return Score(0);
    }
}
