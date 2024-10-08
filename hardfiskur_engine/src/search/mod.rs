mod extensions;
mod negamax;
mod quiescence;

use std::{
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::Instant,
};

use hardfiskur_core::board::{Board, Move, UCIMove};

use crate::{
    move_ordering::MoveOrderer, parameters::MAX_DEPTH, score::Score, search_limits::SearchLimits,
    search_result::SearchResult, search_stats::SearchStats,
    transposition_table::TranspositionTable,
};

pub struct SearchContext<'a> {
    pub board: &'a mut Board,
    pub search_limits: SearchLimits,
    pub start_time: Instant,
    pub stats: SearchStats,
    pub time_up: bool,

    pub tt: &'a mut TranspositionTable,
    pub move_orderer: MoveOrderer,

    pub abort_flag: &'a AtomicBool,

    pub best_root_move: Option<Move>,
}

impl<'a> SearchContext<'a> {
    pub fn new(
        board: &'a mut Board,
        search_limits: SearchLimits,
        tt: &'a mut TranspositionTable,
        abort_flag: &'a AtomicBool,
    ) -> Self {
        Self {
            board,
            search_limits,
            start_time: Instant::now(),
            stats: SearchStats::default(),
            time_up: false,
            tt,
            move_orderer: MoveOrderer::new(),
            abort_flag,
            best_root_move: None,
        }
    }

    pub fn should_exit_search(&mut self) -> bool {
        self.is_time_up() || self.over_node_budget()
    }

    pub fn is_time_up(&mut self) -> bool {
        if self.time_up {
            return true;
        }

        // Avoid syscalls a bit
        if self.stats.nodes_searched % 2048 != 0 {
            return false;
        }

        self.time_up = self.start_time.elapsed() >= self.search_limits.allocated_time
            || self.abort_flag.load(AtomicOrdering::Relaxed);

        self.time_up
    }

    pub fn over_node_budget(&self) -> bool {
        self.stats.nodes_searched >= self.search_limits.node_budget
    }

    pub fn iterative_deepening_search(mut self) -> SearchResult {
        let mut best_score = Score(0);
        let mut best_move = None;

        for depth in 1..=(self.search_limits.depth.min(MAX_DEPTH)) {
            let score = self.negamax::<true>(depth, 0, -Score::INF, Score::INF, 0);

            if let Some(m) = self.best_root_move.take() {
                // If we did not even get a root move from a partial search then
                // we can't accept its results.
                best_score = score;
                best_move = Some(m);

                // Already found a mate, don't need to look any further -- although,
                // don't trust mate scores that are greater than the current depth,
                // as they may be from the TT or extensions
                if let Some(signed_plies) = best_score.as_mate_in_plies() {
                    if signed_plies.abs() <= depth as i32 {
                        break;
                    }
                }
            }

            self.stats.depth = depth as _;

            // TODO: Do this properly, e.g. by providing a listener to feed this
            // information to
            if depth > 1 && self.stats.nodes_searched > 4096 {
                let pv = self.tt.extract_pv(self.board);
                print!("info depth {depth} nodes {} ", self.stats.nodes_searched);
                if let Some(mate) = score.as_mate_in() {
                    print!("score mate {mate} ");
                } else if let Some(cp) = score.as_centipawns() {
                    print!("score cp {cp} ")
                }
                print!("hashfull {} ", self.tt.occupancy());
                print!("pv ",);
                for m in pv {
                    print!("{} ", UCIMove::from(m));
                }
                println!();

                let pv_nodes = self
                    .stats
                    .move_ordering
                    .pv_node_best_move_idxs
                    .iter()
                    .sum::<u64>();
                let cut_nodes = self
                    .stats
                    .move_ordering
                    .beta_cutoff_move_idxs
                    .iter()
                    .sum::<u64>();

                println!(
                    "info string pv_node_proportions {:?}",
                    self.stats
                        .move_ordering
                        .pv_node_best_move_idxs
                        .iter()
                        .map(|&x| x * 1000 / pv_nodes.max(1))
                        .collect::<Vec<_>>()
                );
                println!(
                    "info string beta_node_proportions {:?}",
                    self.stats
                        .move_ordering
                        .beta_cutoff_move_idxs
                        .iter()
                        .map(|&x| x * 1000 / cut_nodes.max(1))
                        .collect::<Vec<_>>()
                );
            }

            if self.should_exit_search() {
                break;
            }
        }

        SearchResult {
            score: best_score,
            best_move,
            stats: self.stats,
            elapsed: self.start_time.elapsed(),
            aborted: self.abort_flag.load(AtomicOrdering::Relaxed),
            hash_full: self.tt.occupancy(),
        }
    }
}
