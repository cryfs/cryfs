use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign, Sum};
use std::ops::{Div, DivAssign, Mul, MulAssign};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Sum,
    PartialOrd,
    Ord,
)]
pub struct NumBytes(u64);

impl Mul<u64> for NumBytes {
    type Output = NumBytes;
    fn mul(self, rhs: u64) -> Self::Output {
        NumBytes(self.0 * rhs)
    }
}
impl MulAssign<u64> for NumBytes {
    fn mul_assign(&mut self, rhs: u64) {
        self.0 *= rhs;
    }
}
impl Mul<NumBytes> for u64 {
    type Output = NumBytes;
    fn mul(self, rhs: NumBytes) -> Self::Output {
        NumBytes(self * rhs.0)
    }
}
impl Div<u64> for NumBytes {
    type Output = NumBytes;
    fn div(self, rhs: u64) -> Self::Output {
        NumBytes(self.0 / rhs)
    }
}
impl DivAssign<u64> for NumBytes {
    fn div_assign(&mut self, rhs: u64) {
        self.0 /= rhs;
    }
}
