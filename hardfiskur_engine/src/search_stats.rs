use std::time::Instant;

#[derive(Debug)]
pub struct SearchStats {
    pub depth: u32,
    pub search_started: Instant,
    pub nodes_searched: u64,
    pub quiescence_nodes: u64,
    pub beta_cutoffs: u32,
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            depth: 0,
            search_started: Instant::now(),
            nodes_searched: 0,
            quiescence_nodes: 0,
            beta_cutoffs: 0,
        }
    }
}
