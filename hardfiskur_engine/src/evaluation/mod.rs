mod mobility;
pub mod piece_tables;

use hardfiskur_core::{
    board::{Bitboard, Board, Color, Piece},
    move_gen::{self, lookups::Lookups},
};
use piece_tables::{material_score, phase_modifier, piece_square_table, FULL_ENDGAME_PHASE};

use crate::score::Score;

pub struct EvalContext<'a> {
    board: &'a Board,
    lookups: &'static Lookups,
    occupied: Bitboard,

    white_pawn_attacks: Bitboard,
    black_pawn_attacks: Bitboard,
}

impl<'a> EvalContext<'a> {
    pub fn new(board: &'a Board, lookups: &'static Lookups) -> Self {
        Self {
            board,
            lookups,
            occupied: board.get_occupied_bitboard(),

            white_pawn_attacks: move_gen::white_pawn_attacks(
                board.get_bitboard_for_piece(Piece::WHITE_PAWN),
            ),
            black_pawn_attacks: move_gen::black_pawn_attacks(
                board.get_bitboard_for_piece(Piece::BLACK_PAWN),
            ),
        }
    }

    pub fn evaluate(&self) -> Score {
        let mut material = 0;
        let mut mobility = 0;
        let mut middlegame_pst = 0;
        let mut endgame_pst = 0;

        let mut game_phase = 0;

        for (piece, mut square) in self.board.pieces() {
            let sign = match piece.color() {
                Color::White => 1,
                Color::Black => -1,
            };

            material += material_score(piece.piece_type()) * sign;

            mobility += self.mobility_score(piece, square) * sign;

            // For PSTs, flip square if black, as they are from white's
            // perspective
            if piece.color().is_black() {
                square = square.flip()
            }

            // Piece square table values
            let (mid, end) = piece_square_table(piece.piece_type(), square);
            middlegame_pst += mid * sign;
            endgame_pst += end * sign;

            game_phase += phase_modifier(piece.piece_type());
        }

        let tapered_eval = (middlegame_pst * game_phase
            + endgame_pst * (FULL_ENDGAME_PHASE - game_phase))
            / FULL_ENDGAME_PHASE;

        Score(material + tapered_eval + mobility)
    }
}
