use std::{fmt::Debug, str::FromStr};

use super::Square;

/// Compact data structure representing some board state. See
/// <https://www.chessprogramming.org/Bitboards>
///
/// Methods assume that the least significant bit is a1, the 8th least
/// significant bit is a8, and the most significant bit is h8.
///
/// The underlying [`u64`] of a bitboard can simply be accessed via `.0`.
///
/// Bitboards have all the bitwise operations defined on them, e.g. `&`, `|`,
/// `^`, and `!`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// Bitboard with no bits set.
    pub const EMPTY: Self = Self(0);
    /// Bitboard with every bit set.
    pub const ALL: Self = Self(u64::MAX);

    /// Bitboard with only squares on the 1st rank set.
    pub const RANK_1: Self = Self(0x00000000000000FF);
    /// Bitboard with only squares on the 4th rank set.
    pub const RANK_4: Self = Self(0x00000000FF000000);
    /// Bitboard with only squares on the 5th rank set.
    pub const RANK_5: Self = Self(0x000000FF00000000);
    /// Bitboard with only squares on the 8th rank set.
    pub const RANK_8: Self = Self(0xFF00000000000000);

    /// Bitboard with only squares on the A file set.
    pub const A_FILE: Self = Self(0x0101010101010101);
    /// Bitboard with only squares on the B file set.
    pub const B_FILE: Self = Self(0x0202020202020202);
    /// Bitboard with only squares on the G file set.
    pub const G_FILE: Self = Self(0x4040404040404040);
    /// Bitboard with only squares on the H file set.
    pub const H_FILE: Self = Self(0x8080808080808080);

    /// Returns whether this bitboard contains anything, i.e. if it is not equal
    /// to 0.
    pub const fn has_piece(self) -> bool {
        self.0 != 0
    }

    /// Returns whether this bitboard is empty, i.e. if it is equal to 0.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the bitwise NOT of this bitboard.
    ///
    /// Equivalent to `!self`, but this method may be useful in const contexts
    /// since const traits have not been stabilised.
    pub const fn not(self) -> Self {
        Self(!self.0)
    }

    /// Returns the bitwise OR of this bitboard with `other`..
    ///
    /// Equivalent to `self | other`, but this method may be useful in const
    /// contexts since const traits have not been stabilised.
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the bitwise AND of this bitboard with `other`.
    ///
    /// Equivalent to `self & other`, but this method may be useful in const
    /// contexts since const traits have not been stabilised.
    pub const fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Returns the bitwise XOR of this bitboard with `other`.
    ///
    /// Equivalent to `self ^ other`, but this method may be useful in const
    /// contexts since const traits have not been stabilised.
    pub const fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    /// Returns a copy of this bitboard with any bits in the other bitboard set
    /// to 0.
    ///
    /// Equivalent to `self & !other`.
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Returns a bitboard with bits set where the bit in `self` is off, or both
    /// the bit in `self` and `other` are on.
    ///
    /// Equivalent to `!self | other`.
    pub const fn implies(self, other: Self) -> Self {
        Self(!self.0 | other.0)
    }

    /// Returns a bitboard with bits set where bits in `self` and `other` are
    /// equal.
    ///
    /// Equivalent to `!(self ^ other)`.
    pub const fn equivalent(self, other: Self) -> Self {
        Self(!(self.0 ^ other.0))
    }

    /// Moves all bits in this bitboard down a rank.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_south(self) -> Self {
        Self(self.0 >> 8)
    }

    /// Moves all bits in this bitboard up a rank.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_north(self) -> Self {
        Self(self.0 << 8)
    }

    /// Moves all bits in this bitboard towards the A file.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_east(self) -> Self {
        Self((self.0 << 1) & !Self::A_FILE.0)
    }

    /// Moves all bits in this bitboard up a rank and towards the A file.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_north_east(self) -> Self {
        Self((self.0 << 9) & !Self::A_FILE.0)
    }

    /// Moves all bits in this bitboard down a rank and towards the A file.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_south_east(self) -> Self {
        Self((self.0 >> 7) & !Self::A_FILE.0)
    }

    /// Moves all bits in this bitboard towards the H file.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_west(self) -> Self {
        Self((self.0 >> 1) & !Self::H_FILE.0)
    }

    /// Moves all bits in this bitboard down a rank and towards the H file.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_south_west(self) -> Self {
        Self((self.0 >> 9) & !Self::H_FILE.0)
    }

    /// Moves all bits in this bitboard up a rank and towards the H file.
    ///
    /// Bits that move off the edge of the board are lost.
    pub const fn step_north_west(self) -> Self {
        Self((self.0 << 7) & !Self::H_FILE.0)
    }

    /// Counts the number of bits set in this bitboard.
    pub const fn pop_count(self) -> u32 {
        self.0.count_ones()
    }

    /// Mirrors this bitboard vertically, so that 1st rank becomes the 8th rank
    /// and vice versa. Files are preserved.
    pub const fn flip_vertical(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Returns the position of the most significant bit that is set.
    ///
    /// If this bitboard is empty, returns [`None`].
    pub const fn msb(self) -> Option<u8> {
        match self.0 {
            0 => None,
            x => Some(63 - x.leading_zeros() as u8),
        }
    }

    /// Returns the position of the least significant bit that is set.
    ///
    /// If this bitboard is empty, returns [`None`].
    pub const fn lsb(self) -> Option<u8> {
        match self.0 {
            0 => None,
            x => Some(x.trailing_zeros() as u8),
        }
    }

    /// Converts a bitboard with a single bit set to a [`Square`] representing
    /// the set bit.
    ///
    /// If this bitboard is empty, returns [`None`].
    ///
    /// # Edge cases
    ///
    /// You should ideally only call this method on bitboards whose
    /// `.pop_count()` is 1. However, for performance reasons this method does
    /// not check this (except to check if the bitboard is empty), and instead
    /// simply returns the [`Square`] corresponding to the least significant bit
    /// that is set.
    pub const fn to_square(self) -> Option<Square> {
        // Have to write it this way because .map is not allowed in a const fn
        match self.lsb() {
            Some(x) => Some(Square::from_u8_unchecked(x)),
            None => None,
        }
    }

    /// Returns a bitboard with only the bit at the provided index set.
    ///
    /// Providing an index larger than 63 may cause an overflow panic.
    pub const fn from_index(index: usize) -> Self {
        Self(1 << index)
    }

    /// Returns a bitboard with only the bit at the provided index set.
    ///
    /// Providing an index larger than 63 may cause an overflow panic.
    pub const fn from_u8(index: u8) -> Self {
        Self(1 << index)
    }

    /// Returns a bitboard with only the bit at the provided square set.
    pub const fn from_square(square: Square) -> Self {
        Self(1 << square.get())
    }

    /// Returns whether the bit at the given square is set.
    pub const fn get(&self, square: Square) -> bool {
        self.0 & (1 << square.get()) > 0
    }

    /// Set the bit at the given square to 1.
    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square.get();
    }

    /// Toggle the bit at the given square.
    pub fn toggle(&mut self, square: Square) {
        self.0 ^= 1 << square.get();
    }

    /// Reset the bit at the given square to 0.
    pub fn reset(&mut self, square: Square) {
        let single_bit = 1 << square.get();
        self.0 |= single_bit;
        self.0 ^= single_bit;
    }

    /// Returns an [`Iterator<Item = u8>`] where the items are the indices of
    /// bits which are set, in ascending order.
    pub fn bits(&self) -> impl Iterator<Item = u8> {
        BitIterator(self.0)
    }

    /// Returns an [`Iterator<Item = Square>`] where the items are the
    /// [`Square`]s corresponding to set bits, in ascending order.
    pub fn set_squares(&self) -> impl Iterator<Item = Square> {
        BitIterator(self.0).map(Square::from_u8_unchecked)
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

impl FromStr for Bitboard {
    type Err = ();

    /// Parses a string into a bitboard. This is intended to make specifying
    /// bitboards easier for tests.
    ///
    /// Bitboards should look something like the following:
    /// ```txt
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . # . . .
    /// . . . . . . . .
    /// # # # # . # # #
    /// . . . . . . . .
    /// ```
    ///
    /// Characters are interpreted as squares starting from the top left (a8),
    /// by file and then by rank.
    ///
    /// The parsing is very permissive:
    /// * All whitespace is ignored.
    /// * `.` and `0` are interpreted as 0.
    /// * Any other character encountered is interpreted as 1.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut square_iter = (0..8)
            .rev()
            .flat_map(|rank| (0..8).map(move |file| Square::new_unchecked(rank, file)));

        let mut bitboard = Bitboard::EMPTY;
        for c in s.chars() {
            match c {
                '.' | '0' => {
                    let _ = square_iter.next().ok_or(())?;
                }
                c if c.is_whitespace() => (),
                _ => {
                    let square = square_iter.next().ok_or(())?;
                    bitboard.set(square);
                }
            }
        }

        if square_iter.next().is_some() {
            return Err(());
        }

        Ok(bitboard)
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.and(rhs)
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
        self.or(rhs)
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
        self.xor(rhs)
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
        self.not()
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

struct BitIterator(u64);

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
    fn test_bitboard_boolean_operators() {
        let a = 0b10010111;
        let b = 0b10101100;

        assert_eq!(Bitboard(a).not(), Bitboard(!a));
        assert_eq!(Bitboard(a).and(Bitboard(b)), Bitboard(a & b));
        assert_eq!(Bitboard(a).or(Bitboard(b)), Bitboard(a | b));
        assert_eq!(Bitboard(a).xor(Bitboard(b)), Bitboard(a ^ b));

        assert_eq!(Bitboard(a).without(Bitboard(b)), Bitboard(a & !b));
        assert_eq!(Bitboard(a).implies(Bitboard(b)), Bitboard(!a | b));
        assert_eq!(Bitboard(a).equivalent(Bitboard(b)), Bitboard(!(a ^ b)));
    }

    #[test]
    fn test_bitboard_bitwise_operators() {
        let a = 0b10010111;
        let b = 0b10101100;

        assert_eq!(!Bitboard(a), Bitboard(!a));
        assert_eq!(Bitboard(a) & Bitboard(b), Bitboard(a & b));
        assert_eq!(Bitboard(a) | Bitboard(b), Bitboard(a | b));
        assert_eq!(Bitboard(a) ^ Bitboard(b), Bitboard(a ^ b));

        let mut c = Bitboard(a);
        c &= Bitboard(b);
        assert_eq!(c, Bitboard(a & b));

        c = Bitboard(a);
        c |= Bitboard(b);
        assert_eq!(c, Bitboard(a | b));

        c = Bitboard(a);
        c ^= Bitboard(b);
        assert_eq!(c, Bitboard(a ^ b));
    }

    #[test]
    fn test_bitboard_shift_operators() {
        let a = 0b10111111;

        assert_eq!(Bitboard(a) << 5, Bitboard(a << 5));
        assert_eq!(Bitboard(a) >> 4, Bitboard(a >> 4));

        let mut b = Bitboard(a);
        b <<= 6;
        assert_eq!(b, Bitboard(a << 6));

        b = Bitboard(a);
        b >>= 2;
        assert_eq!(b, Bitboard(a >> 2));
    }

    #[test]
    fn test_bitboard_has_piece_is_empty() {
        assert!(Bitboard::EMPTY.is_empty());
        assert!(!Bitboard::EMPTY.has_piece());

        assert!(!Bitboard(12).is_empty());
        assert!(Bitboard(12).has_piece());
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
    fn test_bitboad_edge_step() {
        let north_west_corner = b(7, 0);
        assert_eq!(north_west_corner.step_north(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_north_east(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_east(), b(7, 1));
        assert_eq!(north_west_corner.step_south_east(), b(6, 1));
        assert_eq!(north_west_corner.step_south(), b(6, 0));
        assert_eq!(north_west_corner.step_south_west(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_west(), Bitboard::EMPTY);
        assert_eq!(north_west_corner.step_north_west(), Bitboard::EMPTY);

        let north_east_corner = b(7, 7);
        assert_eq!(north_east_corner.step_north(), Bitboard::EMPTY);
        assert_eq!(north_east_corner.step_north_east(), Bitboard::EMPTY);
        assert_eq!(north_east_corner.step_east(), Bitboard::EMPTY);
        assert_eq!(north_east_corner.step_south_east(), Bitboard::EMPTY);
        assert_eq!(north_east_corner.step_south(), b(6, 7));
        assert_eq!(north_east_corner.step_south_west(), b(6, 6));
        assert_eq!(north_east_corner.step_west(), b(7, 6));
        assert_eq!(north_east_corner.step_north_west(), Bitboard::EMPTY);

        let south_west_corner = b(0, 0);
        assert_eq!(south_west_corner.step_north(), b(1, 0));
        assert_eq!(south_west_corner.step_north_east(), b(1, 1));
        assert_eq!(south_west_corner.step_east(), b(0, 1));
        assert_eq!(south_west_corner.step_south_east(), Bitboard::EMPTY);
        assert_eq!(south_west_corner.step_south(), Bitboard::EMPTY);
        assert_eq!(south_west_corner.step_south_west(), Bitboard::EMPTY);
        assert_eq!(south_west_corner.step_west(), Bitboard::EMPTY);
        assert_eq!(south_west_corner.step_north_west(), Bitboard::EMPTY);

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
    fn test_bitboard_step_multiple_bits() {
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

    #[test]
    fn test_bitboard_pop_count() {
        assert_eq!(Bitboard(0).pop_count(), 0);
        assert_eq!(Bitboard(0b10).pop_count(), 1);
        assert_eq!(Bitboard(0b11010100).pop_count(), 4);
        assert_eq!(Bitboard::RANK_8.pop_count(), 8);
        assert_eq!(Bitboard::ALL.pop_count(), 64);
    }

    #[test]
    fn test_bitboard_flip_vertical() {
        assert_eq!(
            Bitboard(0x0A0B0C0_D0E0F1011).flip_vertical(),
            Bitboard(0x11100F0_E0D0C0B0A)
        )
    }

    #[test]
    fn test_bitboard_msb_lsb() {
        let b = Bitboard(0x0FFF0003_8A200000);
        assert_eq!(b.msb(), Some(59));
        assert_eq!(b.lsb(), Some(21));

        assert_eq!(Bitboard::EMPTY.msb(), None);
        assert_eq!(Bitboard::EMPTY.lsb(), None);
    }

    #[test]
    fn test_bitboard_to_square() {
        assert_eq!(b(3, 7).to_square(), Square::new(3, 7));
        assert_eq!(b(1, 4).to_square(), Square::new(1, 4));
        assert_eq!(b(2, 5).to_square(), Square::new(2, 5));
        assert_eq!(b(0, 6).to_square(), Square::new(0, 6));

        assert_eq!(Bitboard::EMPTY.to_square(), None);
        assert_eq!((b(3, 7) | b(1, 4)).to_square(), Square::new(1, 4));
    }

    #[test]
    fn test_bitboard_from_numeric() {
        assert_eq!(Bitboard::from_index(0), b(0, 0));
        assert_eq!(Bitboard::from_u8(0), b(0, 0));

        assert_eq!(Bitboard::from_index(13), b(1, 5));
        assert_eq!(Bitboard::from_u8(13), b(1, 5));

        assert_eq!(Bitboard::from_index(59), b(7, 3));
        assert_eq!(Bitboard::from_u8(59), b(7, 3));
    }

    #[test]
    fn test_bitboard_get() {
        let x = b(1, 3) | b(5, 6) | b(3, 7);

        assert!(x.get(Square::new_unchecked(1, 3)));
        assert!(x.get(Square::new_unchecked(5, 6)));
        assert!(x.get(Square::new_unchecked(3, 7)));
        assert!(!x.get(Square::new_unchecked(2, 0)));
        assert!(!x.get(Square::new_unchecked(5, 1)));
        assert!(!x.get(Square::new_unchecked(4, 4)));
    }

    #[test]
    fn test_bitboard_set() {
        let mut x = Bitboard::EMPTY;

        x.set(Square::new_unchecked(3, 3));
        assert_eq!(x, b(3, 3));

        x.set(Square::new_unchecked(5, 2));
        assert_eq!(x, b(3, 3) | b(5, 2));

        x.set(Square::new_unchecked(3, 3));
        assert_eq!(x, b(3, 3) | b(5, 2));
    }

    #[test]
    fn test_bitboard_toggle() {
        let mut x = Bitboard::EMPTY;

        x.toggle(Square::new_unchecked(3, 3));
        assert_eq!(x, b(3, 3));

        x.toggle(Square::new_unchecked(5, 2));
        assert_eq!(x, b(3, 3) | b(5, 2));

        x.toggle(Square::new_unchecked(3, 3));
        assert_eq!(x, b(5, 2));

        x.toggle(Square::new_unchecked(5, 2));
        assert_eq!(x, Bitboard::EMPTY);
    }

    #[test]
    fn test_bitboard_reset() {
        let mut x = b(1, 2) | b(5, 5);

        x.reset(Square::new_unchecked(1, 2));
        assert_eq!(x, b(5, 5));

        x.reset(Square::new_unchecked(1, 2));
        assert_eq!(x, b(5, 5));

        x.reset(Square::new_unchecked(5, 5));
        assert_eq!(x, Bitboard::EMPTY);

        x.reset(Square::new_unchecked(7, 3));
        assert_eq!(x, Bitboard::EMPTY);
    }

    #[test]
    fn test_bitboard_bits() {
        assert_eq!(Bitboard::EMPTY.bits().collect::<Vec<_>>(), vec![]);

        assert_eq!(
            Bitboard::ALL.bits().collect::<Vec<_>>(),
            (0..64).collect::<Vec<_>>()
        );

        assert_eq!(
            Bitboard(0b10001011_00111100).bits().collect::<Vec<_>>(),
            vec![2, 3, 4, 5, 8, 9, 11, 15]
        );
    }

    #[test]
    fn test_bitboard_set_squares() {
        assert_eq!(Bitboard::EMPTY.set_squares().collect::<Vec<_>>(), vec![]);

        assert_eq!(
            Bitboard::ALL.set_squares().collect::<Vec<_>>(),
            (0..64)
                .map(Square::from_index_unchecked)
                .collect::<Vec<_>>()
        );

        assert_eq!(
            Bitboard(0b10001011_00111100)
                .set_squares()
                .collect::<Vec<_>>(),
            vec![2, 3, 4, 5, 8, 9, 11, 15]
                .into_iter()
                .map(Square::from_index_unchecked)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_bitboard_from_str() {
        assert_eq!(
            Bitboard::from_str(
                "
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . # . . .
            . . . . . . . .
            # # # # . # # #
            . . . . . . . .
                "
            ),
            Ok(Bitboard(0x00000000_1000EF00))
        );

        assert_eq!(
            Bitboard::from_str(&std::iter::repeat('0').take(64).collect::<String>()),
            Ok(Bitboard::EMPTY)
        );

        assert_eq!(
            Bitboard::from_str(
                "
12345678
........
00..00..
xxxxxxxx
abc00def
....    ..1.
..  ..  ..  ..
1  .  .  .  .  .  .  ."
            ),
            Ok(Bitboard(0xFF0000FF_E7400001))
        );

        assert!(Bitboard::from_str("").is_err());
        assert!(Bitboard::from_str("0000001000000").is_err());
        assert!(Bitboard::from_str(&std::iter::repeat('0').take(65).collect::<String>()).is_err());
    }
}
