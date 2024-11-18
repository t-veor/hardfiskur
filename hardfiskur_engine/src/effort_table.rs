use std::collections::HashMap;

use hardfiskur_core::board::Move;

#[derive(Debug, Clone, Default)]
pub struct EffortTable {
    effort: HashMap<Move, u64>,
}

impl EffortTable {
    pub fn log_effort(&mut self, m: Move, subtree_nodes: u64) {
        let nodes = self.effort.entry(m).or_default();
        *nodes += subtree_nodes;
    }

    pub fn get_effort(&self, m: Move, total_nodes: u64) -> f64 {
        let nodes = self.effort.get(&m).copied().unwrap_or(0);

        let effort = nodes as f64 / total_nodes as f64;
        if effort.is_nan() {
            0.0
        } else {
            effort.clamp(0.0, 1.0)
        }
    }
}
