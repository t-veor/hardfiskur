use hardfiskur_core::{
    board::{Bitboard, Board, Piece},
    move_gen,
};

#[derive(Debug, Clone)]
pub struct PawnStructure {
    pub pawns: [Bitboard; 2],
    pub pawn_attacks: [Bitboard; 2],
    pub passed_pawns: [Bitboard; 2],
}

impl PawnStructure {
    pub fn new(board: &Board) -> Self {
        let white_pawns = board.repr()[Piece::WHITE_PAWN];
        let black_pawns = board.repr()[Piece::BLACK_PAWN];

        let white_pawn_attacks = move_gen::white_pawn_attacks(white_pawns);
        let black_pawn_attacks = move_gen::white_pawn_attacks(black_pawns);

        todo!()
    }
}
