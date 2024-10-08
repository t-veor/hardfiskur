pub mod piece_tables;

use hardfiskur_core::board::{Board, Color};
use piece_tables::{material_score, phase_modifier, piece_square_table, FULL_ENDGAME_PHASE};

use crate::score::Score;

pub fn evaluate_for_white(board: &Board) -> Score {
    let mut material = 0;
    let mut middlegame_eval = 0;
    let mut endgame_eval = 0;

    let mut game_phase = 0;

    for (piece, mut square) in board.pieces() {
        let sign = match piece.color() {
            Color::White => 1,
            Color::Black => -1,
        };

        // Flip square if black, as piece square tables are from white's
        // perspective
        if piece.color().is_black() {
            square = square.flip()
        }

        material += material_score(piece.piece_type()) * sign;

        // Piece square table values
        let (mid, end) = piece_square_table(piece.piece_type(), square);
        middlegame_eval += mid * sign;
        endgame_eval += end * sign;

        game_phase += phase_modifier(piece.piece_type());
    }

    let tapered_eval = (middlegame_eval * game_phase
        + endgame_eval * (FULL_ENDGAME_PHASE - game_phase))
        / FULL_ENDGAME_PHASE;

    Score(material + tapered_eval)
}

pub fn evaluate(board: &Board) -> Score {
    let score = evaluate_for_white(board);
    match board.to_move() {
        Color::White => score,
        Color::Black => -score,
    }
}
