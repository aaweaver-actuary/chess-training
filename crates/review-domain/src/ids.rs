//! Type-safe identifier wrappers shared across review domain modules.

use core::fmt;

/// Identifies which strongly typed identifier failed to convert.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdKind {
    /// Identifier for stored chess positions.
    Position,
    /// Identifier for directed edges between positions.
    Edge,
    /// Identifier for individual moves inside an opening tree.
    Move,
    /// Identifier for persisted review cards.
    Card,
    /// Identifier for a learner using the training platform.
    Learner,
    /// Identifier for unlock records associated with learners.
    Unlock,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Position => "position",
            Self::Edge => "edge",
            Self::Move => "move",
            Self::Card => "card",
            Self::Learner => "learner",
            Self::Unlock => "unlock",
        };
        f.write_str(label)
    }
}

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

#[cfg(feature = "std")]
impl std::error::Error for IdConversionError {}

macro_rules! define_id {
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident;
        kind: $kind:ident;
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

        impl TryFrom<u128> for $name {
            type Error = IdConversionError;

            fn try_from(value: u128) -> Result<Self, Self::Error> {
                if value > u128::from(u64::MAX) {
                    return Err(IdConversionError::Overflow {
                        kind: IdKind::$kind,
                        value,
                        max: u64::MAX,
                    });
                }

                Ok(Self::new(value as u64))
            }
        }

        impl TryFrom<i128> for $name {
            type Error = IdConversionError;

            fn try_from(value: i128) -> Result<Self, Self::Error> {
                let value = u128::try_from(value).map_err(|_| IdConversionError::Negative {
                    kind: IdKind::$kind,
                    value,
                })?;

                Self::try_from(value)
            }
        }

        impl TryFrom<i64> for $name {
            type Error = IdConversionError;

            fn try_from(value: i64) -> Result<Self, Self::Error> {
                Self::try_from(i128::from(value))
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
    /// Strongly typed identifier for persisted chess positions.
    ///
    /// ```
    /// use review_domain::ids::{IdConversionError, IdKind, PositionId};
    ///
    /// let id = PositionId::try_from(42_u128).unwrap();
    /// assert_eq!(id.get(), 42);
    ///
    /// let overflow = PositionId::try_from(u128::from(u64::MAX) + 1);
    /// assert!(matches!(
    ///     overflow,
    ///     Err(IdConversionError::Overflow { kind, value, max })
    ///         if kind == IdKind::Position && value == u128::from(u64::MAX) + 1 && max == u64::MAX
    /// ));
    ///
    /// let negative = PositionId::try_from(-1_i64);
    /// assert!(matches!(
    ///     negative,
    ///     Err(IdConversionError::Negative { kind, value })
    ///         if kind == IdKind::Position && value == -1
    /// ));
    /// ```
    pub struct PositionId;
    kind: Position;
);
define_id!(
    /// Strongly typed identifier for directed edges between positions.
    ///
    /// ```
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
    pub struct EdgeId;
    kind: Edge;
);
define_id!(
    /// Strongly typed identifier for specific moves in an opening tree.
    ///
    /// ```
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
    pub struct MoveId;
    kind: Move;
);
define_id!(
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
    pub struct CardId;
    kind: Card;
);
define_id!(
    /// Strongly typed identifier for platform learners.
    ///
    /// ```
    /// use review_domain::ids::{IdConversionError, IdKind, LearnerId};
    ///
    /// let id = LearnerId::try_from(5_u128).unwrap();
    /// assert_eq!(id.get(), 5);
    ///
    /// let overflow = LearnerId::try_from(u128::from(u64::MAX) + 1);
    /// assert!(matches!(
    ///     overflow,
    ///     Err(IdConversionError::Overflow { kind, value, max })
    ///         if kind == IdKind::Learner && value == u128::from(u64::MAX) + 1 && max == u64::MAX
    /// ));
    ///
    /// let negative = LearnerId::try_from(-1_i64);
    /// assert!(matches!(
    ///     negative,
    ///     Err(IdConversionError::Negative { kind, value })
    ///         if kind == IdKind::Learner && value == -1
    /// ));
    /// ```
    pub struct LearnerId;
    kind: Learner;
);
define_id!(
    /// Strongly typed identifier for unlock records tied to learners.
    ///
    /// ```
    /// use review_domain::ids::{IdConversionError, IdKind, UnlockId};
    ///
    /// let id = UnlockId::try_from(3_u128).unwrap();
    /// assert_eq!(id.get(), 3);
    ///
    /// let overflow = UnlockId::try_from(u128::from(u64::MAX) + 1);
    /// assert!(matches!(
    ///     overflow,
    ///     Err(IdConversionError::Overflow { kind, value, max })
    ///         if kind == IdKind::Unlock && value == u128::from(u64::MAX) + 1 && max == u64::MAX
    /// ));
    ///
    /// let negative = UnlockId::try_from(-1_i64);
    /// assert!(matches!(
    ///     negative,
    ///     Err(IdConversionError::Negative { kind, value })
    ///         if kind == IdKind::Unlock && value == -1
    /// ));
    /// ```
    pub struct UnlockId;
    kind: Unlock;
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

    #[test]
    fn try_from_u128_succeeds_within_range() {
        assert_eq!(PositionId::try_from(1_u128).unwrap().get(), 1);
        assert_eq!(EdgeId::try_from(2_u128).unwrap().get(), 2);
        assert_eq!(MoveId::try_from(3_u128).unwrap().get(), 3);
        assert_eq!(CardId::try_from(4_u128).unwrap().get(), 4);
        assert_eq!(LearnerId::try_from(5_u128).unwrap().get(), 5);
        assert_eq!(UnlockId::try_from(6_u128).unwrap().get(), 6);
    }

    #[test]
    fn try_from_u128_reports_overflow() {
        let overflow_value = u128::from(u64::MAX) + 1;

        assert_eq!(
            PositionId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Position,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            EdgeId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Edge,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            MoveId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Move,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            CardId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Card,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            LearnerId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Learner,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            UnlockId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Unlock,
                value: overflow_value,
                max: u64::MAX,
            }
        );
    }

    #[test]
    fn try_from_i64_reports_negative_values() {
        assert_eq!(
            PositionId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Position,
                value: -1,
            }
        );
        assert_eq!(
            EdgeId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Edge,
                value: -1,
            }
        );
        assert_eq!(
            MoveId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Move,
                value: -1,
            }
        );
        assert_eq!(
            CardId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Card,
                value: -1,
            }
        );
        assert_eq!(
            LearnerId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Learner,
                value: -1,
            }
        );
        assert_eq!(
            UnlockId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Unlock,
                value: -1,
            }
        );
    }
}
