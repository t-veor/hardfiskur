use std::sync::{
    atomic::{AtomicBool, Ordering as AtomicOrdering},
    Arc, Mutex,
};

use evaluation::evaluate_for_white;
use hardfiskur_core::board::{Board, Move};
use score::Score;
use search::SearchContext;
use search_limits::SearchLimits;
use search_result::{SearchInfo, SearchResult};
use transposition_table::{TranspositionEntry, TranspositionTable};

pub mod bench;
pub mod evaluation;
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
    transposition_table: Arc<Mutex<TranspositionTable>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            curr_abort_flag: Arc::new(AtomicBool::new(false)),
            transposition_table: Arc::new(Mutex::new(TranspositionTable::new(
                64.try_into().unwrap(),
            ))),
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

        let transposition_table = self.transposition_table.clone();

        std::thread::spawn(move || {
            let mut tt = transposition_table.lock().unwrap();
            let ctx = SearchContext::new(&mut board, search_limits, &mut tt, &abort_flag);

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
        let mut tt = self.transposition_table.lock().unwrap();
        tt.clear();
    }

    pub fn get_tt_entry(&self, current_board: &Board) -> Option<TranspositionEntry> {
        let tt = self.transposition_table.lock().unwrap();
        tt.get(current_board.zobrist_hash())
    }

    pub fn get_pv(&self, current_board: &Board) -> Vec<Move> {
        let tt = self.transposition_table.lock().unwrap();
        tt.extract_pv(&mut current_board.clone())
    }

    pub fn debug_eval(&self, current_board: &Board) -> Score {
        evaluate_for_white(current_board)
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

pub trait SearchReporter: Send + Sync + 'static {
    fn receive_search_info(&self, info: SearchInfo);
    fn search_complete(&self, result: SearchResult);
}

pub struct NullReporter;

impl SearchReporter for NullReporter {
    fn receive_search_info(&self, _info: SearchInfo) {}
    fn search_complete(&self, _result: SearchResult) {}
}
