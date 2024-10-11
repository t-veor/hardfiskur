use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use hardfiskur_core::board::{Board, Color, Move};
use hardfiskur_engine::{
    search_limits::SearchLimits,
    search_result::{SearchInfo, SearchResult},
    Engine, SearchReporter,
};

pub struct SearchThread {
    tx: Sender<(Option<Move>, u64)>,
    rx: Receiver<(Option<Move>, u64)>,
    engine: Engine,

    outstanding_request: bool,
    search_gen: u64,
}

struct GUIReporter<F>
where
    F: Fn() + Send + Sync + 'static,
{
    tx: Sender<(Option<Move>, u64)>,
    generation: u64,
    to_move: Color,
    waker: F,
}

impl<F: Fn() + Send + Sync + 'static> SearchReporter for GUIReporter<F> {
    fn receive_search_info(&self, _info: SearchInfo) {}

    fn search_complete(&self, result: SearchResult) {
        let score = match self.to_move {
            Color::White => result.info.score,
            Color::Black => -result.info.score,
        };

        println!(
            "score {score} depth {} seldepth {} nodes {} time {:?} tt_hits {}",
            result.info.raw_stats.depth,
            result.info.raw_stats.sel_depth,
            result.info.raw_stats.nodes_searched,
            result.info.elapsed,
            result.info.raw_stats.tt_hits
        );

        self.tx.send((result.best_move, self.generation)).unwrap();
        (self.waker)();
    }
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

    pub fn send_search_request(&mut self, board: &Board, waker: impl Fn() + Send + Sync + 'static) {
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
            GUIReporter {
                tx,
                generation: search_gen,
                to_move,
                waker,
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
