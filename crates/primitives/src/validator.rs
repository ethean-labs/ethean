//! Validator index newtype.

use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Index into the validator registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ValidatorIndex(pub u64);

impl ValidatorIndex {
    /// Index zero.
    pub const ZERO: Self = Self(0);

    /// Construct from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Borrow the inner value.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Convert to `usize` for local collection indexing.
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u64> for ValidatorIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ValidatorIndex> for u64 {
    fn from(value: ValidatorIndex) -> Self {
        value.0
    }
}

impl PartialEq<u64> for ValidatorIndex {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for ValidatorIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_index() {
        assert_eq!(ValidatorIndex::ZERO.get(), 0);
        assert_eq!(ValidatorIndex::ZERO.as_usize(), 0);
    }
}
