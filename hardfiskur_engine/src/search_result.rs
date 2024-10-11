use std::time::Duration;

use hardfiskur_core::board::Move;

use crate::{score::Score, search_stats::SearchStats};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub info: SearchInfo,
    pub aborted: bool,
}

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub score: Score,
    pub raw_stats: SearchStats,
    pub elapsed: Duration,
    pub pv: Vec<Move>,
    pub hash_full: u64,
}
