use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use hardfiskur_core::board::{Board, Color, Move};
use hardfiskur_engine::search::iterative_deepening_search;
use threadpool::ThreadPool;

pub struct SearchThread {
    tx: Sender<(Option<Move>, u64)>,
    rx: Receiver<(Option<Move>, u64)>,
    thread_pool: ThreadPool,

    outstanding_request: bool,
    search_gen: u64,
}

impl SearchThread {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let thread_pool = ThreadPool::new(2);

        Self {
            tx,
            rx,
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

        self.thread_pool.execute(move || {
            let (score, search_result, stats) =
                iterative_deepening_search(&mut board, Duration::from_millis(200));

            let score = match board.to_move() {
                Color::White => score,
                Color::Black => -score,
            };

            println!(
                "score {score} depth {} nodes {}",
                stats.depth, stats.nodes_searched
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
