use hardfiskur_core::board::{Board, PieceType, Square};
use zerocopy::FromZeros;

use crate::evaluation::parameters::PIECE_SQUARE_TABLES;

use super::{
    packed_score::S,
    parameters::MATERIAL,
    trace::{EvalTrace, Trace},
    EvalContext,
};

trait BoolColorExt {
    fn coeff(self) -> i16;
    fn sign(self) -> i32;
}

impl BoolColorExt for bool {
    fn coeff(self) -> i16 {
        match self {
            true => 1,
            false => -1,
        }
    }

    fn sign(self) -> i32 {
        match self {
            true => 1,
            false => -1,
        }
    }
}

impl<'a> EvalContext<'a> {
    #[inline]
    pub fn material<const IS_WHITE: bool>(
        &self,
        piece_type: PieceType,
        trace: &mut impl Trace,
    ) -> S {
        trace.add(|t| t.material[piece_type.index()] += IS_WHITE.coeff());

        IS_WHITE.sign() * MATERIAL[piece_type.index()]
    }

    #[inline]
    pub fn piece_square_table<const IS_WHITE: bool>(
        &self,
        piece_type: PieceType,
        square: Square,
        trace: &mut impl Trace,
    ) -> S {
        let square = if IS_WHITE { square.flip() } else { square };

        trace.add(|t| {
            let table = match piece_type {
                PieceType::Pawn => &mut t.pawn_pst,
                PieceType::Knight => &mut t.knight_pst,
                PieceType::Bishop => &mut t.bishop_pst,
                PieceType::Rook => &mut t.rook_pst,
                PieceType::Queen => &mut t.queen_pst,
                PieceType::King => &mut t.king_pst,
            };

            table[square.index()] += IS_WHITE.coeff();
        });

        IS_WHITE.sign() * PIECE_SQUARE_TABLES[piece_type.index()][square.index()]
    }
}
