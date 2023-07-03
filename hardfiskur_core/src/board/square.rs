use std::{
    fmt::{Display, Write},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    pub const WHITE_QUEENSIDE_ROOK: Self = Self::new_unchecked(0, 0);
    pub const WHITE_KINGSIDE_ROOK: Self = Self::new_unchecked(0, 7);
    pub const BLACK_QUEENSIDE_ROOK: Self = Self::new_unchecked(7, 0);
    pub const BLACK_KINGSIDE_ROOK: Self = Self::new_unchecked(7, 7);

    pub const fn new(rank: u8, file: u8) -> Option<Self> {
        if rank >= 8 || file >= 8 {
            None
        } else {
            Some(Self(8 * rank + file))
        }
    }

    pub const fn new_unchecked(rank: u8, file: u8) -> Self {
        Self(rank.overflowing_mul(8).0.overflowing_add(file).0 % 64)
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        if index < 64 {
            Some(Self(index as _))
        } else {
            None
        }
    }

    pub const fn from_u8_unchecked(value: u8) -> Self {
        Self(value % 64)
    }

    pub const fn from_index_unchecked(index: usize) -> Self {
        Self((index % 64) as _)
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as _
    }

    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    pub const fn file(self) -> u8 {
        self.0 % 8
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char((self.file() + b'a') as _)?;
        f.write_char((self.rank() + b'1') as _)
    }
}

impl FromStr for Square {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut char_iter = s.chars();
        let file = char_iter.next().ok_or(())?;
        let rank = char_iter.next().ok_or(())?;
        if char_iter.next().is_some() {
            return Err(());
        }

        let rank = (rank as i32) - ('1' as i32);
        let file = (file as i32) - ('a' as i32);
        if (0..8).contains(&rank) && (0..8).contains(&file) {
            Ok(Square::new_unchecked(rank as _, file as _))
        } else {
            Err(())
        }
    }
}
