pub mod phase;
pub mod piece_tables;

use hardfiskur_core::board::{Board, Color};
use phase::Phase;

use crate::score::Score;

pub fn evaluate_for_white_ex(_board: &Board) -> (Score, Phase) {
    (Score(0), Phase(0))
}

pub fn evaluate_ex(board: &Board) -> (Score, Phase) {
    let (white_score, phase) = evaluate_for_white_ex(board);

    let score = match board.to_move() {
        Color::White => white_score,
        Color::Black => -white_score,
    };

    (score, phase)
}

pub fn evaluate_for_white(board: &Board) -> Score {
    evaluate_for_white_ex(board).0
}

pub fn evaluate(board: &Board) -> Score {
    evaluate_ex(board).0
}
