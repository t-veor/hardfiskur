use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use eframe::egui::mutex::Mutex;
use hardfiskur_core::board::{Board, Color, Move};
use hardfiskur_engine::{
    search::iterative_deepening_search, transposition_table::TranspositionTable,
};
use threadpool::ThreadPool;

pub struct SearchThread {
    tx: Sender<(Option<Move>, u64)>,
    rx: Receiver<(Option<Move>, u64)>,
    thread_pool: ThreadPool,

    transposition_table: Arc<Mutex<TranspositionTable>>,

    outstanding_request: bool,
    search_gen: u64,
}

impl SearchThread {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let thread_pool = ThreadPool::new(2);

        let transposition_table = Arc::new(Mutex::new(TranspositionTable::new(64)));

        Self {
            tx,
            rx,
            transposition_table,
            thread_pool,
            outstanding_request: false,
            search_gen: 0,
        }
    }

    pub fn searching(&self) -> bool {
        self.outstanding_request
    }

    pub fn send_search_request(&mut self, board: &Board, waker: impl Fn() + Send + 'static) {
        let tx = self.tx.clone();

        let mut board = board.clone();
        self.search_gen += 1;
        let search_gen = self.search_gen;

        let transposition_table = self.transposition_table.clone();

        self.thread_pool.execute(move || {
            let mut transposition_table = transposition_table.lock();
            let (score, search_result, stats) = iterative_deepening_search(
                &mut board,
                Duration::from_millis(1000),
                &mut transposition_table,
            );

            let score = match board.to_move() {
                Color::White => score,
                Color::Black => -score,
            };

            println!(
                "score {score} depth {} nodes {} tt_hits {}",
                stats.depth, stats.nodes_searched, stats.tt_hits
            );

            tx.send((search_result, search_gen)).unwrap();
            waker();
        });

        self.outstanding_request = true;
    }

    pub fn cancel_search(&mut self) {
        self.search_gen += 1;
        self.outstanding_request = false;

        // TODO: actual cancelling
    }

    pub fn try_receive_move(&mut self) -> Option<Move> {
        if let Ok((m, search_gen)) = self.rx.try_recv() {
            if search_gen == self.search_gen {
                self.outstanding_request = false;
                return m;
            }
        }

        None
    }
}
