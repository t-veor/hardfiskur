#[derive(Debug, Default, Clone)]
pub struct SearchStats {
    pub depth: u32,
    pub nodes_searched: u64,
    pub quiescence_nodes: u64,
    pub beta_cutoffs: u32,
    pub tt_hits: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            depth: 0,
            nodes_searched: 0,
            quiescence_nodes: 0,
            beta_cutoffs: 0,
            tt_hits: 0,
        }
    }
}
