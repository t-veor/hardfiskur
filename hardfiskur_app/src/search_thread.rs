use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use hardfiskur_core::board::{Board, Color, Move};
use hardfiskur_engine::{search_limits::SearchLimits, Engine};

pub struct SearchThread {
    tx: Sender<(Option<Move>, u64)>,
    rx: Receiver<(Option<Move>, u64)>,
    engine: Engine,

    outstanding_request: bool,
    search_gen: u64,
}

impl SearchThread {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            tx,
            rx,
            engine: Engine::new(),

            outstanding_request: false,
            search_gen: 0,
        }
    }

    pub fn searching(&self) -> bool {
        self.outstanding_request
    }

    pub fn send_search_request(&mut self, board: &Board, waker: impl Fn() + Send + 'static) {
        let tx = self.tx.clone();

        self.search_gen += 1;
        let search_gen = self.search_gen;

        let to_move = board.to_move();

        self.engine.start_search(
            board,
            SearchLimits {
                allocated_time: Duration::from_millis(500),
                ..Default::default()
            },
            move |result| {
                let score = match to_move {
                    Color::White => result.score,
                    Color::Black => -result.score,
                };

                println!(
                    "score {score} depth {} nodes {} time {:?} tt_hits {}",
                    result.stats.depth,
                    result.stats.nodes_searched,
                    result.elapsed,
                    result.stats.tt_hits
                );

                tx.send((result.best_move, search_gen)).unwrap();
                waker();
            },
        );

        self.outstanding_request = true;
    }

    pub fn cancel_search(&mut self) {
        self.search_gen += 1;
        self.outstanding_request = false;
        self.engine.abort_search();
    }

    pub fn reset(&self) {
        self.engine.new_game();
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
