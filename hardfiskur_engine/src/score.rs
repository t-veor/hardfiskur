use std::{
    fmt::{Display, Write},
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use zerocopy::FromZeroes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, FromZeroes)]
pub struct Score(pub i32);

impl Score {
    // This is not i32::MAX so that adding small numbers to it doesn't overflow.
    pub const INF: Self = Self(1_000_000_000);

    const MATE_SCORE: i32 = 20_000_000;
    const MATE_THRESHOLD: i32 = 1_000_000;

    pub const fn get(self) -> i32 {
        self.0
    }

    pub const fn mate_in_plies(ply_from_root: u16) -> Self {
        Self(Self::MATE_SCORE - ply_from_root as i32)
    }

    pub const fn is_mate(self) -> bool {
        self.0.abs() > Self::MATE_THRESHOLD
    }

    pub fn is_mate_for_us(&self) -> bool {
        self.0 > Self::MATE_THRESHOLD
    }

    pub const fn as_mate_in(self) -> Option<i32> {
        if self.0.abs() > Self::MATE_THRESHOLD {
            Some(self.0.signum() * (Self::MATE_SCORE - self.0.abs() + 1) / 2)
        } else {
            None
        }
    }

    pub const fn as_mate_in_plies(self) -> Option<i32> {
        if self.0.abs() > Self::MATE_THRESHOLD {
            Some(self.0.signum() * (Self::MATE_SCORE - self.0.abs()))
        } else {
            None
        }
    }

    pub const fn as_centipawns(self) -> Option<i32> {
        if self.0.abs() > Self::MATE_THRESHOLD {
            None
        } else {
            Some(self.0 / 10)
        }
    }

    pub const fn sub_plies_for_mate(self, ply_from_root: u16) -> Self {
        if self.0 > Self::MATE_THRESHOLD {
            Self(self.0 + ply_from_root as i32)
        } else if self.0 < Self::MATE_THRESHOLD {
            Self(self.0 - ply_from_root as i32)
        } else {
            self
        }
    }

    pub const fn add_plies_for_mate(self, ply_from_root: u16) -> Self {
        if self.0 > Self::MATE_THRESHOLD {
            Self(self.0 - ply_from_root as i32)
        } else if self.0 < Self::MATE_THRESHOLD {
            Self(self.0 + ply_from_root as i32)
        } else {
            self
        }
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<i32> for Score {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<Score> for i32 {
    type Output = Score;

    fn add(self, rhs: Score) -> Self::Output {
        Score(self + rhs.0)
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl AddAssign<i32> for Score {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<i32> for Score {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl SubAssign<i32> for Score {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = self.0.signum();
        let val = self.0.abs();

        f.write_char(if sign >= 0 { '+' } else { '-' })?;

        if val >= Self::INF.0 {
            write!(f, "inf")
        } else if let Some(mate_score) = self.as_mate_in() {
            write!(f, "M{}", mate_score.abs())
        } else {
            let pawn_advantage = val as f64 / 1000.0;
            write!(f, "{pawn_advantage:.3}")
        }
    }
}
