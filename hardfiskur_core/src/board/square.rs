use std::{
    fmt::{Debug, Display, Write},
    str::FromStr,
};

use paste::paste;
use seq_macro::seq;
use thiserror::Error;

/// Represents a square on the chessboard.
///
/// Internally, represents a square as an integer from 0-63, ordered by
/// increasing file then rank, so that 0 is a1, 1 is b1, 2 is c1... 7 is h1, 8
/// is a2, 9 is b2, etc.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    /// Standard starting position of the white queenside rook.
    pub const WHITE_QUEENSIDE_ROOK: Self = Self::new_unchecked(0, 0);
    /// Standard starting position of the white kingside rook.
    pub const WHITE_KINGSIDE_ROOK: Self = Self::new_unchecked(0, 7);
    /// Standard starting position of the black queenside rook.
    pub const BLACK_QUEENSIDE_ROOK: Self = Self::new_unchecked(7, 0);
    /// Standard starting position of the black kingside rook.
    pub const BLACK_KINGSIDE_ROOK: Self = Self::new_unchecked(7, 7);

    /// Construct a [`Square`] from the provided rank and file.
    ///
    /// Ranks are numbered 0-7 with 0 being rank 1 and 7 being rank 8.
    ///
    /// Files are numbered 0-7 with 0 being file A and 7 being file H.
    ///
    /// Returns [`None`] if either `rank` or `file` are greater than 7.
    pub const fn new(rank: u8, file: u8) -> Option<Self> {
        if rank >= 8 || file >= 8 {
            None
        } else {
            Some(Self(8 * rank + file))
        }
    }

    /// Construct a [`Square`] from the provided rank and file, without checking
    /// if the rank and file are valid. Use only if you are sure that `rank` and
    /// `file` are both within the range `0..=7`.
    ///
    /// Note that it is not possible to construct an invalid square from this
    /// method if an invalid `rank` or `file` is provided, as the final result
    /// is truncated to between 0-63 before being stored. This means that the
    /// method does not need to be marked as unsafe. However, this will likely
    /// not be the square that you want.
    pub const fn new_unchecked(rank: u8, file: u8) -> Self {
        Self(rank.overflowing_mul(8).0.overflowing_add(file).0 % 64)
    }

    /// Construct a [`Square`] from the provided [`usize`].
    ///
    /// The indexing scheme starts with 0 being a1, 7 being h1, and 63 being h8.
    /// Values outside the range 0-63 will return [`None`].
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < 64 {
            Some(Self(index as _))
        } else {
            None
        }
    }

    /// Construct a [`Square`] from the provided [`u8`].
    ///
    /// Values outside the range 0-63 will be truncated to within the range to
    /// produce a valid square.
    pub const fn from_u8_unchecked(value: u8) -> Self {
        Self(value % 64)
    }

    /// Construct a [`Square`] from the provided [`usize`].
    ///
    /// Values outside the range 0-63 will be truncated to within the range to
    /// produce a valid square.
    pub const fn from_index_unchecked(index: usize) -> Self {
        Self((index % 64) as _)
    }

    /// Returns the index of this square as a [`u8`].
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns the index of this square as a [`usize`].
    pub const fn index(self) -> usize {
        self.0 as _
    }

    /// Returns the rank of this square.
    ///
    /// Ranks are numbered 0-7 with 0 being rank 1 and 7 being rank 8.
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    /// Returns the file of this square.
    ///
    /// Files are numbered 0-7 with 0 being file A and 7 being file H.
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    /// Returns an iterator over every single square.
    pub fn all() -> impl Iterator<Item = Square> {
        (0..64).map(Square)
    }

    /// Adds an offset to this given square and returns a new square.
    ///
    /// For example, to find the square to the north of this one, you can call
    /// this function with an offset of +8.
    ///
    /// No checking is done to make sure that offsetting does not undesirably
    /// wrap, but the final value is always truncated to between 0..=63 so the
    /// resulting square is always valid.
    pub const fn offset(self, offset: i8) -> Self {
        Self::from_u8_unchecked((self.0 as i8).wrapping_add(offset) as u8)
    }

    pub fn manhattan_distance(self, other: Self) -> u8 {
        let (r1, f1) = (self.rank(), self.file());
        let (r2, f2) = (other.rank(), other.file());

        r1.abs_diff(r2) + f1.abs_diff(f2)
    }

    pub fn euclidean_distance_sq(self, other: Self) -> u32 {
        let (r1, f1) = (self.rank(), self.file());
        let (r2, f2) = (other.rank(), other.file());

        (r1.abs_diff(r2) as u32).pow(2) + (f1.abs_diff(f2) as u32).pow(2)
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char((self.file() + b'a') as _)?;
        f.write_char((self.rank() + b'1') as _)
    }
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid square")]
pub struct ParseSquareError;

impl FromStr for Square {
    type Err = ParseSquareError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut char_iter = s.chars();
        let file = char_iter.next().ok_or(ParseSquareError)?;
        let rank = char_iter.next().ok_or(ParseSquareError)?;
        if char_iter.next().is_some() {
            return Err(ParseSquareError);
        }

        let rank = (rank as i32) - ('1' as i32);
        let file = (file as i32) - ('a' as i32);
        if (0..8).contains(&rank) && (0..8).contains(&file) {
            Ok(Square::new_unchecked(rank as _, file as _))
        } else {
            Err(ParseSquareError)
        }
    }
}

/// Board square aliases
#[allow(clippy::eq_op, clippy::char_lit_as_u8)]
impl Square {
    seq!(RANK in 1..=8 {
        seq!(FILE in 'A'..='H' {
            paste! {
                pub const [<FILE RANK>]: Square = Square::new_unchecked(RANK - 1, FILE as u8 - b'A');
            }
        });
    });
}

impl Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            (self.file() + b'A') as char,
            (self.rank() + b'1') as char
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn square_new() {
        assert_eq!(Square::new(5, 7), Some(Square(47)));
        assert_eq!(Square::new(2, 3), Some(Square(19)));
        assert_eq!(Square::new(7, 1), Some(Square(57)));
        assert_eq!(Square::new(6, 0), Some(Square(48)));

        assert_eq!(Square::new(3, 9), None);
        assert_eq!(Square::new(8, 0), None);
        assert_eq!(Square::new(20, 1), None);
        assert_eq!(Square::new(37, 128), None);
    }

    #[test]
    fn square_new_unchecked() {
        assert_eq!(Square::new_unchecked(5, 7), Square(47));
        assert_eq!(Square::new_unchecked(2, 3), Square(19));
        assert_eq!(Square::new_unchecked(7, 1), Square(57));
        assert_eq!(Square::new_unchecked(6, 0), Square(48));

        assert_eq!(Square::new_unchecked(3, 9), Square(33));
        assert_eq!(Square::new_unchecked(8, 0), Square(0));
        assert_eq!(Square::new_unchecked(20, 1), Square(33));
        assert_eq!(Square::new_unchecked(37, 128), Square(40));
    }

    #[test]
    fn square_aliases() {
        assert_eq!(Square::new_unchecked(5, 7), Square::H6);
        assert_eq!(Square::new_unchecked(2, 3), Square::D3);
        assert_eq!(Square::new_unchecked(7, 1), Square::B8);
        assert_eq!(Square::new_unchecked(6, 0), Square::A7);
    }

    #[test]
    fn square_from_numeric() {
        for i in 0..64 {
            assert_eq!(Square::from_index(i as _), Some(Square(i)));
            assert_eq!(Square::from_u8_unchecked(i), Square(i));
            assert_eq!(Square::from_index_unchecked(i as _), Square(i));
        }

        assert_eq!(Square::from_index(64), None);
        assert_eq!(Square::from_u8_unchecked(64), Square(0));
        assert_eq!(Square::from_index_unchecked(64), Square(0));
    }

    #[test]
    fn square_to_numeric() {
        for i in 0..64 {
            assert_eq!(Square(i).get(), i);
            assert_eq!(Square(i).index(), i as _);
        }
    }

    #[test]
    fn square_rank_and_file() {
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(rank, file).unwrap();
                assert_eq!(square.rank(), rank);
                assert_eq!(square.file(), file);
            }
        }
    }

    #[test]
    fn square_display() {
        let cases = [
            (Square(17), "b3"),
            (Square(63), "h8"),
            (Square(3), "d1"),
            (Square(46), "g6"),
        ];

        for (square, expected) in cases {
            assert_eq!(format!("{square}"), expected);
        }
    }

    #[test]
    fn square_from_str() {
        assert_eq!("a7".parse::<Square>(), Ok(Square(48)));
        assert_eq!("f2".parse::<Square>(), Ok(Square(13)));

        assert_eq!("".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("x".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("f23".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a1 ".parse::<Square>(), Err(ParseSquareError));
    }

    #[test]
    fn square_all() {
        let mut expected = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                expected.push(Square::new(rank, file).unwrap());
            }
        }

        let all = Square::all().collect::<Vec<_>>();

        assert_eq!(all, expected);
    }

    #[test]
    fn square_offset() {
        assert_eq!(Square::E4.offset(8), Square::E5);
        assert_eq!(Square::A1.offset(9), Square::B2);
        assert_eq!(Square::H3.offset(-1), Square::G3);
        assert_eq!(Square::F6.offset(-7), Square::G5);

        assert_eq!(Square::E4.offset(16), Square::E6);
        assert_eq!(Square::E4.offset(-16), Square::E2);

        assert_eq!(Square::A1.offset(-8), Square::A8);
        assert_eq!(Square::H6.offset(9), Square::A8);
    }
}
