pub mod phase;
pub mod piece_tables;

use hardfiskur_core::board::{Board, Color};
use phase::Phase;
use piece_tables::{material_score, piece_square_table};

use crate::score::Score;

pub fn evaluate_for_white_ex(board: &Board) -> (Score, Phase) {
    let mut phase = Phase(0);

    let mut material = 0;
    let mut midgame_eval = 0;
    let mut endgame_eval = 0;

    for (piece, square) in board.pieces() {
        phase.apply_phase(piece);

        let sign = match piece.color() {
            Color::White => 1,
            Color::Black => -1,
        };

        material += material_score(piece.piece_type());

        {
            let corrected_square = match piece.color() {
                Color::White => square,
                Color::Black => square.flip(),
            };

            let (mg, eg) = piece_square_table(piece.piece_type(), corrected_square);

            midgame_eval += sign * mg;
            endgame_eval += sign * eg;
        }
    }

    let tapered_eval = phase.taper(midgame_eval, endgame_eval);

    (Score(material + tapered_eval), phase)
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
