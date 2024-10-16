mod killer_table;
mod see;

use hardfiskur_core::{
    board::{Board, Color, Move, Piece},
    move_gen::MoveVec,
};
use killer_table::KillerTable;
use see::Seer;

use crate::history_table::HistoryTable;

pub struct MoveOrderer {
    killers: KillerTable,
}

impl MoveOrderer {
    pub fn new() -> Self {
        Self {
            killers: KillerTable::default(),
        }
    }

    pub fn update_heuristics(&mut self, _depth: i16, ply_from_root: u16, best_move: Move) {
        if !best_move.is_capture() {
            self.killers.store(ply_from_root, best_move);
        }
    }

    pub fn is_killer(&self, ply_from_root: u16, m: Move) -> bool {
        self.killers.is_killer(ply_from_root, m)
    }
}

impl Default for MoveOrderer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OrderedMoves {
    moves: MoveVec,
    tt_move: Option<Move>,
    scores: Vec<i32>,
}

impl OrderedMoves {
    pub fn new(moves: MoveVec, tt_move: Option<Move>) -> Self {
        Self {
            moves,
            tt_move,
            scores: Vec::new(),
        }
    }

    const WINNING_CAPTURE_BIAS: i32 = 8_000_000;
    const KILLER_BIAS: i32 = 4_000_000;
    const QUIET_BIAS: i32 = 0;
    const LOSING_CAPTURE_BIAS: i32 = -2_000_000;

    #[inline]
    pub fn next_move(
        &mut self,
        board: &Board,
        ply_from_root: u16,
        history: &HistoryTable,
        move_orderer: &MoveOrderer,
    ) -> Option<Move> {
        if let Some(tt_move) = self.tt_move.take() {
            if let Some(idx) = self.moves.iter().position(|&m| m == tt_move) {
                self.moves.swap_remove(idx);
                return Some(tt_move);
            }
        }

        if self.moves.is_empty() {
            return None;
        }

        if self.scores.is_empty() {
            let seer = Seer::new(board);
            self.scores = self
                .moves
                .iter()
                .map(|&m| {
                    Self::score_move(
                        board.to_move(),
                        ply_from_root,
                        &seer,
                        history,
                        move_orderer,
                        m,
                    )
                })
                .collect();
        }

        let mut max_idx = 0;
        let mut max_score = self.scores[0];

        for i in 1..self.scores.len() {
            if self.scores[i] > max_score {
                max_idx = i;
                max_score = self.scores[i];
            }
        }

        self.scores.swap_remove(max_idx);
        Some(self.moves.swap_remove(max_idx))
    }

    pub fn score_move(
        to_move: Color,
        ply_from_root: u16,
        seer: &Seer,
        history: &HistoryTable,
        move_orderer: &MoveOrderer,
        m: Move,
    ) -> i32 {
        // Playing the TT move first already handled by Self::next_move.
        if let Some(victim) = m.captured_piece() {
            let aggressor = m.piece();

            // Is the capture actually winning?
            let bias = if m.promotion().is_some()
                || seer.see(m.from_square(), aggressor, m.to_square(), victim, 0)
            {
                Self::WINNING_CAPTURE_BIAS
            } else {
                Self::LOSING_CAPTURE_BIAS
            };

            // Then, order by MVV-LVA
            bias + Self::mvv_lva_score(victim, aggressor)
        } else if move_orderer.is_killer(ply_from_root, m) {
            Self::KILLER_BIAS
        } else {
            Self::QUIET_BIAS + history.get_quiet_history(to_move, m)
        }
    }

    fn mvv_lva_score(victim: Piece, aggressor: Piece) -> i32 {
        // Most valuable victim (* 10 to make sure it's always considered above
        // the aggressor)
        (victim.piece_type() as i32) * 10
        // Least valuable aggressor
        - (aggressor.piece_type() as i32)
    }
}
