use hardfiskur_core::{
    board::{Bitboard, Board, Piece},
    move_gen,
};

use super::lookups::PASSED_PAWN_MASKS;

#[derive(Debug, Clone)]
pub struct PawnStructure {
    pub pawns: [Bitboard; 2],
    pub pawn_attacks: [Bitboard; 2],
    pub passed_pawns: [Bitboard; 2],
    pub semi_open_files: [Bitboard; 2],
}

impl PawnStructure {
    pub fn new(board: &Board) -> Self {
        let white_pawns = board.repr()[Piece::WHITE_PAWN];
        let black_pawns = board.repr()[Piece::BLACK_PAWN];

        let white_pawn_attacks = move_gen::white_pawn_attacks(white_pawns);
        let black_pawn_attacks = move_gen::white_pawn_attacks(black_pawns);

        let white_passed_pawns = white_pawns
            .squares()
            .filter(|sq| (PASSED_PAWN_MASKS[0][sq.index()] & black_pawns).is_empty())
            .collect();

        let black_passed_pawns = black_pawns
            .squares()
            .filter(|sq| (PASSED_PAWN_MASKS[1][sq.index()] & white_pawns).is_empty())
            .collect();

        let white_semi_open_files = (0..8)
            .map(Bitboard::file_mask)
            .filter(|&file| (file & white_pawns).is_empty())
            .fold(Bitboard::EMPTY, |acc, bb| acc | bb);
        let black_semi_open_files = (0..8)
            .map(Bitboard::file_mask)
            .filter(|&file| (file & black_pawns).is_empty())
            .fold(Bitboard::EMPTY, |acc, bb| acc | bb);

        Self {
            pawns: [white_pawns, black_pawns],
            pawn_attacks: [white_pawn_attacks, black_pawn_attacks],
            passed_pawns: [white_passed_pawns, black_passed_pawns],
            semi_open_files: [white_semi_open_files, black_semi_open_files],
        }
    }
}
