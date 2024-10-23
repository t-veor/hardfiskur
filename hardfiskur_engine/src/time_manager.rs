use std::{
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::{Duration, Instant},
};

use crate::search_limits::{SearchLimits, TimeControls};

pub const MOVE_OVERHEAD: Duration = Duration::from_millis(15);

pub const SOFT_MULTIPLIER: f64 = 1.0 / 30.0;
pub const HARD_MULTIPLIER: f64 = 1.0 / 5.0;
pub const INCREMENT_MULTIPLIER: f64 = 0.75;

pub const CYCLIC_SOFT_MULTIPLIER: f64 = 0.8;
pub const CYCLIC_HARD_MULTIPLIER: f64 = 4.0;

#[derive(Debug, Clone)]
pub struct TimeManager<'a> {
    start_time: Instant,
    soft_bound: Duration,
    hard_bound: Duration,
    max_depth: i16,
    max_nodes: u64,
    abort_flag: &'a AtomicBool,
}

impl<'a> TimeManager<'a> {
    pub fn new(limits: SearchLimits, abort_flag: &'a AtomicBool) -> Self {
        let (soft_bound, hard_bound) = Self::time_bounds(limits.time_controls);

        Self {
            start_time: Instant::now(),
            soft_bound,
            hard_bound,
            max_depth: limits.depth,
            max_nodes: limits.node_budget,
            abort_flag,
        }
    }

    pub fn check_soft_bound(&self, depth: i16, nodes: u64) -> bool {
        if depth >= self.max_depth || nodes >= self.max_nodes {
            return true;
        }

        self.start_time.elapsed() >= self.soft_bound
    }

    pub fn check_hard_bound(&self, nodes: u64) -> bool {
        if nodes >= self.max_nodes {
            return true;
        }

        // Avoid syscalls a bit
        if nodes % 2048 != 0 {
            return false;
        }

        self.start_time.elapsed() >= self.hard_bound
            || self.abort_flag.load(AtomicOrdering::Relaxed)
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    fn time_bounds(controls: TimeControls) -> (Duration, Duration) {
        let (soft, hard) = match controls {
            TimeControls::FixedMoveTime(duration) => (duration, duration),
            TimeControls::FischerTime {
                remaining,
                increment,
            } => {
                let increment = increment.mul_f64(INCREMENT_MULTIPLIER);

                let soft = (remaining + increment).mul_f64(SOFT_MULTIPLIER) + increment;
                let hard = (remaining + increment).mul_f64(HARD_MULTIPLIER) + increment;

                (soft.min(remaining), hard.min(remaining))
            }
            TimeControls::Cyclic {
                remaining,
                increment,
                moves_to_go,
            } => {
                // Plan to use an even amount of time for each move in
                // moves_to_go
                let move_alloc = remaining / moves_to_go;
                let increment = increment.mul_f64(INCREMENT_MULTIPLIER);

                let soft = (move_alloc + increment).mul_f64(CYCLIC_SOFT_MULTIPLIER);
                let hard = (move_alloc + increment).mul_f64(CYCLIC_HARD_MULTIPLIER);

                (soft.min(remaining), hard.min(remaining))
            }
            _ => return (Duration::MAX, Duration::MAX),
        };

        (
            soft.saturating_sub(MOVE_OVERHEAD),
            hard.saturating_sub(MOVE_OVERHEAD),
        )
    }
}
