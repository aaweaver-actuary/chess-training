use super::IdKind;
use std::fmt;

/// Error raised when converting into a strongly typed identifier fails.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdConversionError {
    /// The provided unsigned value exceeded the `u64` range of the identifier.
    Overflow {
        /// The identifier that failed to convert.
        kind: IdKind,
        /// The value that exceeded the supported range.
        value: u128,
        /// The maximum supported value for the identifier.
        max: u64,
    },
    /// The provided signed value was negative.
    Negative {
        /// The identifier that failed to convert.
        kind: IdKind,
        /// The negative value supplied by the caller.
        value: i128,
    },
}

impl fmt::Display for IdConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow { kind, value, max } => {
                write!(
                    f,
                    "{kind} identifier overflow: {value} exceeds maximum {max}",
                )
            }
            Self::Negative { kind, value } => {
                write!(f, "{kind} identifier received negative value {value}")
            }
        }
    }
}

impl std::error::Error for IdConversionError {}
