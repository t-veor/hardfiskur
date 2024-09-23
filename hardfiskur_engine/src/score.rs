use std::{
    fmt::{Display, Write},
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(pub i64);

impl Score {
    // This is not i64::MAX so that adding small numbers to it doesn't overflow.
    pub const INF: Self = Self(999_999_999);

    const MATE_SCORE: i64 = 20_000_000;
    const MATE_THRESHOLD: i64 = 1_000_000;

    pub const fn get(self) -> i64 {
        self.0
    }

    pub const fn mate_in_plies(ply_from_root: u32) -> Self {
        Self(Self::MATE_SCORE - ply_from_root as i64)
    }

    pub const fn is_mate(self) -> bool {
        self.0.abs() > Self::MATE_THRESHOLD
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<i64> for Score {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<Score> for i64 {
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

impl AddAssign<i64> for Score {
    fn add_assign(&mut self, rhs: i64) {
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

impl Sub<i64> for Score {
    type Output = Self;

    fn sub(self, rhs: i64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl SubAssign<i64> for Score {
    fn sub_assign(&mut self, rhs: i64) {
        *self = *self - rhs
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = self.0.signum();
        let val = self.0.abs();

        f.write_char(if sign >= 0 { '+' } else { '-' })?;

        if val > Self::MATE_THRESHOLD {
            let plies_to_mate = Self::MATE_SCORE - val;
            let moves_to_mate = plies_to_mate / 2;
            write!(f, "M{moves_to_mate}")
        } else {
            let pawn_advantage = val as f64 / 1000.0;
            write!(f, "{pawn_advantage:.3}")
        }
    }
}
