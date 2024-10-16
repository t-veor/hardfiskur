mod killer_table;
mod see;

use hardfiskur_core::{
    board::{Board, Color, Move, Piece},
    move_gen::MoveVec,
};

pub use killer_table::KillerTable;
pub use see::Seer;

use crate::history_table::HistoryTable;

pub struct MovePicker {
    moves: MoveVec,
    tt_move: Option<Move>,
    scores: Vec<i32>,
}

impl MovePicker {
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

    pub fn next_move(
        &mut self,
        board: &Board,
        ply_from_root: u16,
        killers: &KillerTable,
        history: &HistoryTable,
    ) -> Option<Move> {
        if let Some(tt_move) = self.tt_move.take() {
            if let Some(idx) = self.moves.iter().position(|&m| m == tt_move) {
                return Some(self.moves.swap_remove(idx));
            }
        }

        if self.moves.is_empty() {
            return None;
        }

        if self.scores.is_empty() {
            self.fill_scores(board, ply_from_root, killers, history);
        }

        Some(self.next_highest_move())
    }

    fn fill_scores(
        &mut self,
        board: &Board,
        ply_from_root: u16,
        killers: &KillerTable,
        history: &HistoryTable,
    ) {
        let seer = Seer::new(board);
        self.scores = vec![0; self.moves.len()];
        for (i, &m) in self.moves.iter().enumerate() {
            self.scores[i] =
                Self::score_move(board.to_move(), ply_from_root, &seer, killers, history, m);
        }
    }

    fn next_highest_move(&mut self) -> Move {
        // Assumes non-empty scores and moves
        let mut max_idx = 0;
        let mut max_score = self.scores[0];

        for i in 1..self.scores.len() {
            if self.scores[i] > max_score {
                max_idx = i;
                max_score = self.scores[i];
            }
        }

        self.scores.swap_remove(max_idx);
        self.moves.swap_remove(max_idx)
    }

    pub fn score_move(
        to_move: Color,
        ply_from_root: u16,
        seer: &Seer,
        killers: &KillerTable,
        history: &HistoryTable,
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
        } else if killers.is_killer(ply_from_root, m) {
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
