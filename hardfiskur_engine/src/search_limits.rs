use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub time_controls: TimeControls,
    pub node_budget: u64,
    pub depth: i16,
}

impl SearchLimits {
    pub fn infinite() -> Self {
        Self {
            time_controls: TimeControls::Infinite,
            node_budget: u64::MAX,
            depth: i16::MAX,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimeControls {
    Infinite,
    FixedMoveTime(Duration),
    FischerTime {
        remaining: Duration,
        increment: Duration,
    },
    Cyclic {
        remaining: Duration,
        increment: Duration,
        moves_to_go: u32,
    },
}
