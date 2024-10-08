use std::{time::Duration, u64};

#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub allocated_time: Duration,
    pub node_budget: u64,
    pub depth: i16,
}

impl SearchLimits {
    pub fn infinite() -> Self {
        Self {
            allocated_time: Duration::MAX,
            node_budget: u64::MAX,
            depth: i16::MAX,
        }
    }
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            allocated_time: Duration::from_millis(1000),
            node_budget: u64::MAX,
            depth: i16::MAX,
        }
    }
}
