//! Structs and functions related to to the board representation.

mod bitboard;
mod board_repr;
mod fen;
mod move_repr;
mod piece;
mod square;

use bitflags::bitflags;

pub use bitboard::Bitboard;
pub use board_repr::BoardRepr;
pub use move_repr::{Move, MoveFlags};
pub use piece::{Color, Piece, PieceType};
pub use square::Square;

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Castling: u8 {
        const WHITE_KINGSIDE  = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE  = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
        const WHITE = Self::WHITE_KINGSIDE.bits() | Self::WHITE_QUEENSIDE.bits();
        const BLACK = Self::BLACK_KINGSIDE.bits() | Self::BLACK_QUEENSIDE.bits();
        const KINGSIDE = Self::WHITE_KINGSIDE.bits() | Self::BLACK_KINGSIDE.bits();
        const QUEENSIDE = Self::WHITE_QUEENSIDE.bits() | Self::BLACK_QUEENSIDE.bits();
    }
}

impl Castling {
    pub fn as_fen_str(self) -> String {
        if self.is_empty() {
            "-".to_owned()
        } else {
            let mut result = String::with_capacity(4);
            if self.contains(Self::WHITE_KINGSIDE) {
                result.push('K');
            }
            if self.contains(Self::WHITE_QUEENSIDE) {
                result.push('Q');
            }
            if self.contains(Self::BLACK_KINGSIDE) {
                result.push('k');
            }
            if self.contains(Self::BLACK_QUEENSIDE) {
                result.push('q');
            }
            result
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    board: BoardRepr,
    to_move: Color,
    castling: Castling,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    fullmoves: u32,
}

impl Board {
    pub fn new(
        board: [Option<Piece>; 64],
        to_move: Color,
        castling: Castling,
        en_passant: Option<Square>,
        halfmove_clock: u32,
        fullmoves: u32,
    ) -> Self {
        let board = BoardRepr::new(board);

        // TODO: calculate Zobrist hash

        Self {
            board,
            to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmoves,
        }
    }

    pub fn starting_position() -> Self {
        Self::try_parse_fen(STARTING_POSITION_FEN).unwrap()
    }

    pub fn fen(&self) -> String {
        fen::board_to_fen(self)
    }

    pub fn try_parse_fen(fen: &str) -> Option<Self> {
        fen::try_parse_fen(fen)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::starting_position()
    }
}
