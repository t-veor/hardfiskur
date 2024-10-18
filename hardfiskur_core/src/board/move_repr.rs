use std::{fmt::Debug, num::NonZeroU32};

use bitflags::bitflags;
use zerocopy_derive::FromZeros;

use super::{Piece, PieceType, Square};

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
        const DOUBLE_PAWN_PUSH = 0b0001 << 24;
        /// Whether this move is a castle. The move should be a double-step
        /// king move, and the direction can be determined by the direction the
        /// king moves in.
        const CASTLE           = 0b0010 << 24;
        /// Whether this move is an en passant capture.
        const EN_PASSANT       = 0b0100 << 24;
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
/// 3       2    2    1    1
/// 1       4    0    6    2      6      0
/// 0000_0XXX CCCC_MMMM PPPP_TTTTTT_FFFFFF
///       ^^^ ^^^^ ^^^^ ^^^^ ^^^^^^ ^^^^^^
///         |    |    |    |      |      |
///         |    |    |    |      |      +-- from square
///         |    |    |    |      +--------- to square
///         |    |    |    +---------------- promoted piece (0 if none)
///         |    |    +--------------------- moved piece
///         |    +-------------------------- captured piece (0 if none)
///         +------------------------------- move flags
/// ```
///
/// This representation allows a move to still be uniquely identifiable in a
/// given board state with the lower 16 bits (12 bits if the promotion piece is
/// not needed).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Move(NonZeroU32);

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
        let to = (to.get() as u32) << 6;
        let promotion = (match promotion {
            Some(piece) => piece.get() as u32,
            None => 0,
        }) << 12;
        let piece = (piece.get() as u32) << 16;
        let captured_piece = (match captured_piece {
            Some(piece) => piece.get() as u32,
            None => 0,
        }) << 20;

        let flags = flags.bits();

        unsafe {
            // Safety: piece cannot be zero, this big OR can't be zero either
            Self(NonZeroU32::new_unchecked(
                flags | promotion | captured_piece | piece | to | from,
            ))
        }
    }

    /// Returns the source square of the moved piece.
    pub const fn from_square(self) -> Square {
        Square::from_u8_unchecked((self.0.get() & 0x3F) as u8)
    }

    /// Returns the destination square of the moved piece.
    pub const fn to_square(self) -> Square {
        Square::from_u8_unchecked(((self.0.get() & 0xFC0) >> 6) as u8)
    }

    // Would really like this to be a const function, but alas
    /// Returns the piece that was moved.
    pub fn piece(self) -> Piece {
        Piece::try_from_u8(((self.0.get() & 0x0F0000) >> 16) as u8)
            .expect("invalid move representation encountered")
    }

    /// Returns if the piece that was moved was of the given type.
    pub const fn is_move_of(self, piece_type: PieceType) -> bool {
        let moved_piece_type = (self.0.get() & 0x070000) >> 16;
        moved_piece_type as u8 == piece_type as u8
    }

    /// Returns the piece that was captured, if any.
    ///
    /// This piece will be on [`to_square`][Self::to_square] unless this move is
    /// an en passant capture.
    ///
    /// If this move is an en passant capture, the square the captured pawn
    /// resided on can be determined using [`Self::en_passant_square`].
    pub const fn captured_piece(self) -> Option<Piece> {
        Piece::try_from_u8(((self.0.get() & 0xF00000) >> 20) as u8)
    }

    /// Returns if this move captures a piece of the given type.
    pub fn is_capture_of(self, piece_type: PieceType) -> bool {
        self.captured_piece()
            .is_some_and(|piece| piece.piece_type() == piece_type)
    }

    /// If this was a pawn move that reached the final rank, returns the
    /// promotion target for this pawn.
    pub const fn promotion(self) -> Option<Piece> {
        Piece::try_from_u8(((self.0.get() & 0xF000) >> 12) as u8)
    }

    /// Returns the special move flags for this move.
    pub const fn flags(self) -> MoveFlags {
        MoveFlags::from_bits_truncate(self.0.get())
    }

    /// Returns true if this move is a capture.
    pub const fn is_capture(self) -> bool {
        self.captured_piece().is_some()
    }

    /// Returns true if this move was an initial double-step move of a pawn.
    pub const fn is_double_pawn_push(self) -> bool {
        MoveFlags::from_bits_retain(self.0.get()).contains(MoveFlags::DOUBLE_PAWN_PUSH)
    }

    /// Returns true if this move was a castling move.
    pub const fn is_castle(self) -> bool {
        MoveFlags::from_bits_retain(self.0.get()).contains(MoveFlags::CASTLE)
    }

    /// Returns true if this move was an en passant capture.
    pub const fn is_en_passant(self) -> bool {
        MoveFlags::from_bits_retain(self.0.get()).contains(MoveFlags::EN_PASSANT)
    }

    /// If this move is an en passant capture, returns the square where the
    /// captured pawn was on (as it is different to )
    ///
    /// Note that this method does not actually check if this move was an en
    /// passant capture for performance reasons. The result of this method
    /// called on a non-en passant move is defined but may be unexpected.
    ///
    /// The result of this method is defined to be the square with the same rank
    /// as the start square, and the same file as the end square.
    pub const fn en_passant_square(self) -> Square {
        Square::new_unchecked(self.from_square().rank(), self.to_square().file())
    }

    /// If this move is a castling move, returns the source and destination
    /// squares of rook involved in the castle. (The `from_square` and
    /// `to_square` of this move are for the king.)
    ///
    /// Note that this method does not actually check if this move was a castle
    /// for performance reasons. The result of this method called on a
    /// non-castling move is defined but may be unexpected.
    pub const fn castling_rook_squares(self) -> (Square, Square) {
        let (from, to) = (self.from_square(), self.to_square());

        let rook_from =
            Square::new_unchecked(from.rank(), if from.file() < to.file() { 7 } else { 0 });
        let rook_to = Square::new_unchecked(from.rank(), (from.file() + to.file()) / 2);

        (rook_from, rook_to)
    }

    /// Returns true if this move is not a pawn move nor a capture.
    ///
    /// Note that this method does not check if the move is non-reversible due
    /// to losing castling rights.
    pub const fn is_reversible(self) -> bool {
        !self.is_move_of(PieceType::Pawn) && !self.is_capture()
    }

    /// Convenience alias for [`MoveBuilder::new`].
    pub const fn builder(from: Square, to: Square, piece: Piece) -> MoveBuilder {
        MoveBuilder::new(from, to, piece)
    }

    pub const fn butterfly_index(self) -> usize {
        (self.0.get() & 0xFFF) as usize
    }

    /// Convert this move into a pre-populated [`MoveBuilder`]. Useful for
    /// editing just one aspect of the move.
    pub fn into_builder(self) -> MoveBuilder {
        MoveBuilder {
            from: self.from_square(),
            to: self.to_square(),
            piece: self.piece(),
            captured_piece: self.captured_piece(),
            promotion: self.promotion(),
            flags: self.flags(),
        }
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from_square())
            .field("to", &self.to_square())
            .field("piece", &self.piece())
            .field("captured_piece", &self.captured_piece())
            .field("promotion", &self.promotion())
            .field("flags", &self.flags())
            .finish()
    }
}

/// Workaround type for supporting zerocopy's [`FromZeros`] trait.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromZeros)]
#[repr(transparent)]
pub struct OptionalMove(Option<NonZeroU32>);

impl OptionalMove {
    #[inline]
    pub fn from_option_move(value: Option<Move>) -> Self {
        Self(value.map(|value| value.0))
    }

    #[inline]
    pub fn as_option_move(self) -> Option<Move> {
        self.0.map(Move)
    }
}

impl From<Option<Move>> for OptionalMove {
    fn from(value: Option<Move>) -> Self {
        Self::from_option_move(value)
    }
}

impl Debug for OptionalMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptionalMove")
            .field(&self.as_option_move())
            .finish()
    }
}

/// Builder struct for convenient construction of a [`Move`].
///
/// The [`Move`] struct requires all its fields up-front in its constructor,
/// which may be annoying when in most cases you don't care for specifying the
/// promotion or the castling state, etc. This struct provides a streaming
/// interface to incrementally build a move, before calling
/// [`MoveBuilder::build`] to finalise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MoveBuilder {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub captured_piece: Option<Piece>,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
}

impl MoveBuilder {
    /// Create a new [`MoveBuilder`].
    ///
    /// Every move requires a source and destination square, as well as the
    /// piece being moved, so these are required in this constructor.
    pub const fn new(from: Square, to: Square, piece: Piece) -> Self {
        Self {
            from,
            to,
            piece,
            captured_piece: None,
            promotion: None,
            flags: MoveFlags::empty(),
        }
    }

    /// Sets the captured piece of this move.
    pub const fn captures(self, captured_piece: Piece) -> Self {
        Self {
            captured_piece: Some(captured_piece),
            ..self
        }
    }

    /// Sets the promotion target of this move.
    pub const fn promotes_to(self, promotion: PieceType) -> Self {
        Self {
            promotion: Some(promotion.with_color(self.piece.color())),
            ..self
        }
    }

    /// Sets the flags to [`MoveFlags::DOUBLE_PAWN_PUSH`].
    pub const fn is_double_pawn_push(self) -> Self {
        Self {
            flags: MoveFlags::DOUBLE_PAWN_PUSH,
            ..self
        }
    }

    /// Sets the flags to [`MoveFlags::CASTLE`].
    pub const fn is_castle(self) -> Self {
        Self {
            flags: MoveFlags::CASTLE,
            ..self
        }
    }

    /// Sets the flags to [`MoveFlags::EN_PASSANT`].
    pub const fn is_en_passant(self) -> Self {
        Self {
            flags: MoveFlags::EN_PASSANT,
            ..self
        }
    }

    /// Finalises and builds the [`Move`].
    pub const fn build(self) -> Move {
        Move::new(
            self.from,
            self.to,
            self.piece,
            self.captured_piece,
            self.promotion,
            self.flags,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    impl MoveBuilder {
        fn assert_eq(&self, the_move: Move) {
            assert_eq!(self.from, the_move.from_square());
            assert_eq!(self.to, the_move.to_square());
            assert_eq!(self.piece, the_move.piece());
            assert_eq!(self.captured_piece, the_move.captured_piece());
            assert_eq!(self.promotion, the_move.promotion());
            assert_eq!(self.flags, the_move.flags());
        }
    }

    const EN_PASSANT_CAPTURES: &[MoveBuilder] = &[
        MoveBuilder::new(Square::B5, Square::A6, Piece::WHITE_PAWN)
            .captures(Piece::BLACK_PAWN)
            .is_en_passant(),
        MoveBuilder::new(Square::F4, Square::G3, Piece::BLACK_PAWN)
            .captures(Piece::WHITE_PAWN)
            .is_en_passant(),
    ];

    const PROMOTIONS: &[MoveBuilder] = &[
        MoveBuilder::new(Square::C7, Square::C8, Piece::WHITE_PAWN).promotes_to(PieceType::Queen),
        MoveBuilder::new(Square::H2, Square::G1, Piece::BLACK_PAWN)
            .captures(Piece::WHITE_BISHOP)
            .promotes_to(PieceType::Rook),
    ];

    const DOUBLE_PAWN_PUSHES: &[MoveBuilder] = &[
        MoveBuilder::new(Square::D2, Square::D4, Piece::WHITE_PAWN).is_double_pawn_push(),
        MoveBuilder::new(Square::E5, Square::E7, Piece::BLACK_PAWN).is_double_pawn_push(),
    ];

    const CASTLES: &[MoveBuilder] = &[
        MoveBuilder::new(Square::E1, Square::G1, Piece::WHITE_KING).is_castle(),
        MoveBuilder::new(Square::E8, Square::C8, Piece::BLACK_KING).is_castle(),
    ];

    const CAPTURES: &[MoveBuilder] = &[
        MoveBuilder::new(Square::D3, Square::H7, Piece::BLACK_BISHOP).captures(Piece::WHITE_QUEEN),
        MoveBuilder::new(Square::E4, Square::C5, Piece::WHITE_KNIGHT).captures(Piece::BLACK_ROOK),
    ];

    const QUIET_MOVES: &[MoveBuilder] = &[
        MoveBuilder::new(Square::B2, Square::B5, Piece::BLACK_QUEEN),
        MoveBuilder::new(Square::C6, Square::C7, Piece::WHITE_PAWN),
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
            let the_move = move_case.build();
            move_case.assert_eq(the_move);

            for piece_type in PieceType::ALL {
                assert_eq!(
                    the_move.is_move_of(piece_type),
                    move_case.piece.piece_type() == piece_type
                );
                assert_eq!(
                    the_move.is_capture_of(piece_type),
                    move_case
                        .captured_piece
                        .is_some_and(|piece| piece.piece_type() == piece_type)
                );
            }
        }
    }

    #[test]
    fn move_is_capture() {
        let capture_test_moves = EN_PASSANT_CAPTURES.iter().chain(CAPTURES);

        for move_case in capture_test_moves {
            let the_move = move_case.build();
            assert!(the_move.is_capture());
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.build();
            assert!(!the_move.is_capture());
        }
    }

    #[test]
    fn move_is_double_pawn_push() {
        for move_case in DOUBLE_PAWN_PUSHES {
            let the_move = move_case.build();
            assert!(the_move.is_double_pawn_push());
            assert!(the_move.flags().contains(MoveFlags::DOUBLE_PAWN_PUSH));
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.build();
            assert!(!the_move.is_double_pawn_push());
            assert!(!the_move.flags().contains(MoveFlags::DOUBLE_PAWN_PUSH));
        }
    }

    #[test]
    fn move_is_castle() {
        for move_case in CASTLES {
            let the_move = move_case.build();
            assert!(the_move.is_castle());
            assert!(the_move.flags().contains(MoveFlags::CASTLE));
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.build();
            assert!(!the_move.is_castle());
            assert!(!the_move.flags().contains(MoveFlags::CASTLE));
        }
    }

    #[test]
    fn move_is_en_passant() {
        for move_case in EN_PASSANT_CAPTURES {
            let the_move = move_case.build();
            assert!(the_move.is_en_passant());
            assert!(the_move.flags().contains(MoveFlags::EN_PASSANT));
        }

        for move_case in QUIET_MOVES {
            let the_move = move_case.build();
            assert!(!the_move.is_en_passant());
            assert!(!the_move.flags().contains(MoveFlags::EN_PASSANT));
        }
    }

    #[test]
    fn move_into_builder() {
        let all_test_moves = EN_PASSANT_CAPTURES
            .iter()
            .chain(PROMOTIONS)
            .chain(DOUBLE_PAWN_PUSHES)
            .chain(CASTLES)
            .chain(CAPTURES)
            .chain(QUIET_MOVES);

        for move_case in all_test_moves {
            let the_move = move_case.build();
            let new_builder = the_move.into_builder();

            assert_eq!(*move_case, new_builder);
        }
    }
}
