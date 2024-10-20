use std::{fmt::Debug, str::FromStr};

use paste::paste;
use seq_macro::seq;

use super::Square;

/// Compact data structure representing some board state. See
/// <https://www.chessprogramming.org/Bitboards>
///
/// Methods assume that the least significant bit is a1 and the most significant
/// bit is h8, and squares are indexed by increasing file and then rank (i.e.
/// index 0 is a1, index 1 is b1, index 2 is c1... index 7 is h1, index 8 is a2,
/// index 9 is b2, etc.).
///
/// The underlying [`u64`] of a bitboard can simply be accessed via `.0`.
///
/// Bitboards have all the bitwise operations defined on them, e.g. `&`, `|`,
/// `^`, and `!`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// Bitboard with no bits set.
    pub const EMPTY: Self = Self(0);
    /// Bitboard with every bit set.
    pub const ALL: Self = Self(u64::MAX);

    /// Returns a bitboard with all of the bits in the given rank set.
    pub const fn rank_mask(rank: u8) -> Self {
        Self(0x00000000000000FF << (rank * 8))
    }

    /// Returns a bitboard with all of the bits in the given file set.
    pub const fn file_mask(file: u8) -> Self {
        Self(0x0101010101010101 << file)
    }

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
    #[inline]
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

    pub const fn isolate_lsb(self) -> Bitboard {
        Self(self.0 & self.0.wrapping_neg())
    }

    /// Converts a bitboard with a single bit set to a [`Square`] representing
    /// the set bit.
    ///
    /// If this bitboard is empty, returns [`None`].
    ///
    /// # Edge cases
    ///
    /// You should ideally only call this method on bitboards whose
    /// [`Self::pop_count()`] is 1. However, for performance reasons this method
    /// does not check this (except to check if the bitboard is empty), and
    /// instead simply returns the [`Square`] corresponding to the least
    /// significant bit that is set.
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
    pub fn squares(&self) -> impl Iterator<Item = Square> {
        BitIterator(self.0).map(Square::from_u8_unchecked)
    }
}

#[allow(clippy::eq_op, clippy::char_lit_as_u8)]
impl Bitboard {
    seq!(RANK in 1..=8 {
        paste! {
            pub const [<RANK_ RANK>]: Self = Self::rank_mask(RANK - 1);
        }
    });

    seq!(FILE in 'A'..='H' {
        paste! {
            pub const [<FILE _FILE>]: Self = Self::file_mask(FILE as u8 - b'A');
        }
    });
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
    /// * `.`, `0`, and `\u{00a0}` (non-breaking space) are interpreted as 0.
    /// * All other whitespace is ignored.
    /// * Any other character encountered is interpreted as 1.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut square_iter = (0..8)
            .rev()
            .flat_map(|rank| (0..8).map(move |file| Square::new_unchecked(rank, file)));

        let mut bitboard = Bitboard::EMPTY;
        for c in s.chars() {
            match c {
                '.' | '0' | '\u{00a0}' => {
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

impl FromIterator<Square> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::EMPTY, |acc, square| acc | Self::from_square(square))
    }
}

struct BitIterator(u64);

impl Iterator for BitIterator {
    type Item = u8;

    // This one #[inline] results in about a 25% speedup (4.31 m/s -> 5.48 m/s)
    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            let r = self.0.trailing_zeros();
            self.0 &= self.0 - 1;
            Some(r as _)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(64))
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    fn b(rank: u8, file: u8) -> Bitboard {
        Bitboard::from_square(Square::new_unchecked(rank, file))
    }

    #[test]
    fn bitboard_rank_mask() {
        assert_eq!(Bitboard::rank_mask(0), Bitboard(0x00000000000000FF));
        assert_eq!(Bitboard::rank_mask(1), Bitboard(0x000000000000FF00));
        assert_eq!(Bitboard::rank_mask(2), Bitboard(0x0000000000FF0000));
        assert_eq!(Bitboard::rank_mask(3), Bitboard(0x00000000FF000000));
        assert_eq!(Bitboard::rank_mask(4), Bitboard(0x000000FF00000000));
        assert_eq!(Bitboard::rank_mask(5), Bitboard(0x0000FF0000000000));
        assert_eq!(Bitboard::rank_mask(6), Bitboard(0x00FF000000000000));
        assert_eq!(Bitboard::rank_mask(7), Bitboard(0xFF00000000000000));
    }

    #[test]
    fn bitboard_file_mask() {
        assert_eq!(Bitboard::file_mask(0), Bitboard(0x0101010101010101));
        assert_eq!(Bitboard::file_mask(1), Bitboard(0x0202020202020202));
        assert_eq!(Bitboard::file_mask(2), Bitboard(0x0404040404040404));
        assert_eq!(Bitboard::file_mask(3), Bitboard(0x0808080808080808));
        assert_eq!(Bitboard::file_mask(4), Bitboard(0x1010101010101010));
        assert_eq!(Bitboard::file_mask(5), Bitboard(0x2020202020202020));
        assert_eq!(Bitboard::file_mask(6), Bitboard(0x4040404040404040));
        assert_eq!(Bitboard::file_mask(7), Bitboard(0x8080808080808080));
    }

    #[test]
    fn bitboard_aliases() {
        assert_eq!(Bitboard::RANK_1, Bitboard(0x00000000000000FF));
        assert_eq!(Bitboard::RANK_2, Bitboard(0x000000000000FF00));
        assert_eq!(Bitboard::RANK_3, Bitboard(0x0000000000FF0000));
        assert_eq!(Bitboard::RANK_4, Bitboard(0x00000000FF000000));
        assert_eq!(Bitboard::RANK_5, Bitboard(0x000000FF00000000));
        assert_eq!(Bitboard::RANK_6, Bitboard(0x0000FF0000000000));
        assert_eq!(Bitboard::RANK_7, Bitboard(0x00FF000000000000));
        assert_eq!(Bitboard::RANK_8, Bitboard(0xFF00000000000000));

        assert_eq!(Bitboard::A_FILE, Bitboard(0x0101010101010101));
        assert_eq!(Bitboard::B_FILE, Bitboard(0x0202020202020202));
        assert_eq!(Bitboard::C_FILE, Bitboard(0x0404040404040404));
        assert_eq!(Bitboard::D_FILE, Bitboard(0x0808080808080808));
        assert_eq!(Bitboard::E_FILE, Bitboard(0x1010101010101010));
        assert_eq!(Bitboard::F_FILE, Bitboard(0x2020202020202020));
        assert_eq!(Bitboard::G_FILE, Bitboard(0x4040404040404040));
        assert_eq!(Bitboard::H_FILE, Bitboard(0x8080808080808080));
    }

    #[test]
    fn bitboard_boolean_operators() {
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
    fn bitboard_bitwise_operators() {
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
    fn bitboard_shift_operators() {
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
    fn bitboard_has_piece_is_empty() {
        assert!(Bitboard::EMPTY.is_empty());
        assert!(!Bitboard::EMPTY.has_piece());

        assert!(!Bitboard(12).is_empty());
        assert!(Bitboard(12).has_piece());
    }

    #[test]
    fn bitboard_single_step() {
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
    fn bitboard_edge_step() {
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
    fn bitboard_step_multiple_bits() {
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
    fn bitboard_pop_count() {
        assert_eq!(Bitboard(0).pop_count(), 0);
        assert_eq!(Bitboard(0b10).pop_count(), 1);
        assert_eq!(Bitboard(0b11010100).pop_count(), 4);
        assert_eq!(Bitboard::RANK_8.pop_count(), 8);
        assert_eq!(Bitboard::ALL.pop_count(), 64);
    }

    #[test]
    fn bitboard_flip_vertical() {
        assert_eq!(
            Bitboard(0x0A0B0C0_D0E0F1011).flip_vertical(),
            Bitboard(0x11100F0_E0D0C0B0A)
        )
    }

    #[test]
    fn bitboard_msb_lsb() {
        let b = Bitboard(0x0FFF0003_8A200000);
        assert_eq!(b.msb(), Some(59));
        assert_eq!(b.lsb(), Some(21));

        assert_eq!(Bitboard::EMPTY.msb(), None);
        assert_eq!(Bitboard::EMPTY.lsb(), None);
    }

    #[test]
    fn bitboard_to_square() {
        assert_eq!(b(3, 7).to_square(), Square::new(3, 7));
        assert_eq!(b(1, 4).to_square(), Square::new(1, 4));
        assert_eq!(b(2, 5).to_square(), Square::new(2, 5));
        assert_eq!(b(0, 6).to_square(), Square::new(0, 6));

        assert_eq!(Bitboard::EMPTY.to_square(), None);
        assert_eq!((b(3, 7) | b(1, 4)).to_square(), Square::new(1, 4));
    }

    #[test]
    fn bitboard_from_numeric() {
        assert_eq!(Bitboard::from_index(0), b(0, 0));
        assert_eq!(Bitboard::from_u8(0), b(0, 0));

        assert_eq!(Bitboard::from_index(13), b(1, 5));
        assert_eq!(Bitboard::from_u8(13), b(1, 5));

        assert_eq!(Bitboard::from_index(59), b(7, 3));
        assert_eq!(Bitboard::from_u8(59), b(7, 3));
    }

    #[test]
    fn bitboard_get() {
        let x = b(1, 3) | b(5, 6) | b(3, 7);

        assert!(x.get(Square::new_unchecked(1, 3)));
        assert!(x.get(Square::new_unchecked(5, 6)));
        assert!(x.get(Square::new_unchecked(3, 7)));
        assert!(!x.get(Square::new_unchecked(2, 0)));
        assert!(!x.get(Square::new_unchecked(5, 1)));
        assert!(!x.get(Square::new_unchecked(4, 4)));
    }

    #[test]
    fn bitboard_set() {
        let mut x = Bitboard::EMPTY;

        x.set(Square::new_unchecked(3, 3));
        assert_eq!(x, b(3, 3));

        x.set(Square::new_unchecked(5, 2));
        assert_eq!(x, b(3, 3) | b(5, 2));

        x.set(Square::new_unchecked(3, 3));
        assert_eq!(x, b(3, 3) | b(5, 2));
    }

    #[test]
    fn bitboard_toggle() {
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
    fn bitboard_reset() {
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
    fn bitboard_bits() {
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
    fn bitboard_set_squares() {
        assert_eq!(Bitboard::EMPTY.squares().collect::<Vec<_>>(), vec![]);

        assert_eq!(
            Bitboard::ALL.squares().collect::<Vec<_>>(),
            (0..64)
                .map(Square::from_index_unchecked)
                .collect::<Vec<_>>()
        );

        assert_eq!(
            Bitboard(0b10001011_00111100).squares().collect::<Vec<_>>(),
            vec![2, 3, 4, 5, 8, 9, 11, 15]
                .into_iter()
                .map(Square::from_index_unchecked)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn bitboard_from_str() {
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
