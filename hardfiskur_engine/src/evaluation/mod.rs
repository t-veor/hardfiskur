pub mod lookups;
pub mod packed_score;
pub mod parameters;
pub mod pawn_structure;
pub mod phase;
pub mod template_params;
pub mod terms;
pub mod trace;

use hardfiskur_core::{
    board::{Bitboard, Board, Color},
    move_gen::lookups::Lookups,
};
use packed_score::PackedScore;
use pawn_structure::PawnStructure;
use phase::Phase;
use template_params::{Bishop, Black, Knight, Queen, Rook, White};
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
    lookups: &'static Lookups,

    occupied: Bitboard,

    pawns: PawnStructure,
}

impl<'a> EvalContext<'a> {
    pub fn new(board: &'a Board) -> Self {
        let occupied = board.get_occupied_bitboard();

        let pawns = PawnStructure::new(board);

        Self {
            board,
            lookups: Lookups::get_instance(),

            occupied,

            pawns,
        }
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

        // Mobility
        score += self.mobility::<White, Knight>(trace);
        score += self.mobility::<White, Bishop>(trace);
        score += self.mobility::<White, Rook>(trace);
        score += self.mobility::<White, Queen>(trace);

        score += self.mobility::<Black, Knight>(trace);
        score += self.mobility::<Black, Bishop>(trace);
        score += self.mobility::<Black, Rook>(trace);
        score += self.mobility::<Black, Queen>(trace);

        // Passed pawns
        score += self.passed_pawns::<White>(trace);
        score += self.passed_pawns::<Black>(trace);

        (Score(phase.taper_packed(score)), phase)
    }
}
