#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{IdConversionError, IdKind};

    #[test]
    fn new_and_get() {
        let id = CardId::new(123);
        assert_eq!(id.get(), 123);
    }

    #[test]
    fn from_u64_and_into_u64() {
        let id: CardId = 42u64.into();
        let raw: u64 = id.into();
        assert_eq!(raw, 42);
    }

    #[test]
    fn try_from_u128_success() {
        let id = CardId::try_from(7_u128).unwrap();
        assert_eq!(id.get(), 7);
    }

    #[test]
    fn try_from_u128_overflow() {
        let overflow = CardId::try_from(u128::from(u64::MAX) + 1);
        match overflow {
            Err(IdConversionError::Overflow { kind, value, max }) => {
                assert_eq!(kind, IdKind::Card);
                assert_eq!(value, u128::from(u64::MAX) + 1);
                assert_eq!(max, u64::MAX);
            }
            _ => panic!("Expected Overflow error"),
        }
    }

    #[test]
    fn try_from_i128_success() {
        let id = CardId::try_from(123_i128).unwrap();
        assert_eq!(id.get(), 123);
    }

    #[test]
    fn try_from_i128_negative() {
        let negative = CardId::try_from(-1_i128);
        match negative {
            Err(IdConversionError::Negative { kind, value }) => {
                assert_eq!(kind, IdKind::Card);
                assert_eq!(value, -1_i128);
            }
            _ => panic!("Expected Negative error"),
        }
    }

    #[test]
    fn try_from_i128_overflow() {
        let overflow = CardId::try_from(i128::try_from(u128::from(u64::MAX)).unwrap() + 1);
        match overflow {
            Err(IdConversionError::Overflow { kind, value, max }) => {
                assert_eq!(kind, IdKind::Card);
                assert_eq!(value, u128::from(u64::MAX) + 1);
                assert_eq!(max, u64::MAX);
            }
            _ => panic!("Expected Overflow error"),
        }
    }

    #[test]
    fn try_from_i64_success() {
        let id = CardId::try_from(99_i64).unwrap();
        assert_eq!(id.get(), 99);
    }

    #[test]
    fn try_from_i64_negative() {
        let negative = CardId::try_from(-1_i64);
        match negative {
            Err(IdConversionError::Negative { kind, value }) => {
                assert_eq!(kind, IdKind::Card);
                assert_eq!(value, i128::from(-1_i64));
            }
            _ => panic!("Expected Negative error"),
        }
    }

    #[test]
    fn try_from_i64_overflow() {
        let overflow = CardId::try_from(i128::from(i64::MAX) + 1);
        // This will be positive and within u64, so should succeed
        assert!(overflow.is_ok());
        let overflow = CardId::try_from(i64::try_from(u128::from(u64::MAX)).unwrap() + 1);
        // This will be negative, so should error
        assert!(overflow.is_err());
    }

    #[test]
    fn display_impl() {
        let id = CardId::new(555);
        assert_eq!(format!("{id}"), "CardId(555)");
    }

    #[test]
    fn default_is_zero() {
        let id = CardId::default();
        assert_eq!(id.get(), 0);
    }
}
use std::fmt;

use crate::ids::{IdConversionError, IdKind};

/// Strongly typed identifier for review cards stored in the system.
///
/// ```
/// use review_domain::ids::{CardId, IdConversionError, IdKind};
///
/// let id = CardId::try_from(7_u128).unwrap();
/// assert_eq!(id.get(), 7);
///
/// let overflow = CardId::try_from(u128::from(u64::MAX) + 1);
/// assert!(matches!(
///     overflow,
///     Err(IdConversionError::Overflow { kind, value, max })
///         if kind == IdKind::Card && value == u128::from(u64::MAX) + 1 && max == u64::MAX
/// ));
///
/// let negative = CardId::try_from(-1_i64);
/// assert!(matches!(
///     negative,
///     Err(IdConversionError::Negative { kind, value })
///         if kind == IdKind::Card && value == -1
/// ));
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardId(u64);

impl CardId {
    /// Creates a new identifier wrapper from a raw `u64` value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw `u64` backing this identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for CardId {
    /// Creates a new identifier wrapper from a raw `u64` value.
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<CardId> for u64 {
    /// Returns the raw `u64` backing this identifier.
    fn from(value: CardId) -> Self {
        value.0
    }
}

impl TryFrom<u128> for CardId {
    type Error = IdConversionError;
    /// Attempts to create a new identifier wrapper from a raw `u128` value.
    /// Fails if the value exceeds the `u64` range.
    ///
    /// # Errors
    ///
    /// Returns `IdConversionError::Overflow` if the value exceeds `u64::MAX`.
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value > u128::from(u64::MAX) {
            return Err(IdConversionError::Overflow {
                kind: IdKind::Card,
                value,
                max: u64::MAX,
            });
        }
        Ok(Self::new(u64::try_from(value).unwrap()))
    }
}

impl TryFrom<i128> for CardId {
    type Error = IdConversionError;
    /// Attempts to create a new identifier wrapper from a raw `i128` value.
    /// Fails if the value is negative or exceeds the `u64` range.
    ///
    /// # Errors
    ///
    /// Returns `IdConversionError::Negative` if the value is negative.
    /// Returns `IdConversionError::Overflow` if the value exceeds `u64::MAX`.
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let value = u128::try_from(value).map_err(|_| IdConversionError::Negative {
            kind: IdKind::Card,
            value,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i64> for CardId {
    type Error = IdConversionError;

    /// Attempts to create a new identifier wrapper from a raw `i64` value.
    /// Fails if the value is negative or exceeds the `u64` range.
    ///
    /// # Errors
    ///
    /// Returns `IdConversionError::Negative` if the value is negative.
    /// Returns `IdConversionError::Overflow` if the value exceeds `u64::MAX`.
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(i128::from(value))
    }
}

impl fmt::Display for CardId {
    /// Formats the identifier for display purposes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CardId({})", self.0)
    }
}
