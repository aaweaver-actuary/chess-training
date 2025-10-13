//! Type-safe identifier wrappers shared across review domain modules.

use core::{fmt, str::FromStr};
use thiserror::Error;

/// Identifier categories used when reporting conversion failures.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IdKind {
    Position,
    Edge,
    Move,
    Card,
    Learner,
    Unlock,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = self.type_name();
        f.write_str(label)
    }
}

impl IdKind {
    const fn type_name(self) -> &'static str {
        match self {
            Self::Position => "position",
            Self::Edge => "edge",
            Self::Move => "move",
            Self::Card => "card",
            Self::Learner => "learner",
            Self::Unlock => "unlock",
        }
    }
}

/// Errors produced when converting primitive values into identifier wrappers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdConversionError {
    /// Conversion overflowed the target identifier range.
    #[error("{kind} id overflow converting {value} (max {max})")]
    Overflow { kind: IdKind, value: u128, max: u64 },
    /// Conversion attempted to store a negative value in an unsigned identifier.
    #[error("{kind} id cannot represent negative value {value}")]
    Negative { kind: IdKind, value: i128 },
    /// Conversion failed because the input could not be parsed as an integer.
    #[error("{kind} id could not parse `{input}`")]
    InvalidFormat { kind: IdKind, input: String },
}

macro_rules! define_id {
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident => $kind:expr;
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        #[repr(transparent)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $vis struct $name(u64);

        impl $name {
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

            /// Consumes the identifier and returns the raw value.
            #[must_use]
            pub const fn into_inner(self) -> u64 {
                self.0
            }

            /// Returns the raw value by reference.
            #[must_use]
            pub const fn as_u64(&self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<&$name> for u64 {
            fn from(value: &$name) -> Self {
                value.0
            }
        }

        impl TryFrom<u128> for $name {
            type Error = IdConversionError;

            fn try_from(value: u128) -> Result<Self, Self::Error> {
                u64::try_from(value).map(Self::new).map_err(|_| {
                    IdConversionError::Overflow {
                        kind: $kind,
                        value,
                        max: u64::MAX,
                    }
                })
            }
        }

        impl TryFrom<i128> for $name {
            type Error = IdConversionError;

            fn try_from(value: i128) -> Result<Self, Self::Error> {
                if value < 0 {
                    return Err(IdConversionError::Negative {
                        kind: $kind,
                        value,
                    });
                }
                #[allow(clippy::cast_sign_loss)]
                Self::try_from(value as u128)
            }
        }

        impl TryFrom<i64> for $name {
            type Error = IdConversionError;

            fn try_from(value: i64) -> Result<Self, Self::Error> {
                Self::try_from(i128::from(value))
            }
        }

        impl FromStr for $name {
            type Err = IdConversionError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let value = s.parse::<u128>().map_err(|_| IdConversionError::InvalidFormat {
                    kind: $kind,
                    input: s.to_string(),
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

define_id!(
    pub struct PositionId => IdKind::Position;
);
define_id!(
    pub struct EdgeId => IdKind::Edge;
);
define_id!(
    pub struct MoveId => IdKind::Move;
);
define_id!(
    pub struct CardId => IdKind::Card;
);
define_id!(
    pub struct LearnerId => IdKind::Learner;
);
define_id!(
    pub struct UnlockId => IdKind::Unlock;
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_representation_matches_display() {
        let position = PositionId::new(91);
        assert_eq!(format!("{position}"), format!("{position:?}"));
    }

    #[test]
    fn default_is_zero() {
        let edge = EdgeId::default();
        assert_eq!(edge.get(), 0);
    }
}
