use std::{
    fmt::{Debug, Display, Write},
    num::NonZeroU8,
};

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum PieceType {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl PieceType {
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

    pub const fn is_slider(self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 8,
}

impl Color {
    pub const fn is_white(self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false,
        }
    }

    pub const fn is_black(self) -> bool {
        !self.is_white()
    }

    pub const fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::White
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Piece(NonZeroU8);

impl Piece {
    pub const fn new(color: Color, piece_type: PieceType) -> Self {
        // Safety: piece_type as u8 can never be 0
        unsafe { Self(NonZeroU8::new_unchecked(color as u8 | piece_type as u8)) }
    }

    pub const fn white(piece_type: PieceType) -> Self {
        Self::new(Color::White, piece_type)
    }

    pub const fn black(piece_type: PieceType) -> Self {
        Self::new(Color::Black, piece_type)
    }

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

    pub const fn color(self) -> Color {
        if self.0.get() & 8 > 0 {
            Color::Black
        } else {
            Color::White
        }
    }

    pub const fn is_white(self) -> bool {
        self.color().is_white()
    }

    pub const fn is_black(self) -> bool {
        self.color().is_black()
    }

    pub const fn piece_type(self) -> PieceType {
        match self.0.get() & 7 {
            1 => PieceType::Pawn,
            2 => PieceType::Knight,
            3 => PieceType::Bishop,
            4 => PieceType::Rook,
            5 => PieceType::Queen,
            6 => PieceType::King,
            _ => unreachable!(),
        }
    }

    pub const fn get(self) -> u8 {
        self.0.get()
    }

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

    pub fn unicode_char(self) -> char {
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

    pub const fn try_from_char(c: char) -> Option<Self> {
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
        f.write_char(if self.is_white() {
            self.piece_type().as_uppercase_char()
        } else {
            self.piece_type().as_lowercase_char()
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

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

    #[test]
    fn test_piece_type_as_uppercase() {
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
    fn test_piece_type_as_lowercase() {
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
    fn test_piece_type_is_slider() {
        for piece_type in SLIDERS {
            assert!(piece_type.is_slider())
        }
        for piece_type in NON_SLIDERS {
            assert!(!piece_type.is_slider())
        }
    }

    #[test]
    fn test_color_is_white_and_is_black() {
        assert!(Color::White.is_white());
        assert!(!Color::Black.is_white());

        assert!(!Color::White.is_black());
        assert!(Color::Black.is_black());
    }

    #[test]
    fn test_color_flip() {
        assert_eq!(Color::White.flip(), Color::Black);
        assert_eq!(Color::Black.flip(), Color::White);
    }

    #[test]
    fn test_piece_create_and_unpack() {
        for &color in ALL_COLORS {
            for &piece_type in ALL_PIECE_TYPES {
                let piece = Piece::new(color, piece_type);
                assert_eq!(piece.color(), color);
                assert_eq!(piece.piece_type(), piece_type);
            }
        }
    }

    #[test]
    fn test_piece_as_fen_char() {
        let cases = [
            (Piece::new(Color::White, PieceType::Pawn), 'P'),
            (Piece::new(Color::White, PieceType::Bishop), 'B'),
            (Piece::new(Color::White, PieceType::King), 'K'),
            (Piece::new(Color::Black, PieceType::Knight), 'n'),
            (Piece::new(Color::Black, PieceType::Rook), 'r'),
            (Piece::new(Color::Black, PieceType::Queen), 'q'),
        ];
        for (piece, expected) in cases {
            assert_eq!(piece.as_fen_char(), expected);
        }
    }
}
