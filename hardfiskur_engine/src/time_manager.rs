use std::{
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::{Duration, Instant},
};

use hardfiskur_core::board::Move;

use crate::search_limits::{SearchLimits, TimeControls};

pub const MOVE_OVERHEAD: Duration = Duration::from_millis(15);

pub const SOFT_MULTIPLIER: f64 = 1.0 / 30.0;
pub const HARD_MULTIPLIER: f64 = 1.0 / 5.0;
pub const INCREMENT_MULTIPLIER: f64 = 0.75;

pub const CYCLIC_SOFT_MULTIPLIER: f64 = 0.8;
pub const CYCLIC_HARD_MULTIPLIER: f64 = 4.0;

pub const SOFT_BOUND_ADJUSTMENT_MIN_DEPTH: i16 = 10;

pub const NODE_ADJUSTMENT_BIAS: f64 = 2.0;
pub const NODE_ADJUSTMENT_WEIGHT: f64 = -1.5;

// Adapted from Stash
pub const MOVE_STABILITY_ADJUSTMENT: [f64; 5] = [2.50, 1.20, 0.90, 0.80, 0.75];

#[derive(Debug, Clone)]
pub struct TimeManager<'a> {
    start_time: Instant,
    soft_bound: Duration,
    hard_bound: Duration,

    max_depth: i16,
    max_nodes: u64,

    last_best_move: Option<Move>,

    move_stability: usize,
    best_move_effort: f64,

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

            last_best_move: None,
            move_stability: 0,
            best_move_effort: 1.0,

            abort_flag,
        }
    }

    pub fn on_iteration_end(&mut self, depth: i16, best_move: Option<Move>, best_move_effort: f64) {
        // Results from first few iterations are not very stable
        if depth < SOFT_BOUND_ADJUSTMENT_MIN_DEPTH {
            return;
        }

        if self.last_best_move == best_move {
            self.move_stability += 1;
        } else {
            self.last_best_move = best_move;
            self.move_stability = 0;
        }

        self.best_move_effort = best_move_effort;
    }

    fn node_adjustment(&self) -> f64 {
        NODE_ADJUSTMENT_BIAS + NODE_ADJUSTMENT_WEIGHT * self.best_move_effort
    }

    fn move_stability_adjustment(&self) -> f64 {
        *MOVE_STABILITY_ADJUSTMENT
            .get(self.move_stability)
            .or(MOVE_STABILITY_ADJUSTMENT.last())
            .unwrap()
    }

    pub fn check_soft_bound(&self, depth: i16, nodes: u64) -> bool {
        if depth >= self.max_depth || nodes >= self.max_nodes {
            return true;
        }

        let soft_bound = if depth < SOFT_BOUND_ADJUSTMENT_MIN_DEPTH {
            self.soft_bound
        } else {
            // Adjust the soft bound based on several parameters.
            let mut soft_bound = self.soft_bound.as_secs_f64();

            soft_bound *= self.node_adjustment();
            soft_bound *= self.move_stability_adjustment();

            Duration::try_from_secs_f64(soft_bound).unwrap_or(Duration::MAX)
        };

        self.start_time.elapsed() >= soft_bound
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
