pub mod piece_tables;

use hardfiskur_core::board::{Board, Color, Move, PieceType};
use piece_tables::{material_score, phase_modifier, piece_square_table, FULL_ENDGAME_PHASE};

use crate::score::Score;

pub fn evaluate(board: &Board) -> Score {
    let mut material = 0;
    let mut middlegame_pst = 0;
    let mut endgame_pst = 0;

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
        middlegame_pst += mid * sign;
        endgame_pst += end * sign;

        game_phase += phase_modifier(piece.piece_type());
    }

    let tapered_eval = (middlegame_pst * game_phase
        + endgame_pst * (FULL_ENDGAME_PHASE - game_phase))
        / FULL_ENDGAME_PHASE;

    Score(material + tapered_eval)
}
