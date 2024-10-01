use std::{time::Duration, u64};

#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub allocated_time: Duration,
    pub node_budget: u64,
    pub depth: u32,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            allocated_time: Duration::from_millis(500),
            node_budget: u64::MAX,
            depth: u32::MAX,
        }
    }
}
