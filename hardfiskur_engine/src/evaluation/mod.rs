pub mod lookups;
pub mod packed_score;
pub mod parameters;
pub mod pawn_structure;
pub mod phase;
pub mod template_params;
pub mod terms;
pub mod trace;

use hardfiskur_core::{
    board::{Bitboard, Board, Color, Square},
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
    kings: [Square; 2],
    king_zones: [Bitboard; 2],
}

impl<'a> EvalContext<'a> {
    pub fn new(board: &'a Board) -> Self {
        let lookups = Lookups::get_instance();
        let occupied = board.get_occupied_bitboard();

        let pawns = PawnStructure::new(board);

        let white_king = board.get_king(Color::White);
        let black_king = board.get_king(Color::Black);

        // King zone is defined to be the 3x4 area of the king -- the 3x3 area
        // around the king, plus 3 additional tiles in the forward direction.
        let mut white_king_zone = lookups.get_king_moves(white_king);
        white_king_zone |= white_king_zone.step_north();
        let mut black_king_zone = lookups.get_king_moves(black_king);
        black_king_zone |= black_king_zone.step_south();

        Self {
            board,
            lookups,

            occupied,

            pawns,
            kings: [white_king, black_king],
            king_zones: [white_king_zone, black_king_zone],
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
                score += self.open_file_bonus::<White>(piece.piece_type(), square, trace);
            }
        }

        for (piece, bitboard) in self.board.repr().boards_colored(Color::Black) {
            for square in bitboard.squares() {
                phase.apply_phase(piece);
                score += self.material::<Black>(piece.piece_type(), trace);
                score += self.piece_square_table::<Black>(piece.piece_type(), square, trace);
                score += self.open_file_bonus::<Black>(piece.piece_type(), square, trace);
            }
        }

        // Mobility
        score += self.mobility_and_king_zone_attacks::<White, Knight>(trace);
        score += self.mobility_and_king_zone_attacks::<White, Bishop>(trace);
        score += self.mobility_and_king_zone_attacks::<White, Rook>(trace);
        score += self.mobility_and_king_zone_attacks::<White, Queen>(trace);

        score += self.mobility_and_king_zone_attacks::<Black, Knight>(trace);
        score += self.mobility_and_king_zone_attacks::<Black, Bishop>(trace);
        score += self.mobility_and_king_zone_attacks::<Black, Rook>(trace);
        score += self.mobility_and_king_zone_attacks::<Black, Queen>(trace);

        // Passed pawns
        score += self.passed_pawns::<White>(trace);
        score += self.passed_pawns::<Black>(trace);

        // Doubled pawns
        score += self.doubled_pawns::<White>(trace);
        score += self.doubled_pawns::<Black>(trace);

        // Isolated pawns
        score += self.isolated_pawns::<White>(trace);
        score += self.isolated_pawns::<Black>(trace);

        // Phalanx pawns
        score += self.phalanx_pawns::<White>(trace);
        score += self.phalanx_pawns::<Black>(trace);

        // Protected pawns
        score += self.protected_pawns::<White>(trace);
        score += self.protected_pawns::<Black>(trace);

        // Pawn shield
        score += self.pawn_shield::<White>(trace);
        score += self.pawn_shield::<Black>(trace);

        // Outposts
        score += self.knight_outposts::<White>(trace);
        score += self.knight_outposts::<Black>(trace);
        score += self.bishop_outposts::<White>(trace);
        score += self.bishop_outposts::<Black>(trace);

        (Score(phase.taper_packed(score)), phase)
    }
}
