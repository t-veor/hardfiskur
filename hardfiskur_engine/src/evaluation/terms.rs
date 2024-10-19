use hardfiskur_core::board::{PieceType, Square};

use crate::evaluation::parameters::PIECE_SQUARE_TABLES;

use super::{
    packed_score::S, parameters::MATERIAL, template_params::ColorParam, trace::Trace, EvalContext,
};

impl<'a> EvalContext<'a> {
    #[inline]
    pub fn material<Color: ColorParam>(&self, piece_type: PieceType, trace: &mut impl Trace) -> S {
        trace.add(|t| t.material[piece_type.index()] += Color::COEFF);

        Color::SIGN * MATERIAL[piece_type.index()]
    }

    #[inline]
    pub fn piece_square_table<Color: ColorParam>(
        &self,
        piece_type: PieceType,
        square: Square,
        trace: &mut impl Trace,
    ) -> S {
        let square = if Color::IS_WHITE {
            square.flip()
        } else {
            square
        };

        trace.add(|t| {
            let table = match piece_type {
                PieceType::Pawn => &mut t.pawn_pst,
                PieceType::Knight => &mut t.knight_pst,
                PieceType::Bishop => &mut t.bishop_pst,
                PieceType::Rook => &mut t.rook_pst,
                PieceType::Queen => &mut t.queen_pst,
                PieceType::King => &mut t.king_pst,
            };

            table[square.index()] += Color::COEFF;
        });

        Color::SIGN * PIECE_SQUARE_TABLES[piece_type.index()][square.index()]
    }
}
