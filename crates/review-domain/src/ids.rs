//! Identifier wrappers for review-domain entities.

use core::fmt;
use thiserror::Error;

/// Enumerates the identifier categories supported by this crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdKind {
    /// Identifier representing a chess position.
    Position,
    /// Identifier representing a repertoire edge.
    Edge,
    /// Identifier representing a move hashed from SAN/position context.
    Move,
    /// Identifier representing a review card instance.
    Card,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Position => "position",
            Self::Edge => "edge",
            Self::Move => "move",
            Self::Card => "card",
        };

        f.write_str(label)
    }
}

/// Errors that occur when attempting to convert primitive values into identifier wrappers.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum IdConversionError {
    /// Returned when a signed integer is negative.
    #[error("{kind} identifiers cannot be negative (received {value})")]
    Negative {
        /// The category of identifier being constructed.
        kind: IdKind,
        /// The offending value.
        value: i128,
    },
    /// Returned when an unsigned integer exceeds [`u64::MAX`].
    #[error("{kind} identifiers must be at most {max} (received {value})")]
    Overflow {
        /// The category of identifier being constructed.
        kind: IdKind,
        /// The offending value.
        value: u128,
        /// Maximum supported value.
        max: u64,
    },
}

macro_rules! define_id {
    ($name:ident, $kind:expr) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[repr(transparent)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name(u64);

        impl $name {
            /// Creates a new identifier from a raw [`u64`] value.
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the raw [`u64`] backing this identifier.
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
                value.get()
            }
        }

        impl TryFrom<i64> for $name {
            type Error = IdConversionError;

            fn try_from(value: i64) -> Result<Self, Self::Error> {
                match u64::try_from(value) {
                    Ok(raw) => Ok(Self::new(raw)),
                    Err(_) => Err(IdConversionError::Negative {
                        kind: $kind,
                        value: i128::from(value),
                    }),
                }
            }
        }

        impl TryFrom<u128> for $name {
            type Error = IdConversionError;

            fn try_from(value: u128) -> Result<Self, Self::Error> {
                match u64::try_from(value) {
                    Ok(raw) => Ok(Self::new(raw)),
                    Err(_) => Err(IdConversionError::Overflow {
                        kind: $kind,
                        value,
                        max: u64::MAX,
                    }),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.get())
            }
        }
    };
}

define_id!(PositionId, IdKind::Position);
define_id!(EdgeId, IdKind::Edge);
define_id!(MoveId, IdKind::Move);
define_id!(CardId, IdKind::Card);
