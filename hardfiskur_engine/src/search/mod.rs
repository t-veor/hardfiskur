mod extensions;
mod forward_pruning;
mod negamax;
mod node_types;
mod quiescence;

use std::sync::atomic::AtomicBool;

use hardfiskur_core::board::{Board, Move};
use node_types::Root;

use crate::{
    effort_table::EffortTable,
    history_table::HistoryTable,
    move_ordering::KillerTable,
    parameters::MAX_DEPTH,
    score::Score,
    search_limits::SearchLimits,
    search_result::{SearchInfo, SearchResult},
    search_stats::SearchStats,
    time_manager::TimeManager,
    transposition_table::TranspositionTable,
};

pub struct SearchContext<'a> {
    pub board: &'a mut Board,
    pub stats: SearchStats,

    pub time_manager: TimeManager<'a>,
    pub search_cancelled: bool,

    pub tt: &'a mut TranspositionTable,
    pub history: &'a mut HistoryTable,
    pub killers: KillerTable,
    pub effort: EffortTable,

    pub best_root_move: Option<Move>,
}

impl<'a> SearchContext<'a> {
    pub fn new(
        board: &'a mut Board,
        search_limits: SearchLimits,
        tt: &'a mut TranspositionTable,
        history: &'a mut HistoryTable,
        abort_flag: &'a AtomicBool,
    ) -> Self {
        Self {
            board,
            stats: SearchStats::default(),

            time_manager: TimeManager::new(search_limits, abort_flag),
            search_cancelled: false,

            tt,
            history,
            killers: KillerTable::default(),
            effort: EffortTable::default(),

            best_root_move: None,
        }
    }

    pub fn consistency_check(&self) {
        self.board.consistency_check();
    }

    pub fn should_exit_search(&mut self) -> bool {
        if self.search_cancelled {
            return true;
        }

        self.search_cancelled |= self
            .time_manager
            .check_hard_bound(self.stats.nodes_searched);
        self.search_cancelled
    }

    pub fn check_soft_bound(&mut self, depth: i16) -> bool {
        if self.search_cancelled {
            return true;
        }

        self.search_cancelled |= self
            .time_manager
            .check_soft_bound(depth, self.stats.nodes_searched);
        self.search_cancelled
    }

    pub fn get_search_info(&mut self, score: Score) -> SearchInfo {
        SearchInfo {
            score,
            raw_stats: self.stats.clone(),
            elapsed: self.time_manager.start_time().elapsed(),
            pv: self.tt.extract_pv(self.board),
            hash_full: self.tt.occupancy(),
        }
    }

    pub fn iterative_deepening_search(
        mut self,
        send_search_info: impl Fn(SearchInfo),
    ) -> SearchResult {
        let mut best_score = Score(0);
        let mut best_move = None;

        for depth in 1..=MAX_DEPTH {
            let score = self.negamax::<Root>(depth, 0, -Score::INF, Score::INF);

            // Accept the found best move, even from a partial search.
            if let Some(m) = self.best_root_move.take() {
                best_move = Some(m);

                // Already found a mate, don't need to look any further --
                // although, don't trust mate scores that are greater than the
                // current depth, as they may be from the TT or extensions
                if let Some(signed_plies) = best_score.as_mate_in_plies() {
                    if signed_plies.abs() <= depth as i32 {
                        break;
                    }
                }
            }

            self.stats.depth = depth as _;

            // Update soft bound parameters on the time manager
            self.time_manager.on_iteration_end(
                depth,
                match best_move {
                    Some(m) => self.effort.get_effort(m, self.stats.nodes_searched),
                    None => 0.0,
                },
            );

            // Must search to at least depth 1.
            if depth > 1 && self.check_soft_bound(depth) {
                break;
            }

            best_score = score;

            send_search_info(self.get_search_info(best_score));
        }

        // In the rare case that the engine doesn't return a move, just play the
        // first one in this position
        if best_move.is_none() {
            eprintln!("Search did not return root best move, engine is probably going to blunder!");
            best_move = self.board.legal_moves().first().copied();
        }

        SearchResult {
            best_move,
            info: self.get_search_info(best_score),
        }
    }

    pub fn update_beta_cutoff_heuristics(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        best_move: Move,
        failed_quiets: &[Move],
    ) {
        if !best_move.is_capture() {
            self.killers.store(ply_from_root, best_move);
            self.history
                .update_quiets(self.board.to_move(), depth, best_move, failed_quiets);
        }
    }
}
