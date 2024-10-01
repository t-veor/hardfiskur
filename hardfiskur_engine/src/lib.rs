use std::{
    sync::{
        atomic::{AtomicBool, Ordering as AtomicOrdering},
        Arc, Mutex,
    },
    time::Duration,
};

use hardfiskur_core::board::Board;
use search::{iterative_deepening_search, SearchContext};
use search_result::SearchResult;
use transposition_table::TranspositionTable;

pub mod evaluation;
pub mod move_ordering;
pub mod score;
pub mod search;
pub mod search_result;
pub mod search_stats;
pub mod transposition_table;

pub struct Engine {
    curr_abort_flag: Arc<AtomicBool>,
    transposition_table: Arc<Mutex<TranspositionTable>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            curr_abort_flag: Arc::new(AtomicBool::new(false)),
            transposition_table: Arc::new(Mutex::new(TranspositionTable::new(64))),
        }
    }

    pub fn start_search(
        &mut self,
        board: &Board,
        allocated_time: Duration,
        callback: impl FnOnce(SearchResult) + Send + 'static,
    ) {
        let mut board = board.clone();

        self.curr_abort_flag = Arc::new(AtomicBool::new(false));
        let abort_flag = self.curr_abort_flag.clone();

        let transposition_table = self.transposition_table.clone();

        std::thread::spawn(move || {
            let mut tt = transposition_table.lock().unwrap();
            let ctx = SearchContext::new(&mut board, allocated_time, &mut tt, &abort_flag);

            callback(iterative_deepening_search(ctx));
        });
    }

    pub fn abort_search(&self) {
        self.curr_abort_flag.store(true, AtomicOrdering::Relaxed);
    }

    pub fn new_game(&self) {
        self.abort_search();
        let mut tt = self.transposition_table.lock().unwrap();
        tt.clear();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.curr_abort_flag.store(true, AtomicOrdering::Relaxed);
    }
}
