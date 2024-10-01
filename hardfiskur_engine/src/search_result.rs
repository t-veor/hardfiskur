use std::time::Duration;

use hardfiskur_core::board::Move;

use crate::{score::Score, search_stats::SearchStats};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: Score,
    pub best_move: Option<Move>,
    pub stats: SearchStats,
    pub elapsed: Duration,
    pub aborted: bool,
}
