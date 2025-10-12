//! Identifier newtypes that wrap raw `u64` values for stronger type safety.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Errors that can occur when constructing identifier newtypes from primitive values.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdentifierError {
    /// The provided value cannot be represented within a `u64`.
    #[error(
        "{type_name} cannot be constructed from value {attempted_value} because it exceeds u64::MAX"
    )]
    Overflow {
        /// Name of the identifier type that failed to construct.
        type_name: &'static str,
        /// The numeric value that overflowed the target identifier range.
        attempted_value: u128,
    },
    /// Attempted to construct an identifier from a negative numeric value.
    #[error("{type_name} cannot be constructed from a negative value")]
    Negative {
        /// Name of the identifier type that failed to construct.
        type_name: &'static str,
    },
    /// Failed to parse an identifier from a string representation.
    #[error("{type_name} failed to parse from '{input}'")]
    Parse {
        /// Name of the identifier type that failed to construct.
        type_name: &'static str,
        /// The provided input string.
        input: String,
    },
}

macro_rules! define_identifier {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(u64);

        impl $name {
            /// Creates a new identifier from the provided `u64`.
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the underlying numeric value.
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
            type Error = IdentifierError;

            fn try_from(value: u128) -> Result<Self, Self::Error> {
                u64::try_from(value)
                    .map(Self::new)
                    .map_err(|_| IdentifierError::Overflow {
                        type_name: stringify!($name),
                        attempted_value: value,
                    })
            }
        }

        impl TryFrom<i128> for $name {
            type Error = IdentifierError;

            fn try_from(value: i128) -> Result<Self, Self::Error> {
                match u64::try_from(value) {
                    Ok(number) => Ok(Self::new(number)),
                    Err(_) if value < 0 => Err(IdentifierError::Negative {
                        type_name: stringify!($name),
                    }),
                    Err(_) => Err(IdentifierError::Overflow {
                        type_name: stringify!($name),
                        attempted_value: u128::try_from(value).unwrap_or(u128::MAX),
                    }),
                }
            }
        }

        impl FromStr for $name {
            type Err = IdentifierError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let value = s.parse::<u128>().map_err(|_| IdentifierError::Parse {
                    type_name: stringify!($name),
                    input: s.to_owned(),
                })?;
                Self::try_from(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

define_identifier!(/// Identifier for chess positions within repertoires.
PositionId);
define_identifier!(/// Identifier for edges connecting positions in repertoires.
EdgeId);
define_identifier!(/// Identifier for moves that belong to an edge transition.
MoveId);
define_identifier!(/// Identifier for review cards stored in persistence.
CardId);

define_identifier!(/// Identifier for learners/owners of review cards.
LearnerId);

define_identifier!(/// Identifier for unlock workflow records.
UnlockId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_format_includes_type_name() {
        let id = EdgeId::from(12_u64);
        assert_eq!(id.to_string(), "EdgeId(12)");
    }

    #[test]
    fn parsing_identifier_from_string_round_trips() {
        let original = CardId::from(987_u64);
        let parsed: CardId = "987".parse().expect("parse identifier");
        assert_eq!(parsed, original);
    }
}
