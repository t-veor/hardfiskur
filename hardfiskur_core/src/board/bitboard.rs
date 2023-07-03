use std::fmt::Debug;

use super::Square;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(u64::MAX);

    pub const RANK_1: Self = Self(0x00000000000000FF);
    pub const RANK_4: Self = Self(0x00000000FF000000);
    pub const RANK_5: Self = Self(0x000000FF00000000);
    pub const RANK_8: Self = Self(0xFF00000000000000);

    pub const A_FILE: Self = Self(0x0101010101010101);
    pub const B_FILE: Self = Self(0x0202020202020202);
    pub const G_FILE: Self = Self(0x4040404040404040);
    pub const H_FILE: Self = Self(0x8080808080808080);

    pub const fn has_piece(self) -> bool {
        self.0 != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn implies(self, other: Self) -> Self {
        Self(!self.0 | other.0)
    }

    pub const fn equivalent(self, other: Self) -> Self {
        Self(!(self.0 ^ other.0))
    }

    pub const fn step_south(self) -> Self {
        Self(self.0 >> 8)
    }

    pub const fn step_north(self) -> Self {
        Self(self.0 << 8)
    }

    pub const fn step_east(self) -> Self {
        Self((self.0 << 1) & !Self::A_FILE.0)
    }

    pub const fn step_north_east(self) -> Self {
        Self((self.0 << 9) & !Self::A_FILE.0)
    }

    pub const fn step_south_east(self) -> Self {
        Self((self.0 >> 7) & !Self::A_FILE.0)
    }

    pub const fn step_west(self) -> Self {
        Self((self.0 >> 1) & !Self::H_FILE.0)
    }

    pub const fn step_south_west(self) -> Self {
        Self((self.0 >> 9) & !Self::H_FILE.0)
    }

    pub const fn step_north_west(self) -> Self {
        Self((self.0 << 7) & !Self::H_FILE.0)
    }

    pub const fn pop_count(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn flip_vertical(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Returns 255 if self.is_empty()
    pub const fn msb(self) -> u8 {
        63u8.wrapping_sub(self.0.leading_zeros() as _)
    }

    /// Returns 64 if self.is_empty()
    pub const fn lsb(self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    /// Assumes that pop_count is 1!
    pub const fn to_square(self) -> Square {
        Square::from_index_unchecked(self.0.trailing_zeros() as _)
    }

    pub const fn from_index(index: u8) -> Self {
        Self(1 << index)
    }

    pub const fn from_square(square: Square) -> Self {
        Self(1 << square.get())
    }

    pub const fn get(&self, square: Square) -> bool {
        self.0 & (1 << square.get()) > 0
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square.get();
    }

    pub fn toggle(&mut self, square: Square) {
        self.0 ^= 1 << square.get();
    }

    pub fn reset(&mut self, square: Square) {
        let single_bit = 1 << square.get();
        self.0 |= single_bit;
        self.0 ^= single_bit;
    }

    pub fn bits(&self) -> impl Iterator<Item = u8> {
        BitIterator(self.0)
    }
}

impl Debug for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("Bitboard(\n")?;
        for rank in (0..8).rev() {
            f.write_str("    ")?;
            for file in 0..8 {
                let square = Square::new_unchecked(rank, file);
                f.write_str(" ")?;
                f.write_str(if self.get(square) { "#" } else { "." })?;
            }
            f.write_str("\n")?;
        }
        f.write_str(")")
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::ops::Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self {
        Self(self.0 << rhs)
    }
}

impl std::ops::ShlAssign<u8> for Bitboard {
    fn shl_assign(&mut self, rhs: u8) {
        *self = *self << rhs
    }
}

impl std::ops::Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self {
        Self(self.0 >> rhs)
    }
}

impl std::ops::ShrAssign<u8> for Bitboard {
    fn shr_assign(&mut self, rhs: u8) {
        *self = *self >> rhs
    }
}

pub struct BitIterator(u64);

impl Iterator for BitIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            let t = self.0 & self.0.wrapping_neg();
            let r = self.0.trailing_zeros();
            self.0 ^= t;
            Some(r as _)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(64))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn b(rank: u8, file: u8) -> Bitboard {
        Bitboard::from_square(Square::new_unchecked(rank, file))
    }

    #[test]
    fn test_bitboard_single_step() {
        let board = b(3, 3);
        assert_eq!(board.step_north(), b(4, 3));
        assert_eq!(board.step_north_east(), b(4, 4));
        assert_eq!(board.step_east(), b(3, 4));
        assert_eq!(board.step_south_east(), b(2, 4));
        assert_eq!(board.step_south(), b(2, 3));
        assert_eq!(board.step_south_west(), b(2, 2));
        assert_eq!(board.step_west(), b(3, 2));
        assert_eq!(board.step_north_west(), b(4, 2));
    }

    #[test]
    fn test_edge_step() {
        let north_west_corner = b(7, 0);
        assert_eq!(north_west_corner.step_north(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_north_east(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_east(), b(7, 1));
        assert_eq!(north_west_corner.step_south_east(), b(6, 1));
        assert_eq!(north_west_corner.step_south(), b(6, 0));
        assert_eq!(north_west_corner.step_south_west(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_west(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_north_west(), Bitboard::EMPTY);

        let south_east_corner = b(0, 7);
        assert_eq!(south_east_corner.step_north(), b(1, 7));
        assert_eq!(south_east_corner.step_north_east(), Bitboard::EMPTY);
        assert_eq!(south_east_corner.step_east(), Bitboard::EMPTY);
        assert_eq!(south_east_corner.step_south_east(), Bitboard::EMPTY);
        assert_eq!(south_east_corner.step_south(), Bitboard::EMPTY);
        assert_eq!(south_east_corner.step_south_west(), Bitboard::EMPTY);
        assert_eq!(south_east_corner.step_west(), b(0, 6));
        assert_eq!(south_east_corner.step_north_west(), b(1, 6));
    }

    #[test]
    fn test_step_multiple_bits() {
        let north_west_corner = b(7, 0);
        let south_east_corner = b(0, 7);
        let center = b(3, 3);
        let board = north_west_corner | south_east_corner | center;

        assert_eq!(board.step_north(), b(4, 3) | b(1, 7));
        assert_eq!(board.step_north_east(), b(4, 4));
        assert_eq!(board.step_east(), b(3, 4) | b(7, 1));
        assert_eq!(board.step_south_east(), b(2, 4) | b(6, 1));
        assert_eq!(board.step_south(), b(2, 3) | b(6, 0));
        assert_eq!(board.step_south_west(), b(2, 2));
        assert_eq!(board.step_west(), b(3, 2) | b(0, 6));
        assert_eq!(board.step_north_west(), b(4, 2) | b(1, 6));
    }
}
