#[derive(Debug, Default, Clone)]
pub struct SearchStats {
    pub depth: u32,
    pub nodes_searched: u64,
    pub quiescence_nodes: u64,
    pub beta_cutoffs: u32,
    pub tt_hits: u64,
    pub move_ordering: MoveOrderingStats,
}

#[derive(Debug, Default, Clone)]
pub struct MoveOrderingStats {
    // Last element is all the remaining cases.
    pub pv_node_best_move_idxs: [u32; 8],
    pub beta_cutoff_move_idxs: [u32; 8],
}

impl MoveOrderingStats {
    pub fn record_best_move(&mut self, move_idx: usize) {
        let idx = move_idx.min(self.pv_node_best_move_idxs.len() - 1);
        self.pv_node_best_move_idxs[idx] += 1;
    }

    pub fn record_beta_cutoff(&mut self, move_idx: usize) {
        let idx = move_idx.min(self.beta_cutoff_move_idxs.len() - 1);
        self.beta_cutoff_move_idxs[idx] += 1;
    }
}
