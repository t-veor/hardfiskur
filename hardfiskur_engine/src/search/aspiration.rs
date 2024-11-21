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
        let mut alpha = prev_score.saturating_sub(delta).max(-Score::INF);
        let mut beta = prev_score.saturating_add(delta).min(Score::INF);
        let mut reduction = 0;

        loop {
            let score = self.negamax::<Root>((depth - reduction).max(1), 0, alpha, beta);

            // Give up if time is up
            if self.should_exit_search() {
                return Score(0);
            }

            if score <= alpha {
                // Fail-low, grow the window downwards.
                alpha = alpha.saturating_sub(delta).max(-Score::INF);
                beta = alpha.midpoint(beta);
                reduction = 0;
            } else if score >= beta {
                // Fail-high, grow the window upwards
                beta = beta.saturating_add(delta).min(Score::INF);
                reduction += 1;
            } else {
                // Window passed
                return score;
            }

            // double window size and re-search
            delta = delta.saturating_mul(2);
        }
    }
}
