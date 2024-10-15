use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedScore(i64);

impl PackedScore {
    pub const ZERO: Self = Self(0);

    pub const fn new(mg: i32, eg: i32) -> Self {
        Self(((eg as i64) << 32) + mg as i64)
    }

    pub const fn mg(self) -> i32 {
        self.0 as i32
    }

    pub const fn eg(self) -> i32 {
        ((self.0 + 0x8000_0000) >> 32) as i32
    }
}

pub type S = PackedScore;

impl Add for PackedScore {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for PackedScore {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for PackedScore {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for PackedScore {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Neg for PackedScore {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Mul<i32> for PackedScore {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs as i64)
    }
}

impl Mul<PackedScore> for i32 {
    type Output = PackedScore;

    fn mul(self, rhs: PackedScore) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<i32> for PackedScore {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs
    }
}

#[macro_export]
macro_rules! s {
    (0) => {
        crate::evaluation::packed_score::PackedScore::new(0, 0)
    };
    ($mg:literal, $eg:literal) => {
        crate::evaluation::packed_score::PackedScore::new($mg, $eg)
    };
}
