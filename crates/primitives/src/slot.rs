//! Slot newtype with checked arithmetic.

use crate::error::PrimitiveError;
use core::fmt;
use core::ops::{Add, AddAssign, Div, Rem, Sub, SubAssign};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Consensus slot index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Slot(pub u64);

impl Slot {
    /// Slot zero.
    pub const ZERO: Self = Self(0);

    /// Construct from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Borrow the inner value.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Checked addition.
    pub fn checked_add(self, rhs: u64) -> Result<Self, PrimitiveError> {
        self.0
            .checked_add(rhs)
            .map(Self)
            .ok_or(PrimitiveError::Overflow)
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: u64) -> Result<Self, PrimitiveError> {
        self.0
            .checked_sub(rhs)
            .map(Self)
            .ok_or(PrimitiveError::Underflow)
    }

    /// Saturating subtraction (`rhs` may be a raw `u64` or another `Slot`).
    pub fn saturating_sub(self, rhs: impl Into<u64>) -> Self {
        Self(self.0.saturating_sub(rhs.into()))
    }

    /// Saturating addition.
    pub fn saturating_add(self, rhs: impl Into<u64>) -> Self {
        Self(self.0.saturating_add(rhs.into()))
    }
}

impl From<u64> for Slot {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Slot> for u64 {
    fn from(value: Slot) -> Self {
        value.0
    }
}

impl Add<u64> for Slot {
    type Output = Slot;

    fn add(self, rhs: u64) -> Slot {
        Slot(self.0 + rhs)
    }
}

impl Sub<u64> for Slot {
    type Output = Slot;

    fn sub(self, rhs: u64) -> Slot {
        Slot(self.0 - rhs)
    }
}

impl AddAssign<u64> for Slot {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl SubAssign<u64> for Slot {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

impl Div<u64> for Slot {
    type Output = u64;

    fn div(self, rhs: u64) -> u64 {
        self.0 / rhs
    }
}

impl Rem<u64> for Slot {
    type Output = u64;

    fn rem(self, rhs: u64) -> u64 {
        self.0 % rhs
    }
}

impl PartialEq<u64> for Slot {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u64> for Slot {
    fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_add_overflow() {
        assert_eq!(
            Slot::new(u64::MAX).checked_add(1),
            Err(PrimitiveError::Overflow)
        );
    }

    #[test]
    fn checked_sub_underflow() {
        assert_eq!(Slot::ZERO.checked_sub(1), Err(PrimitiveError::Underflow));
    }

    #[test]
    fn checked_add_ok() {
        assert_eq!(Slot::new(3).checked_add(2).unwrap(), Slot::new(5));
    }
}
