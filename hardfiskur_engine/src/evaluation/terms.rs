use hardfiskur_core::board::{Piece, PieceType, Square};

use crate::evaluation::parameters::{
    BISHOP_MOBILITY, KNIGHT_MOBILITY, PIECE_SQUARE_TABLES, QUEEN_MOBILITY, ROOK_MOBILITY,
};

use super::{
    packed_score::S,
    parameters::MATERIAL,
    template_params::{ColorParam, PieceTypeParam},
    trace::Trace,
    EvalContext,
};

impl<'a> EvalContext<'a> {
    #[inline]
    pub fn material<C: ColorParam>(&self, piece_type: PieceType, trace: &mut impl Trace) -> S {
        trace.add(|t| t.material[piece_type.index()] += C::COEFF);

        C::SIGN * MATERIAL[piece_type.index()]
    }

    #[inline]
    pub fn piece_square_table<C: ColorParam>(
        &self,
        piece_type: PieceType,
        square: Square,
        trace: &mut impl Trace,
    ) -> S {
        let square = if C::IS_WHITE { square.flip() } else { square };

        trace.add(|t| {
            let table = match piece_type {
                PieceType::Pawn => &mut t.pawn_pst,
                PieceType::Knight => &mut t.knight_pst,
                PieceType::Bishop => &mut t.bishop_pst,
                PieceType::Rook => &mut t.rook_pst,
                PieceType::Queen => &mut t.queen_pst,
                PieceType::King => &mut t.king_pst,
            };

            table[square.index()] += C::COEFF;
        });

        C::SIGN * PIECE_SQUARE_TABLES[piece_type.index()][square.index()]
    }

    #[inline]
    pub fn mobility<C: ColorParam, P: PieceTypeParam>(&self, trace: &mut impl Trace) -> S {
        const {
            assert!(
                !matches!(P::PIECE_TYPE, PieceType::Pawn | PieceType::King),
                "Can't call mobility() with Pawn or King!"
            );
        }

        let mut total = S::ZERO;

        let mobility_squares = if C::IS_WHITE {
            !self.black_pawn_attacks
        } else {
            !self.white_pawn_attacks
        } & !self.board.get_bitboard_for_color(C::COLOR);

        let piece_bb = self
            .board
            .get_bitboard_for_piece(Piece::new(C::COLOR, P::PIECE_TYPE));

        for square in piece_bb.squares() {
            let mobility_bb = match P::PIECE_TYPE {
                PieceType::Knight => self.lookups.get_knight_moves(square),
                PieceType::Bishop => self.lookups.get_bishop_attacks(self.occupied, square),
                PieceType::Rook => self.lookups.get_rook_attacks(self.occupied, square),
                PieceType::Queen => self.lookups.get_queen_attacks(self.occupied, square),
                PieceType::Pawn | PieceType::King => unreachable!(),
            } & mobility_squares;

            let mobility_count = mobility_bb.pop_count() as usize;

            trace.add(|t| match P::PIECE_TYPE {
                PieceType::Knight => t.knight_mobility[mobility_count] += C::COEFF,
                PieceType::Bishop => t.bishop_mobility[mobility_count] += C::COEFF,
                PieceType::Rook => t.rook_mobility[mobility_count] += C::COEFF,
                PieceType::Queen => t.queen_mobility[mobility_count] += C::COEFF,
                PieceType::Pawn | PieceType::King => unreachable!(),
            });

            total += C::SIGN
                * match P::PIECE_TYPE {
                    PieceType::Knight => KNIGHT_MOBILITY[mobility_count],
                    PieceType::Bishop => BISHOP_MOBILITY[mobility_count],
                    PieceType::Rook => ROOK_MOBILITY[mobility_count],
                    PieceType::Queen => QUEEN_MOBILITY[mobility_count],
                    PieceType::Pawn | PieceType::King => unreachable!(),
                };
        }

        total
    }
}
