use hardfiskur_core::board::{PieceType, Square};

use crate::evaluation::parameters::PIECE_SQUARE_TABLES;

use super::{packed_score::S, parameters::MATERIAL, trace::Trace, EvalContext};

trait SignExt {
    fn sign(self) -> i32;
}

impl SignExt for bool {
    fn sign(self) -> i32 {
        match self {
            true => 1,
            false => -1,
        }
    }
}

impl<'a> EvalContext<'a> {
    pub fn material<const IS_WHITE: bool>(
        &self,
        piece_type: PieceType,
        trace: &mut impl Trace,
    ) -> S {
        trace.add(|t| t.material[piece_type.index()] += IS_WHITE.sign());

        IS_WHITE.sign() * MATERIAL[piece_type.index()]
    }

    pub fn piece_square_table<const IS_WHITE: bool>(
        &self,
        piece_type: PieceType,
        square: Square,
        trace: &mut impl Trace,
    ) -> S {
        let square = if IS_WHITE { square } else { square.flip() };

        trace.add(|t| {
            let table = &mut match piece_type {
                PieceType::Pawn => t.pawn_pst,
                PieceType::Knight => t.knight_pst,
                PieceType::Bishop => t.bishop_pst,
                PieceType::Rook => t.rook_pst,
                PieceType::Queen => t.queen_pst,
                PieceType::King => t.king_pst,
            };
            table[square.index()] += IS_WHITE.sign();
        });

        IS_WHITE.sign() * PIECE_SQUARE_TABLES[piece_type.index()][square.index()]
    }
}
