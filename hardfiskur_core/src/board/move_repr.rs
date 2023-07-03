use bitflags::bitflags;

use super::{Piece, Square};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MoveFlags: u32 {
        const DOUBLE_PAWN_PUSH = 0b0001 << 28;
        const CASTLE           = 0b0010 << 28;
        const EN_PASSANT       = 0b0100 << 28;
    }
}

/// Move data, encoded as a 32-bit integer.
/// 0XXX_PPPP CCCC_MMMM 00TTTTTT 00FFFFFF
/// ^^^^ ^^^^ ^^^^ ^^^^   ^^^^^^   ^^^^^^
///    |    |    |    |        |        |
///    |    |    |    |        |        +-- from square
///    |    |    |    |        +----------- to square
///    |    |    |    +-------------------- moved piece
///    |    |    +------------------------- captured piece (0 if none)
///    |    +------------------------------ promoted to piece (0 if none)
///    +----------------------------------- move flags
#[derive(Clone, Copy)]
pub struct Move(u32);

impl Move {
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

    pub const fn new_unchecked(inner: u32) -> Self {
        Self(inner)
    }

    pub const fn from_square(self) -> Square {
        Square::from_u8_unchecked((self.0 & 0x3F) as u8)
    }

    pub const fn to_square(self) -> Square {
        Square::from_u8_unchecked(((self.0 & 0x3F00) >> 8) as u8)
    }

    // Would really like this to be a const function, but alas
    pub fn piece(self) -> Piece {
        Piece::try_from_u8(((self.0 & 0x0F0000) >> 16) as u8)
            .expect("invalid move representation encountered")
    }

    pub const fn captured_piece(self) -> Option<Piece> {
        Piece::try_from_u8(((self.0 & 0xF00000) >> 20) as u8)
    }

    pub const fn promotion(self) -> Option<Piece> {
        Piece::try_from_u8(((self.0 & 0x0F000000) > 24) as u8)
    }

    pub const fn flags(self) -> MoveFlags {
        MoveFlags::from_bits_truncate(self.0)
    }

    pub const fn is_capture(self) -> bool {
        self.captured_piece().is_some()
    }

    pub const fn is_double_pawn_push(self) -> bool {
        MoveFlags::from_bits_retain(self.0).contains(MoveFlags::DOUBLE_PAWN_PUSH)
    }

    pub const fn is_castle(self) -> bool {
        MoveFlags::from_bits_retain(self.0).contains(MoveFlags::CASTLE)
    }

    pub const fn is_en_passant(self) -> bool {
        MoveFlags::from_bits_retain(self.0).contains(MoveFlags::EN_PASSANT)
    }
}
