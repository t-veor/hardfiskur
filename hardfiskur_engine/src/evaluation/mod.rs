pub mod packed_score;
pub mod phase;
pub mod piece_tables;

use hardfiskur_core::board::{Board, Color};
use packed_score::PackedScore;
use phase::Phase;
use piece_tables::{material_score, piece_square_table};

use crate::score::Score;

pub fn evaluate_for_white_ex(board: &Board) -> (Score, Phase) {
    let mut phase = Phase(0);
    let mut packed_score = PackedScore::ZERO;

    for (piece, bitboard) in board.repr().boards_colored(Color::White) {
        for square in bitboard.squares() {
            phase.apply_phase(piece);
            packed_score += material_score(piece.piece_type());
            packed_score += piece_square_table(piece.piece_type(), square);
        }
    }

    for (piece, bitboard) in board.repr().boards_colored(Color::Black) {
        for square in bitboard.squares() {
            phase.apply_phase(piece);
            packed_score -= material_score(piece.piece_type());
            packed_score -= piece_square_table(piece.piece_type(), square.flip());
        }
    }

    (Score(phase.taper_packed(packed_score)), phase)
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
