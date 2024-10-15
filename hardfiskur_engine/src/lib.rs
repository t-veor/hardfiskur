use std::sync::{
    atomic::{AtomicBool, Ordering as AtomicOrdering},
    Arc, Mutex,
};

use evaluation::evaluate_for_white;
use hardfiskur_core::board::{Board, Move};
use history_table::HistoryTable;
use score::Score;
use search::SearchContext;
use search_limits::SearchLimits;
use search_result::{SearchInfo, SearchResult};
use transposition_table::{TranspositionEntry, TranspositionTable};

pub mod bench;
pub mod evaluation;
pub mod history_table;
pub mod move_ordering;
pub mod parameters;
pub mod score;
pub mod search;
pub mod search_limits;
pub mod search_result;
pub mod search_stats;
pub mod transposition_table;

pub struct Engine {
    curr_abort_flag: Arc<AtomicBool>,
    persistent: Arc<Mutex<Persistent>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            curr_abort_flag: Arc::new(AtomicBool::new(false)),
            persistent: Arc::new(Mutex::new(Persistent {
                tt: TranspositionTable::new(32.try_into().unwrap()),
                history: HistoryTable::new(),
            })),
        }
    }

    pub fn start_search(
        &mut self,
        board: &Board,
        search_limits: SearchLimits,
        reporter: impl SearchReporter,
    ) {
        let mut board = board.clone();

        self.curr_abort_flag = Arc::new(AtomicBool::new(false));
        let abort_flag = self.curr_abort_flag.clone();

        let persistent = self.persistent.clone();

        std::thread::spawn(move || {
            let persistent = &mut *persistent.lock().unwrap();
            let ctx = SearchContext::new(
                &mut board,
                search_limits,
                &mut persistent.tt,
                &mut persistent.history,
                &abort_flag,
            );

            let result = ctx.iterative_deepening_search(|info| {
                reporter.receive_search_info(info);
            });

            reporter.search_complete(result);
        });
    }

    pub fn abort_search(&self) {
        self.curr_abort_flag.store(true, AtomicOrdering::Relaxed);
    }

    pub fn new_game(&self) {
        self.abort_search();
        let mut persistent = self.persistent.lock().unwrap();
        persistent.tt.clear();
        persistent.history.clear();
    }

    pub fn get_tt_entry(&self, current_board: &Board) -> Option<TranspositionEntry> {
        let persistent = self.persistent.lock().unwrap();
        persistent.tt.get(current_board.zobrist_hash())
    }

    pub fn get_pv(&self, current_board: &Board) -> Vec<Move> {
        let persistent = self.persistent.lock().unwrap();
        persistent.tt.extract_pv(&mut current_board.clone())
    }

    pub fn debug_eval(&self, current_board: &Board) -> Score {
        evaluate_for_white(current_board)
    }

    pub fn set_tt_size(&mut self, size_in_mb: usize) {
        let mut persistent = self.persistent.lock().unwrap();
        persistent.tt.resize(size_in_mb.try_into().unwrap());
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.curr_abort_flag.store(true, AtomicOrdering::Relaxed);
    }
}

struct Persistent {
    tt: TranspositionTable,
    history: HistoryTable,
}

pub trait SearchReporter: Send + Sync + 'static {
    fn receive_search_info(&self, info: SearchInfo);
    fn search_complete(&self, result: SearchResult);
}

pub struct NullReporter;

impl SearchReporter for NullReporter {
    fn receive_search_info(&self, _info: SearchInfo) {}
    fn search_complete(&self, _result: SearchResult) {}
}
