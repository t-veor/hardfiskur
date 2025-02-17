use std::{
    fmt::{Debug, Display, Write},
    num::NonZeroU8,
    str::FromStr,
};

use num_derive::{FromPrimitive, ToPrimitive};

/// Represents the type of a piece, but not its colour.
///
/// Piece types are assigned integers 1-6, so that they fit in 3-bits, and also
/// that 0 is unused -- this is so that Rust can optimise [`Option<PieceType>`]
/// to use the value 0 for [`None`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
pub enum PieceType {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl PieceType {
    pub const ALL: [PieceType; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    /// Convenience method for constructing a white [`Piece`].
    pub const fn white(self) -> Piece {
        Piece::white(self)
    }

    /// Convenience method for constructing a black [`Piece`].
    pub const fn black(self) -> Piece {
        Piece::black(self)
    }

    /// Convenience method for constructing a [`Piece`] with the supplied
    /// [`Color`].
    pub const fn with_color(self, color: Color) -> Piece {
        Piece::new(color, self)
    }

    /// Get the FEN representation of the white version of this piece.
    pub const fn as_uppercase_char(self) -> char {
        match self {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
        }
    }

    /// Get the FEN representation of the black version of this piece.
    pub const fn as_lowercase_char(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }

    /// Returns true if this piece type is a slider (i.e. a bishop, rook, or
    /// queen).
    pub const fn is_slider(self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }

    pub const fn index(self) -> usize {
        self as usize - 1
    }
}

/// Convenience aliases.
impl PieceType {
    pub const P: Self = Self::Pawn;
    pub const N: Self = Self::Knight;
    pub const B: Self = Self::Bishop;
    pub const R: Self = Self::Rook;
    pub const Q: Self = Self::Queen;
    pub const K: Self = Self::King;
}

impl From<Piece> for PieceType {
    fn from(value: Piece) -> Self {
        value.piece_type()
    }
}

/// Represents a player in a chess game (either white or black).
///
/// Used for representing who a [`Piece`] belongs to. Uses 0 for white and 8 for
/// black, such that the [`Piece`] representation can simply be formed from a
/// bitwise or of the [`Color`] and [`PieceType`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    White = 0,
    Black = 8,
}

impl Color {
    /// Returns true if this is [`Color::White`].
    pub const fn is_white(self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false,
        }
    }

    /// Returns true if this is [`Color::Black`].
    pub const fn is_black(self) -> bool {
        !self.is_white()
    }

    /// Inverts the color, i.e. maps [`Color::White`] to [`Color::Black`] and
    /// vice versa.
    pub const fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

/// Represents a piece in a chess game.
///
/// Internal representation is a 4 bit integer, formed by a bitwise-or of the
/// [`Color`] and [`PieceType`]. Since [`PieceType`] can never be 0, Rust can
/// optimise the [`None`] of an [`Option<Piece>`] to be represented by 0.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece(NonZeroU8);

impl Piece {
    /// Constructs a [`Piece`] from a [`Color`] and a [`PieceType`].
    pub const fn new(color: Color, piece_type: PieceType) -> Self {
        // Safety: piece_type as u8 can never be 0
        unsafe { Self(NonZeroU8::new_unchecked(color as u8 | piece_type as u8)) }
    }

    /// Convenience method for constructing a white [`Piece`] from a
    /// [`PieceType`].
    pub const fn white(piece_type: PieceType) -> Self {
        Self::new(Color::White, piece_type)
    }

    /// Convenience method for constructing a black [`Piece`] from a
    /// [`PieceType`].
    pub const fn black(piece_type: PieceType) -> Self {
        Self::new(Color::Black, piece_type)
    }

    /// Convenience method for constructing a pawn of the given [`Color`].
    pub const fn pawn(color: Color) -> Self {
        Self::new(color, PieceType::Pawn)
    }

    /// Convenience method for constructing a knight of the given [`Color`].
    pub const fn knight(color: Color) -> Self {
        Self::new(color, PieceType::Knight)
    }

    /// Convenience method for constructing a bishop of the given [`Color`].
    pub const fn bishop(color: Color) -> Self {
        Self::new(color, PieceType::Bishop)
    }

    /// Convenience method for constructing a rook of the given [`Color`].
    pub const fn rook(color: Color) -> Self {
        Self::new(color, PieceType::Rook)
    }

    /// Convenience method for constructing a queen of the given [`Color`].
    pub const fn queen(color: Color) -> Self {
        Self::new(color, PieceType::Queen)
    }

    /// Convenience method for constructing a king of the given [`Color`].
    pub const fn king(color: Color) -> Self {
        Self::new(color, PieceType::King)
    }

    /// Constructs a [`Piece`] from its 4-bit representation.
    ///
    /// Note that since Rust does not have a 4-bit type, `value` will first be
    /// truncated to 4 bits. Then, if the value is invalid, [`None`] will be
    /// returned.
    ///
    /// This means that you can also use this method to construct a
    /// [`Option<Piece>`] from its 4-bit representation.
    pub const fn try_from_u8(value: u8) -> Option<Self> {
        let value = value & 0x0F;
        if value & 0x07 == 0 || value & 0x07 == 7 {
            None
        } else {
            // Safety: if value was 0, then value & 0x07 == 0, so we wouldn't
            // get to this branch
            unsafe { Some(Self(NonZeroU8::new_unchecked(value))) }
        }
    }

    /// Returns the [`Color`] of this piece.
    pub const fn color(self) -> Color {
        if self.0.get() & 8 > 0 {
            Color::Black
        } else {
            Color::White
        }
    }

    /// Returns if this piece is white.
    pub const fn is_white(self) -> bool {
        self.color().is_white()
    }

    /// Returns if this piece is black.
    pub const fn is_black(self) -> bool {
        self.color().is_black()
    }

    /// Returns if this piece is a pawn.
    pub const fn is_pawn(self) -> bool {
        matches!(self.piece_type(), PieceType::Pawn)
    }

    /// Returns if this piece is a knight.
    pub const fn is_knight(self) -> bool {
        matches!(self.piece_type(), PieceType::Knight)
    }

    /// Returns if this piece is a bishop.
    pub const fn is_bishop(self) -> bool {
        matches!(self.piece_type(), PieceType::Bishop)
    }

    /// Returns if this piece is a rook.
    pub const fn is_rook(self) -> bool {
        matches!(self.piece_type(), PieceType::Rook)
    }

    /// Returns if this piece is a queen.
    pub const fn is_queen(self) -> bool {
        matches!(self.piece_type(), PieceType::Queen)
    }

    /// Returns if this piece is a king.
    pub const fn is_king(self) -> bool {
        matches!(self.piece_type(), PieceType::King)
    }

    /// Returns the [`PieceType`] of this piece.
    pub const fn piece_type(self) -> PieceType {
        match self.0.get() & 0x07 {
            1 => PieceType::Pawn,
            2 => PieceType::Knight,
            3 => PieceType::Bishop,
            4 => PieceType::Rook,
            5 => PieceType::Queen,
            6 => PieceType::King,
            _ => {
                // This one unreachable_unchecked raises NPS from 2.86m/s to
                // 3.34 m/s. To not be too crazy, with debug_assertions turned
                // on I replace this with a regular safe unreachable.
                #[cfg(debug_assertions)]
                {
                    unreachable!()
                }
                // Safety - self.0 should always be an OR of Color (0 or 8) and
                // PieceType (1-6), so the xor with 7 should always extract a
                // PieceType
                #[cfg(not(debug_assertions))]
                unsafe {
                    std::hint::unreachable_unchecked()
                }
            }
        }
    }

    /// Returns the internal 4-bit representation of this piece.
    pub const fn get(self) -> u8 {
        self.0.get()
    }

    /// Returns the internal 4-bit representation of this piece as a
    /// [`NonZeroU8`].
    pub const fn get_constrained(self) -> NonZeroU8 {
        self.0
    }

    pub const fn as_fen_char(self) -> char {
        if self.color().is_white() {
            self.piece_type().as_uppercase_char()
        } else {
            self.piece_type().as_lowercase_char()
        }
    }

    pub fn as_unicode_char(self) -> char {
        let base = match self.color() {
            Color::White => 0x2654,
            Color::Black => 0x265a,
        };
        let offset = match self.piece_type() {
            PieceType::Pawn => 5,
            PieceType::Knight => 4,
            PieceType::Bishop => 3,
            PieceType::Rook => 2,
            PieceType::Queen => 1,
            PieceType::King => 0,
        };
        char::from_u32(base + offset).unwrap()
    }

    pub const fn try_from_fen_char(c: char) -> Option<Self> {
        let piece_type = match c.to_ascii_uppercase() {
            'P' => PieceType::Pawn,
            'N' => PieceType::Knight,
            'B' => PieceType::Bishop,
            'R' => PieceType::Rook,
            'Q' => PieceType::Queen,
            'K' => PieceType::King,
            _ => return None,
        };

        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };

        Some(Self::new(color, piece_type))
    }

    pub const fn is_slider(self) -> bool {
        self.piece_type().is_slider()
    }
}

/// Convenient constants for specifying specific pieces.
impl Piece {
    pub const WHITE_PAWN: Piece = Piece::new(Color::White, PieceType::Pawn);
    pub const WHITE_KNIGHT: Piece = Piece::new(Color::White, PieceType::Knight);
    pub const WHITE_BISHOP: Piece = Piece::new(Color::White, PieceType::Bishop);
    pub const WHITE_ROOK: Piece = Piece::new(Color::White, PieceType::Rook);
    pub const WHITE_QUEEN: Piece = Piece::new(Color::White, PieceType::Queen);
    pub const WHITE_KING: Piece = Piece::new(Color::White, PieceType::King);
    pub const BLACK_PAWN: Piece = Piece::new(Color::Black, PieceType::Pawn);
    pub const BLACK_KNIGHT: Piece = Piece::new(Color::Black, PieceType::Knight);
    pub const BLACK_BISHOP: Piece = Piece::new(Color::Black, PieceType::Bishop);
    pub const BLACK_ROOK: Piece = Piece::new(Color::Black, PieceType::Rook);
    pub const BLACK_QUEEN: Piece = Piece::new(Color::Black, PieceType::Queen);
    pub const BLACK_KING: Piece = Piece::new(Color::Black, PieceType::King);
}

impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Piece")
            .field("color", &self.color())
            .field("piece_type", &self.piece_type())
            .finish()
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.as_fen_char())
    }
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            s.chars()
                .next()
                .and_then(Piece::try_from_fen_char)
                .ok_or(())
        } else {
            Err(())
        }
    }
}

impl From<(Color, PieceType)> for Piece {
    fn from((color, piece_type): (Color, PieceType)) -> Self {
        Self::new(color, piece_type)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    const ALL_PIECE_TYPES: &[PieceType] = &[
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];

    const ALL_COLORS: &[Color] = &[Color::White, Color::Black];

    const SLIDERS: &[PieceType] = &[PieceType::Bishop, PieceType::Rook, PieceType::Queen];
    const NON_SLIDERS: &[PieceType] = &[PieceType::Pawn, PieceType::Knight, PieceType::King];

    const PIECES_BY_REPRESENTATIONS: &[(u8, Option<Piece>)] = &[
        (0, None),
        (1, Some(Piece::WHITE_PAWN)),
        (2, Some(Piece::WHITE_KNIGHT)),
        (3, Some(Piece::WHITE_BISHOP)),
        (4, Some(Piece::WHITE_ROOK)),
        (5, Some(Piece::WHITE_QUEEN)),
        (6, Some(Piece::WHITE_KING)),
        (7, None),
        (8, None),
        (9, Some(Piece::BLACK_PAWN)),
        (10, Some(Piece::BLACK_KNIGHT)),
        (11, Some(Piece::BLACK_BISHOP)),
        (12, Some(Piece::BLACK_ROOK)),
        (13, Some(Piece::BLACK_QUEEN)),
        (14, Some(Piece::BLACK_KING)),
        (15, None),
    ];

    #[test]
    fn piece_type_convenience_constructors() {
        for piece_type in ALL_PIECE_TYPES {
            assert!(piece_type.white().is_white());
        }

        for piece_type in ALL_PIECE_TYPES {
            assert!(piece_type.black().is_black());
        }

        for piece_type in ALL_PIECE_TYPES {
            for &color in ALL_COLORS {
                assert_eq!(piece_type.with_color(color).color(), color);
            }
        }
    }

    #[test]
    fn piece_type_as_uppercase() {
        let cases = [
            (PieceType::Pawn, 'P'),
            (PieceType::Knight, 'N'),
            (PieceType::Bishop, 'B'),
            (PieceType::Rook, 'R'),
            (PieceType::Queen, 'Q'),
            (PieceType::King, 'K'),
        ];

        for (piece_type, expected) in cases {
            assert_eq!(piece_type.as_uppercase_char(), expected);
        }
    }

    #[test]
    fn piece_type_as_lowercase() {
        let cases = [
            (PieceType::Pawn, 'p'),
            (PieceType::Knight, 'n'),
            (PieceType::Bishop, 'b'),
            (PieceType::Rook, 'r'),
            (PieceType::Queen, 'q'),
            (PieceType::King, 'k'),
        ];
        for (piece_type, expected) in cases {
            assert_eq!(piece_type.as_lowercase_char(), expected);
        }
    }

    #[test]
    fn piece_type_is_slider() {
        for piece_type in SLIDERS {
            assert!(piece_type.is_slider())
        }
        for piece_type in NON_SLIDERS {
            assert!(!piece_type.is_slider())
        }
    }

    #[test]
    fn color_default_is_white() {
        assert_eq!(Color::default(), Color::White);
    }

    #[test]
    fn color_is_white_and_is_black() {
        assert!(Color::White.is_white());
        assert!(!Color::Black.is_white());

        assert!(!Color::White.is_black());
        assert!(Color::Black.is_black());
    }

    #[test]
    fn color_flip() {
        assert_eq!(Color::White.flip(), Color::Black);
        assert_eq!(Color::Black.flip(), Color::White);
    }

    #[test]
    fn piece_create_and_unpack() {
        for &color in ALL_COLORS {
            for &piece_type in ALL_PIECE_TYPES {
                let piece = Piece::new(color, piece_type);
                assert_eq!(piece.color(), color);
                assert_eq!(piece.piece_type(), piece_type);
            }
        }
    }

    #[test]
    fn piece_convenience_constructors() {
        for &piece_type in ALL_PIECE_TYPES {
            let white_piece = Piece::white(piece_type);
            assert_eq!(white_piece.color(), Color::White);
            let black_piece = Piece::black(piece_type);
            assert_eq!(black_piece.color(), Color::Black);
        }
    }

    #[test]
    fn piece_try_from_u8() {
        for garbage_upper in 0..16 {
            for &(repr, piece) in PIECES_BY_REPRESENTATIONS {
                let repr = garbage_upper << 4 | repr;
                assert_eq!(Piece::try_from_u8(repr), piece);
            }
        }
    }

    #[test]
    fn piece_is_white_is_black() {
        assert!(Piece::WHITE_BISHOP.is_white());
        assert!(Piece::WHITE_QUEEN.is_white());
        assert!(Piece::BLACK_PAWN.is_black());
        assert!(Piece::BLACK_KING.is_black());
    }

    #[test]
    fn piece_get_repr() {
        for &(repr, piece) in PIECES_BY_REPRESENTATIONS {
            if let Some(piece) = piece {
                assert_eq!(piece.get(), repr);
                assert_eq!(piece.get_constrained().get(), repr);
            }
        }
    }

    #[test]
    fn piece_as_fen_char() {
        let cases = [
            (Piece::WHITE_PAWN, 'P'),
            (Piece::WHITE_BISHOP, 'B'),
            (Piece::WHITE_KING, 'K'),
            (Piece::BLACK_KNIGHT, 'n'),
            (Piece::BLACK_ROOK, 'r'),
            (Piece::BLACK_QUEEN, 'q'),
        ];
        for (piece, expected) in cases {
            assert_eq!(piece.as_fen_char(), expected);
        }
    }

    #[test]
    fn piece_as_unicode_char() {
        let cases = [
            (Piece::WHITE_PAWN, '♙'),
            (Piece::WHITE_BISHOP, '♗'),
            (Piece::WHITE_KING, '♔'),
            (Piece::BLACK_KNIGHT, '♞'),
            (Piece::BLACK_ROOK, '♜'),
            (Piece::BLACK_QUEEN, '♛'),
        ];
        for (piece, expected) in cases {
            assert_eq!(piece.as_unicode_char(), expected);
        }
    }

    #[test]
    fn piece_try_from_fen_char() {
        let cases = [
            ('P', Piece::WHITE_PAWN),
            ('B', Piece::WHITE_BISHOP),
            ('K', Piece::WHITE_KING),
            ('n', Piece::BLACK_KNIGHT),
            ('r', Piece::BLACK_ROOK),
            ('q', Piece::BLACK_QUEEN),
        ];
        for (piece, expected) in cases {
            assert_eq!(Piece::try_from_fen_char(piece), Some(expected));
        }

        assert_eq!(Piece::try_from_fen_char('X'), None);
        assert_eq!(Piece::try_from_fen_char('a'), None);
    }

    #[test]
    fn piece_is_slider() {
        for piece_type in SLIDERS {
            assert!(piece_type.white().is_slider());
            assert!(piece_type.black().is_slider());
        }
        for piece_type in NON_SLIDERS {
            assert!(!piece_type.white().is_slider());
            assert!(!piece_type.black().is_slider());
        }
    }

    #[test]
    fn piece_display() {
        for &color in ALL_COLORS {
            for &piece_type in ALL_PIECE_TYPES {
                let piece = Piece::new(color, piece_type);
                assert_eq!(piece.as_fen_char().to_string(), format!("{piece}"));
            }
        }
    }

    #[test]
    fn piece_from_str() {
        assert_eq!("K".parse(), Ok(Piece::WHITE_KING));
        assert_eq!("n".parse(), Ok(Piece::BLACK_KNIGHT));
        assert_eq!("b".parse(), Ok(Piece::BLACK_BISHOP));
        assert_eq!("".parse::<Piece>(), Err(()));
        assert_eq!("a".parse::<Piece>(), Err(()));
        assert_eq!("KK".parse::<Piece>(), Err(()));
    }
}
