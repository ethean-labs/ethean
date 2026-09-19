//! Epoch newtype and explicit slot conversion.

use core::fmt;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::PrimitiveError;
use crate::slot::Slot;

/// Consensus epoch index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Epoch(pub u64);

impl Epoch {
    /// Epoch zero.
    pub const ZERO: Self = Self(0);

    /// Construct from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Borrow the inner value.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Convert a slot into an epoch using an explicit `slots_per_epoch`.
    ///
    /// Does not hard-code any network constant; callers must supply the period.
    pub fn from_slot(slot: Slot, slots_per_epoch: u64) -> Result<Self, PrimitiveError> {
        if slots_per_epoch == 0 {
            return Err(PrimitiveError::ZeroDivisor);
        }
        Ok(Self(slot.get() / slots_per_epoch))
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

    /// Saturating subtraction (`rhs` may be a raw `u64` or another `Epoch`).
    pub fn saturating_sub(self, rhs: impl Into<u64>) -> Self {
        Self(self.0.saturating_sub(rhs.into()))
    }

    /// Saturating addition.
    pub fn saturating_add(self, rhs: impl Into<u64>) -> Self {
        Self(self.0.saturating_add(rhs.into()))
    }
}

impl From<u64> for Epoch {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Epoch> for u64 {
    fn from(value: Epoch) -> Self {
        value.0
    }
}

impl Add<u64> for Epoch {
    type Output = Epoch;

    fn add(self, rhs: u64) -> Epoch {
        Epoch(self.0 + rhs)
    }
}

impl Sub<u64> for Epoch {
    type Output = Epoch;

    fn sub(self, rhs: u64) -> Epoch {
        Epoch(self.0 - rhs)
    }
}

impl AddAssign<u64> for Epoch {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl SubAssign<u64> for Epoch {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

impl Mul<u64> for Epoch {
    type Output = Slot;

    fn mul(self, rhs: u64) -> Slot {
        Slot::new(self.0.saturating_mul(rhs))
    }
}

impl PartialEq<u64> for Epoch {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u64> for Epoch {
    fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slot_rejects_zero_period() {
        assert_eq!(
            Epoch::from_slot(Slot::new(10), 0),
            Err(PrimitiveError::ZeroDivisor)
        );
    }

    #[test]
    fn from_slot_divides() {
        assert_eq!(
            Epoch::from_slot(Slot::new(64), 32).unwrap(),
            Epoch::new(2)
        );
    }

    #[test]
    fn checked_add_overflow() {
        assert_eq!(
            Epoch::new(u64::MAX).checked_add(1),
            Err(PrimitiveError::Overflow)
        );
    }

    #[test]
    fn zero_epoch() {
        assert_eq!(Epoch::ZERO.get(), 0);
    }
}
