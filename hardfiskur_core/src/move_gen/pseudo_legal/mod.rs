use crate::board::Bitboard;

use super::{MoveGenMasks, MoveGenerator};

mod pawn_moves;

impl<'board, 'moves> MoveGenerator<'board, 'moves> {
    fn pseudo_legal_moves(&mut self, masks: &MoveGenMasks) {
        self.pseudo_legal_pawn_moves(masks);

        unimplemented!()
    }
}
