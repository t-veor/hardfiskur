use crate::{
    parameters::{ASPIRATION_INITIAL_WINDOW, ASPIRATION_MIN_DEPTH},
    score::Score,
};

use super::{node_types::Root, SearchContext};

impl<'a> SearchContext<'a> {
    pub fn aspiration_search(&mut self, prev_score: Score, depth: i16) -> Score {
        // Skip doing the aspiration search when the depth is low, as the score is very unstable at low depths.
        if depth < ASPIRATION_MIN_DEPTH {
            return self.negamax::<Root>(depth, 0, -Score::INF, Score::INF);
        }

        let mut delta = ASPIRATION_INITIAL_WINDOW;
        loop {
            let alpha = Score(prev_score.0.saturating_sub(delta)).max(-Score::INF);
            let beta = Score(prev_score.0.saturating_add(delta)).min(Score::INF);

            let score = self.negamax::<Root>(depth, 0, alpha, beta);

            // Give up if time is up
            if self.should_exit_search() {
                return Score(0);
            }

            if alpha < score && score < beta {
                return score;
            }

            // Window failed, double window size and re-search
            delta = delta.saturating_mul(2);
        }
    }
}
