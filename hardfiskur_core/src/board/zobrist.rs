use std::{
    fmt::Debug,
    ops::{BitXor, BitXorAssign},
    sync::OnceLock,
};

use rand::{RngCore, SeedableRng};
use zerocopy_derive::FromZeros;

use super::{Castling, Color, Piece, Square};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, FromZeros)]
pub struct ZobristHash(pub u64);

impl ZobristHash {
    pub fn piece(piece: Piece, square: Square) -> Self {
        // TODO: remove bounds check?
        let instance = ZobristTable::get_instance();
        let index = piece.get() as usize;
        Self(instance.pieces[index * 64 + square.index()])
    }

    pub fn color(color: Color) -> Self {
        let instance = ZobristTable::get_instance();
        match color {
            Color::White => Self(0),
            Color::Black => Self(instance.black),
        }
    }

    pub fn castling(castling: Castling) -> Self {
        let instance = ZobristTable::get_instance();
        let index = castling.bits() as usize;
        Self(instance.castling[index])
    }

    pub fn en_passant(en_passant: Option<Square>) -> Self {
        let instance = ZobristTable::get_instance();
        match en_passant {
            Some(square) => {
                let index = square.file() as usize;
                Self(instance.en_passant[index])
            }
            None => Self(0),
        }
    }

    pub fn toggle_piece(&mut self, piece: Piece, square: Square) {
        *self ^= Self::piece(piece, square)
    }
}

impl BitXor for ZobristHash {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for ZobristHash {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Debug for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ZobristHash")
            .field(&format_args!("{:#016X}", self.0))
            .finish()
    }
}

struct ZobristTable {
    pieces: [u64; 16 * 64], // pieces fit in a 4-bit integer
    black: u64,
    castling: [u64; 16],
    en_passant: [u64; 8],
}

impl ZobristTable {
    fn new() -> Self {
        let mut rng = rand_chacha::ChaCha12Rng::from_seed([
            0x94, 0xaa, 0x13, 0x7c, 0xe3, 0x62, 0xaf, 0x0d, 0x3f, 0xb2, 0x3b, 0xba, 0x78, 0xe2,
            0x21, 0x18, 0xf0, 0xc3, 0xbd, 0xb3, 0x59, 0xac, 0x84, 0x13, 0x17, 0x58, 0x01, 0x54,
            0x54, 0x72, 0xd2, 0xc3,
        ]);

        let mut pieces = [0; 16 * 64];
        pieces.fill_with(|| rng.next_u64());

        let black = rng.next_u64();

        let mut castling = [0; 16];
        castling.fill_with(|| rng.next_u64());

        let mut en_passant = [0; 8];
        en_passant.fill_with(|| rng.next_u64());

        Self {
            pieces,
            black,
            castling,
            en_passant,
        }
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: OnceLock<ZobristTable> = OnceLock::new();

        INSTANCE.get_or_init(Self::new)
    }
}
