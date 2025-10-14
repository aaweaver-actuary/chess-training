use std::fmt;

use crate::ids::{IdConversionError, IdKind};

/// Strongly typed identifier for specific moves in an opening tree.
///
/// # Examples
/// ```rust
/// use review_domain::ids::{IdConversionError, IdKind, MoveId};
///
/// let id = MoveId::try_from(99_u128).unwrap();
/// assert_eq!(id.get(), 99);
///
/// let overflow = MoveId::try_from(u128::from(u64::MAX) + 1);
/// assert!(matches!(
///     overflow,
///     Err(IdConversionError::Overflow { kind, value, max })
///         if kind == IdKind::Move && value == u128::from(u64::MAX) + 1 && max == u64::MAX
/// ));
///
/// let negative = MoveId::try_from(-1_i64);
/// assert!(matches!(
///     negative,
///     Err(IdConversionError::Negative { kind, value })
///         if kind == IdKind::Move && value == -1
/// ));
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoveId(pub u64);

impl MoveId {
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

impl From<u64> for MoveId {
    /// Creates a new identifier wrapper from a raw `u64` value.
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<MoveId> for u64 {
    /// Returns the raw `u64` backing this identifier.
    fn from(value: MoveId) -> Self {
        value.0
    }
}

impl TryFrom<u128> for MoveId {
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
                kind: IdKind::Move,
                value,
                max: u64::MAX,
            });
        }
        Ok(Self::new(value as u64))
    }
}

impl TryFrom<i128> for MoveId {
    type Error = IdConversionError;
    /// Attempts to create a new identifier wrapper from a raw `i128` value.
    /// Fails if the value is negative or exceeds the `u64` range.
    ///
    /// # Errors
    ///
    /// Returns `IdConversionError::Negative` if the value is negative.
    /// Returns `IdConversionError::Overflow` if the value exceeds `u64::MAX
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let value = u128::try_from(value).map_err(|_| IdConversionError::Negative {
            kind: IdKind::Move,
            value,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i64> for MoveId {
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

impl fmt::Display for MoveId {
    /// Formats the identifier for display purposes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MoveId({})", self.0)
    }
}
