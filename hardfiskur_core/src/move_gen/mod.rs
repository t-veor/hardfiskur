use arrayvec::ArrayVec;
use bitflags::bitflags;

use crate::board::{Bitboard, BoardRepr, Castling, Color, Move, PieceType, Square};

use self::lookups::Lookups;

pub mod bitboard_utils;
pub mod lookups;
pub mod magic;
mod pseudo_legal;

/// Maximum number of moves that could occur in a legal position, used for
/// stack-allocating a vector to hold moves.
///
/// The actual number appears to be 218 in this position:
///
/// R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1
///
/// But 256 is a nice number and a good buffer in case there could be more.
pub const MAX_MOVES: usize = 256;

const POSSIBLE_PROMOTIONS: &[PieceType] = &[
    PieceType::Queen,
    PieceType::Knight,
    PieceType::Rook,
    PieceType::Bishop,
];

pub type MoveVec = ArrayVec<Move, MAX_MOVES>;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveGenFlags: u8 {
        const GEN_CAPTURES = 0b01;
        const GEN_QUIET_MOVES = 0b10;
    }
}

impl Default for MoveGenFlags {
    fn default() -> Self {
        Self::all()
    }
}

/// Masks used by the pseudo-legal move generation methods that restrict the
/// kinds of moves produced.
///
/// These masks are typically all set to [`Bitboard::ALL`], but in cases where
/// there is a check or pin, they may be set so that the psuedo-legal move
/// generation methods handle checks and pins correctly, giving us legal move
/// generation for free without (much) special handling when the king is in
/// check.
///
/// The reason the capture and push masks are separate is due to en passant --
/// it may have sufficed to have a single "push" mask represent both squares
/// that can be moved onto, except in the case of en passant, the square that
/// the capturing piece lands on is different to the square that the captured
/// piece is on. Two different masks are needed to handle the two situations
/// where a pawn which can be en passant captured is giving check, and where
/// capturing a pawn via en passant also blocks a check.
///
/// Inspiration for this style of move generation comes from
/// <https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/>.
#[derive(Debug, Clone)]
struct MoveGenMasks {
    /// Pieces are only capturable if they are in this mask. This is normally
    /// [`Bitboard::ALL`].
    ///
    /// If the king is in check (and there is only one checker), this mask will
    /// consist of only the square of the checker -- indicating that any
    /// capturing moves generated must capture the checker.
    capture: Bitboard,
    /// Squares can only be moved onto if they are in this mask. This is
    /// normally [`Bitboard::ALL`].
    ///
    /// If the king is in check by a sliding piece, this mask will consist of
    /// the squares in between the king and the checker -- indicating that any
    /// moves generated must result in the moved piece landing on a square that
    /// blocks the check.
    push: Bitboard,
    /// Pieces can only be moved if they are in this mask. This is normally
    /// [`Bitboard:ALL`].
    ///
    /// Pieces which are absolutely pinned will be filtered out of this mask and
    /// their move generation will be handled separately from the pseudo-legal
    /// move generation.
    movable: Bitboard,
}

impl Default for MoveGenMasks {
    fn default() -> Self {
        Self {
            capture: Bitboard::ALL,
            push: Bitboard::ALL,
            movable: Bitboard::ALL,
        }
    }
}

pub struct MoveGenerator<'board, 'moves> {
    lookups: &'static Lookups,
    board: &'board BoardRepr,
    to_move: Color,
    empty: Bitboard,
    occupied: Bitboard,
    en_passant: Option<Square>,
    castling: Castling,
    flags: MoveGenFlags,
    out_moves: &'moves mut MoveVec,
}

impl<'board, 'moves> MoveGenerator<'board, 'moves> {
    pub fn new(
        board: &'board BoardRepr,
        to_move: Color,
        en_passant: Option<Square>,
        castling: Castling,
        flags: MoveGenFlags,
        out_moves: &'moves mut MoveVec,
    ) -> Self {
        Self {
            lookups: Lookups::get_instance(),
            board,
            to_move,
            empty: board.empty(),
            occupied: board.occupied(),
            en_passant,
            castling,
            flags,
            out_moves,
        }
    }
}
