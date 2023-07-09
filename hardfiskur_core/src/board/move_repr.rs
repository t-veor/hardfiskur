use std::fmt::Debug;

use bitflags::bitflags;

use super::{Piece, Square};

bitflags! {
    /// Flags representing special kinds of moves that need special handling.
    ///
    /// Only one of these flags should be set at any one time.
    ///
    /// Can be accessed from a move via the [`Move::flags`] method. Convenience
    /// methods [`Move::is_double_pawn_push`], [`Move::is_castle`], and
    /// [`Move::is_en_passant`] are provided.
    ///
    /// These are stored in the highest 4 bits of a [`u32`] so it plays nicely
    /// with the [`Move`] representation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MoveFlags: u32 {
        /// Whether this move is the initial double move of a pawn. This is
        /// useful for knowing if en passant is available on the next turn.
        const DOUBLE_PAWN_PUSH = 0b0001 << 28;
        /// Whether this move is a castle. The move should be a double-step
        /// king move, and the direction can be determined by the direction the
        /// king moves in.
        const CASTLE           = 0b0010 << 28;
        /// Whether this move is an en passant capture.
        const EN_PASSANT       = 0b0100 << 28;
    }
}

/// Compact representation of a chess move.
///
/// Assuming the board is in a valid state that allows the move, this structure
/// contains all the data required to unambiguously perform the move.
///
/// Assuming the board is in a state immediately after this move has been
/// performed, this structure also contains all the data required to
/// unambiguously undo this move.
///
/// The internal representation is a 32-bit integer which packs all the fields
/// of the move thusly:
/// ```txt
/// 3  2    2    2    1
/// 1  8    4    0    6        8        0
/// 0XXX_PPPP CCCC_MMMM 00TTTTTT 00FFFFFF
/// ^^^^ ^^^^ ^^^^ ^^^^   ^^^^^^   ^^^^^^
///    |    |    |    |        |        |
///    |    |    |    |        |        +-- from square
///    |    |    |    |        +----------- to square
///    |    |    |    +-------------------- moved piece
///    |    |    +------------------------- captured piece (0 if none)
///    |    +------------------------------ promoted to piece (0 if none)
///    +----------------------------------- move flags
/// ```
#[derive(Clone, Copy)]
pub struct Move(u32);

impl Move {
    /// Constructs a new [`Move`].
    ///
    /// Note that this method will not check to see if the move performed is
    /// actually legal, i.e. making sure a bishop move actually moved on a
    /// diagonal, ensuring that the [`MoveFlags::EN_PASSANT`] flag is only used
    /// for an en passant capture, etc.
    pub const fn new(
        from: Square,
        to: Square,
        piece: Piece,
        captured_piece: Option<Piece>,
        promotion: Option<Piece>,
        flags: MoveFlags,
    ) -> Self {
        let from = from.get() as u32;
        let to = (to.get() as u32) << 8;
        let piece = (piece.get() as u32) << 16;
        let captured_piece = (match captured_piece {
            Some(piece) => piece.get() as u32,
            None => 0,
        }) << 20;
        let promotion = (match promotion {
            Some(piece) => piece.get() as u32,
            None => 0,
        }) << 24;
        let flags = flags.bits();

        Self(flags | promotion | captured_piece | piece | to | from)
    }

    /// Returns the source square of the moved piece.
    pub const fn from_square(self) -> Square {
        Square::from_u8_unchecked((self.0 & 0x3F) as u8)
    }

    /// Returns the destination square of the moved piece.
    pub const fn to_square(self) -> Square {
        Square::from_u8_unchecked(((self.0 & 0x3F00) >> 8) as u8)
    }

    // Would really like this to be a const function, but alas
    /// Returns the piece that was moved.
    pub fn piece(self) -> Piece {
        Piece::try_from_u8(((self.0 & 0x0F0000) >> 16) as u8)
            .expect("invalid move representation encountered")
    }

    /// Returns the piece that was captured, if any.
    ///
    /// This piece will be on [`to_square`][Self::to_square] unless this move is
    /// an en passant capture.
    ///
    /// If this move is an en passant capture, the square the captured pawn
    /// resided on can be determined thusly:
    /// ```
    /// # use hardfiskur_core::board::{Square, Move};
    /// fn en_passant_captured_pawn_square(the_move: Move) -> Square {
    ///     Square::new_unchecked(
    ///         the_move.from_square().rank(),
    ///         the_move.to_square().file(),
    ///     )
    /// }
    /// ```
    pub const fn captured_piece(self) -> Option<Piece> {
        Piece::try_from_u8(((self.0 & 0xF00000) >> 20) as u8)
    }

    /// If this was a pawn move that reached the final rank, returns the
    /// promotion target for this pawn.
    pub const fn promotion(self) -> Option<Piece> {
        Piece::try_from_u8(((self.0 & 0x0F000000) >> 24) as u8)
    }

    /// Returns the special move flags for this move.
    pub const fn flags(self) -> MoveFlags {
        MoveFlags::from_bits_truncate(self.0)
    }

    /// Returns true if this move is a capture.
    pub const fn is_capture(self) -> bool {
        self.captured_piece().is_some()
    }

    /// Returns true if this move was an initial double-step move of a pawn.
    pub const fn is_double_pawn_push(self) -> bool {
        MoveFlags::from_bits_retain(self.0).contains(MoveFlags::DOUBLE_PAWN_PUSH)
    }

    /// Returns true if this move was a castling move.
    pub const fn is_castle(self) -> bool {
        MoveFlags::from_bits_retain(self.0).contains(MoveFlags::CASTLE)
    }

    /// Returns true if this move was an en passant capture.
    pub const fn is_en_passant(self) -> bool {
        MoveFlags::from_bits_retain(self.0).contains(MoveFlags::EN_PASSANT)
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from_square())
            .field("to", &self.to_square())
            .field("piece", &self.piece())
            .field("captured_pieced", &self.captured_piece())
            .field("promotion", &self.promotion())
            .field("flags", &self.flags())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Debug, Clone)]
    struct ExplodedMove {
        from: Square,
        to: Square,
        piece: Piece,
        captured_piece: Option<Piece>,
        promotion: Option<Piece>,
        flags: MoveFlags,
    }

    impl ExplodedMove {
        fn create_move(&self) -> Move {
            Move::new(
                self.from,
                self.to,
                self.piece,
                self.captured_piece,
                self.promotion,
                self.flags,
            )
        }

        fn assert_eq(&self, the_move: Move) {
            assert_eq!(self.from, the_move.from_square());
            assert_eq!(self.to, the_move.to_square());
            assert_eq!(self.piece, the_move.piece());
            assert_eq!(self.captured_piece, the_move.captured_piece());
            assert_eq!(self.promotion, the_move.promotion());
            assert_eq!(self.flags, the_move.flags());
        }
    }

    const EN_PASSANT_CAPTURES: &[ExplodedMove] = &[
        ExplodedMove {
            from: Square::new_unchecked(4, 1),
            to: Square::new_unchecked(5, 0),
            piece: Piece::WHITE_PAWN,
            captured_piece: Some(Piece::BLACK_PAWN),
            promotion: None,
            flags: MoveFlags::EN_PASSANT,
        },
        ExplodedMove {
            from: Square::new_unchecked(3, 5),
            to: Square::new_unchecked(2, 6),
            piece: Piece::BLACK_PAWN,
            captured_piece: Some(Piece::WHITE_PAWN),
            promotion: None,
            flags: MoveFlags::EN_PASSANT,
        },
    ];

    const PROMOTIONS: &[ExplodedMove] = &[
        ExplodedMove {
            from: Square::new_unchecked(6, 2),
            to: Square::new_unchecked(7, 2),
            piece: Piece::WHITE_PAWN,
            captured_piece: None,
            promotion: Some(Piece::WHITE_QUEEN),
            flags: MoveFlags::empty(),
        },
        ExplodedMove {
            from: Square::new_unchecked(1, 7),
            to: Square::new_unchecked(0, 6),
            piece: Piece::BLACK_PAWN,
            captured_piece: Some(Piece::WHITE_BISHOP),
            promotion: Some(Piece::BLACK_ROOK),
            flags: MoveFlags::empty(),
        },
    ];

    const DOUBLE_PAWN_PUSHES: &[ExplodedMove] = &[
        ExplodedMove {
            from: Square::new_unchecked(1, 3),
            to: Square::new_unchecked(3, 3),
            piece: Piece::WHITE_PAWN,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::DOUBLE_PAWN_PUSH,
        },
        ExplodedMove {
            from: Square::new_unchecked(6, 4),
            to: Square::new_unchecked(4, 4),
            piece: Piece::BLACK_PAWN,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::DOUBLE_PAWN_PUSH,
        },
    ];

    const CASTLES: &[ExplodedMove] = &[
        ExplodedMove {
            from: Square::new_unchecked(0, 4),
            to: Square::new_unchecked(0, 6),
            piece: Piece::WHITE_KING,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::CASTLE,
        },
        ExplodedMove {
            from: Square::new_unchecked(7, 4),
            to: Square::new_unchecked(7, 2),
            piece: Piece::BLACK_KING,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::CASTLE,
        },
    ];

    const CAPTURES: &[ExplodedMove] = &[
        ExplodedMove {
            from: Square::new_unchecked(2, 3),
            to: Square::new_unchecked(6, 7),
            piece: Piece::BLACK_BISHOP,
            captured_piece: Some(Piece::WHITE_QUEEN),
            promotion: None,
            flags: MoveFlags::empty(),
        },
        ExplodedMove {
            from: Square::new_unchecked(3, 4),
            to: Square::new_unchecked(4, 2),
            piece: Piece::WHITE_KNIGHT,
            captured_piece: Some(Piece::BLACK_ROOK),
            promotion: None,
            flags: MoveFlags::empty(),
        },
    ];

    const QUIET_MOVES: &[ExplodedMove] = &[
        ExplodedMove {
            from: Square::new_unchecked(1, 1),
            to: Square::new_unchecked(4, 1),
            piece: Piece::BLACK_QUEEN,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::empty(),
        },
        ExplodedMove {
            from: Square::new_unchecked(5, 2),
            to: Square::new_unchecked(6, 2),
            piece: Piece::WHITE_PAWN,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::empty(),
        },
    ];

    #[test]
    fn move_create_and_unpack() {
        let all_test_moves = EN_PASSANT_CAPTURES
            .iter()
            .chain(PROMOTIONS)
            .chain(DOUBLE_PAWN_PUSHES)
            .chain(CASTLES)
            .chain(CAPTURES)
            .chain(QUIET_MOVES);

        for move_case in all_test_moves {
            let the_move = move_case.create_move();
            move_case.assert_eq(the_move);
        }
    }

    #[test]
    fn move_is_capture() {
        let capture_test_moves = EN_PASSANT_CAPTURES.iter().chain(CAPTURES);

        for move_case in capture_test_moves {
            let the_move = move_case.create_move();
            assert!(the_move.is_capture());
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.create_move();
            assert!(!the_move.is_capture());
        }
    }

    #[test]
    fn move_is_double_pawn_push() {
        for move_case in DOUBLE_PAWN_PUSHES {
            let the_move = move_case.create_move();
            assert!(the_move.is_double_pawn_push());
            assert!(the_move.flags().contains(MoveFlags::DOUBLE_PAWN_PUSH));
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.create_move();
            assert!(!the_move.is_double_pawn_push());
            assert!(!the_move.flags().contains(MoveFlags::DOUBLE_PAWN_PUSH));
        }
    }

    #[test]
    fn move_is_castle() {
        for move_case in CASTLES {
            let the_move = move_case.create_move();
            assert!(the_move.is_castle());
            assert!(the_move.flags().contains(MoveFlags::CASTLE));
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.create_move();
            assert!(!the_move.is_castle());
            assert!(!the_move.flags().contains(MoveFlags::CASTLE));
        }
    }

    #[test]
    fn move_is_en_passant() {
        for move_case in EN_PASSANT_CAPTURES {
            let the_move = move_case.create_move();
            assert!(the_move.is_en_passant());
            assert!(the_move.flags().contains(MoveFlags::EN_PASSANT));
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.create_move();
            assert!(!the_move.is_en_passant());
            assert!(!the_move.flags().contains(MoveFlags::EN_PASSANT));
        }
    }
}
