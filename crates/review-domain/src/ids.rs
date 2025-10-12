//! Strongly-typed identifier wrappers shared across the review domain crate.

use core::fmt;

/// Errors encountered when converting primitive numeric values into identifier wrappers.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum IdConversionError {
    /// The provided value cannot be represented as an unsigned 64-bit integer.
    #[error("identifier value {value} exceeds u64::MAX")]
    Overflow { value: u128 },
}

macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(u64);

        impl $name {
            /// Creates a new identifier wrapper from a 64-bit unsigned integer.
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the underlying raw identifier value.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl TryFrom<u128> for $name {
            type Error = IdConversionError;

            fn try_from(value: u128) -> Result<Self, Self::Error> {
                u64::try_from(value)
                    .map(Self::new)
                    .map_err(|_| IdConversionError::Overflow { value })
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id_type!(PositionId);
define_id_type!(EdgeId);
define_id_type!(MoveId);
define_id_type!(CardId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_wrapper_with_expected_value() {
        let id = CardId::new(42);
        assert_eq!(id.get(), 42);
        assert_eq!(u64::from(id), 42);
        assert_eq!(CardId::from(42_u64), id);
    }

    #[test]
    fn display_renders_underlying_value() {
        let id = PositionId::new(7);
        assert_eq!(id.to_string(), "7");
    }
}
