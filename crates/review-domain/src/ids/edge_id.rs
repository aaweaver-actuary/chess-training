use std::fmt;

use crate::ids::{IdConversionError, IdKind};

/// Strongly typed identifier for directed edges between positions.
///
/// # Examples
/// ```rust
/// use review_domain::ids::{EdgeId, IdConversionError, IdKind};
///
/// let id = EdgeId::try_from(7_u128).unwrap();
/// assert_eq!(id.get(), 7);
///
/// let overflow = EdgeId::try_from(u128::from(u64::MAX) + 1);
/// assert!(matches!(
///     overflow,
///     Err(IdConversionError::Overflow { kind, value, max })
///         if kind == IdKind::Edge && value == u128::from(u64::MAX) + 1 && max == u64::MAX
/// ));
///
/// let negative = EdgeId::try_from(-1_i64);
/// assert!(matches!(
///     negative,
///     Err(IdConversionError::Negative { kind, value })
///         if kind == IdKind::Edge && value == -1
/// ));
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EdgeId(pub u64);

impl EdgeId {
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

impl From<u64> for EdgeId {
    /// Creates a new identifier wrapper from a raw `u64` value.
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<EdgeId> for u64 {
    /// Returns the raw `u64` backing this identifier.
    fn from(value: EdgeId) -> Self {
        value.0
    }
}

impl TryFrom<u128> for EdgeId {
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
                kind: IdKind::Edge,
                value,
                max: u64::MAX,
            });
        }
        Ok(Self::new(u64::try_from(value).unwrap()))
    }
}

impl TryFrom<i128> for EdgeId {
    type Error = IdConversionError;
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let value = u128::try_from(value).map_err(|_| IdConversionError::Negative {
            kind: IdKind::Edge,
            value,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i64> for EdgeId {
    type Error = IdConversionError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(i128::from(value))
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EdgeId({})", self.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_new_and_get() {
        let id = EdgeId::new(42);
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn test_from_u64() {
        let id: EdgeId = 123u64.into();
        assert_eq!(id.get(), 123);
    }

    #[test]
    fn test_into_u64() {
        let id = EdgeId::new(999);
        let raw: u64 = id.into();
        assert_eq!(raw, 999);
    }

    #[test]
    fn test_try_from_u128_within_range() {
        let id = EdgeId::try_from(0_u128).unwrap();
        assert_eq!(id.get(), 0);

        let id = EdgeId::try_from(u128::from(u64::MAX)).unwrap();
        assert_eq!(id.get(), u64::MAX);
    }

    #[test]
    fn test_try_from_u128_overflow() {
        let overflow_val = u128::from(u64::MAX) + 1;
        let err = EdgeId::try_from(overflow_val).unwrap_err();
        assert!(matches!(
            err,
            IdConversionError::Overflow { kind, value, max }
                if kind == IdKind::Edge && value == overflow_val && max == u64::MAX
        ));
    }

    #[test]
    fn test_try_from_i128_positive() {
        let id = EdgeId::try_from(123_i128).unwrap();
        assert_eq!(id.get(), 123);
    }

    #[test]
    fn test_try_from_i128_negative() {
        let err = EdgeId::try_from(-1_i128).unwrap_err();
        assert!(matches!(
            err,
            IdConversionError::Negative { kind, value }
                if kind == IdKind::Edge && value == -1
        ));
    }

    #[test]
    fn test_try_from_i128_overflow() {
        let overflow_val = u128::from(u64::MAX) + 1;
        let err = EdgeId::try_from(overflow_val).unwrap_err();
        assert!(matches!(
            err,
            IdConversionError::Overflow { kind, value, max }
                if kind == IdKind::Edge && value == overflow_val && max == u64::MAX
        ));
    }

    #[test]
    fn test_try_from_i64_positive() {
        let id = EdgeId::try_from(456_i64).unwrap();
        assert_eq!(id.get(), 456);
    }

    #[test]
    fn test_try_from_i64_negative() {
        let err = EdgeId::try_from(-42_i64).unwrap_err();
        assert!(matches!(
            err,
            IdConversionError::Negative { kind, value }
                if kind == IdKind::Edge && value == -42
        ));
    }

    #[test]
    fn test_display() {
        let id = EdgeId::new(12345);
        assert_eq!(format!("{id}"), "EdgeId(12345)");
    }

    #[test]
    fn test_default() {
        let id = EdgeId::default();
        assert_eq!(id.get(), 0);
    }

    #[test]
    fn test_equality_and_ordering() {
        let a = EdgeId::new(1);
        let b = EdgeId::new(2);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, EdgeId::new(1));
        assert_ne!(a, b);
    }

    #[test]
    fn test_hash() {
        let id1 = EdgeId::new(100);
        let id2 = EdgeId::new(100);
        let id3 = EdgeId::new(101);

        let mut hasher1 = DefaultHasher::new();
        id1.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        id2.hash(&mut hasher2);

        let mut hasher3 = DefaultHasher::new();
        id3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }
}
