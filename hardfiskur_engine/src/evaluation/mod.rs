pub mod packed_score;
pub mod parameters;
pub mod phase;
pub mod template_params;
pub mod terms;
pub mod trace;

use hardfiskur_core::board::{Board, Color};
use packed_score::PackedScore;
use phase::Phase;
use template_params::{Black, White};
use trace::{NullTrace, Trace};

use crate::score::Score;

pub fn evaluate_for_white_ex(board: &Board) -> (Score, Phase) {
    let eval_context = EvalContext::new(board);
    eval_context.evaluate_ex(&mut NullTrace)
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

pub struct EvalContext<'a> {
    board: &'a Board,
}

impl<'a> EvalContext<'a> {
    pub fn new(board: &'a Board) -> Self {
        Self { board }
    }

    pub fn evaluate_ex(&self, trace: &mut impl Trace) -> (Score, Phase) {
        let mut phase = Phase(0);
        let mut score = PackedScore::ZERO;

        for (piece, bitboard) in self.board.repr().boards_colored(Color::White) {
            for square in bitboard.squares() {
                phase.apply_phase(piece);
                score += self.material::<White>(piece.piece_type(), trace);
                score += self.piece_square_table::<White>(piece.piece_type(), square, trace);
            }
        }

        for (piece, bitboard) in self.board.repr().boards_colored(Color::Black) {
            for square in bitboard.squares() {
                phase.apply_phase(piece);
                score += self.material::<Black>(piece.piece_type(), trace);
                score += self.piece_square_table::<Black>(piece.piece_type(), square, trace);
            }
        }

        (Score(phase.taper_packed(score)), phase)
    }
}
