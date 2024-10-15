mod killer_table;
mod see;

use hardfiskur_core::{
    board::{Board, Color, Move, Piece},
    move_gen::MoveVec,
};
use killer_table::KillerTable;

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

impl MoveOrderer {
    const HASH_MOVE_SCORE: i32 = 100_000_000;
    const WINNING_CAPTURE_BIAS: i32 = 8_000_000;
    const KILLER_BIAS: i32 = 4_000_000;
    const QUIET_BIAS: i32 = 0;
    // const LOSING_CAPTURE_BIAS: i32 = -2_000_000;

    pub fn order_moves(
        &self,
        board: &Board,
        ply_from_root: u16,
        tt_move: Option<Move>,
        history: &HistoryTable,
        moves: MoveVec,
    ) -> OrderedMoves {
        // let seer = Seer::new(board);

        let scores = moves
            .iter()
            .map(|m| self.score_move(board.to_move(), ply_from_root, tt_move, history, *m))
            .collect();
        OrderedMoves { moves, scores }
    }

    pub fn score_move(
        &self,
        to_move: Color,
        ply_from_root: u16,
        tt_move: Option<Move>,
        history: &HistoryTable,
        m: Move,
    ) -> i32 {
        if Some(m) == tt_move {
            Self::HASH_MOVE_SCORE
        } else if let Some(victim) = m.captured_piece() {
            let aggressor = m.piece();
            // Order by MVV-LVA
            Self::WINNING_CAPTURE_BIAS + self.mvv_lva_score(victim, aggressor)
        } else if self.is_killer(ply_from_root, m) {
            Self::KILLER_BIAS
        } else {
            Self::QUIET_BIAS + history.get_quiet_history(to_move, m)
        }
    }

    fn mvv_lva_score(&self, victim: Piece, aggressor: Piece) -> i32 {
        // Most valuable victim (* 10 to make sure it's always considered above
        // the aggressor)
        (victim.piece_type() as i32) * 10
        // Least valuable aggressor
        - (aggressor.piece_type() as i32)
    }
}

pub struct OrderedMoves {
    moves: MoveVec,
    scores: Vec<i32>,
}

impl Iterator for OrderedMoves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.scores.is_empty() {
            return None;
        }

        let mut max = self.scores[0];
        let mut max_idx = 0;

        for i in 1..self.scores.len() {
            if self.scores[i] > max {
                max = self.scores[i];
                max_idx = i;
            }
        }

        self.scores.swap_remove(max_idx);
        Some(self.moves.swap_remove(max_idx))
    }
}
