//! Strongly typed identifier wrappers used across the review domain crate.
//!
//! These newtypes replace direct usage of raw `u64` identifiers and provide
//! conversions plus serde support so existing serialization formats remain
//! stable.

use core::fmt;
use std::convert::TryFrom;
use std::num::TryFromIntError;

macro_rules! define_id_type {
    ($(#[$meta:meta])* $name:ident, $doc:expr) => {
        $(#[$meta])*
        #[doc = $doc]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        #[repr(transparent)]
        pub struct $name(u64);

        impl $name {
            /// Creates a new identifier wrapper from the provided raw value.
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the inner `u64` representation of the identifier.
            #[must_use]
            pub const fn into_inner(self) -> u64 {
                self.0
            }

            /// Returns the inner `u64` representation without moving the value.
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

        impl TryFrom<i64> for $name {
            type Error = TryFromIntError;

            fn try_from(value: i64) -> Result<Self, Self::Error> {
                u64::try_from(value).map(Self)
            }
        }

        impl TryFrom<i128> for $name {
            type Error = TryFromIntError;

            fn try_from(value: i128) -> Result<Self, Self::Error> {
                u64::try_from(value).map(Self)
            }
        }

        impl TryFrom<u128> for $name {
            type Error = TryFromIntError;

            fn try_from(value: u128) -> Result<Self, Self::Error> {
                u64::try_from(value).map(Self)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0).finish()
            }
        }
    };
}

define_id_type!(
    /// Identifier assigned to a chess position derived from deterministic hashing.
    PositionId,
    "Identifier assigned to a chess position derived from deterministic hashing."
);

define_id_type!(
    /// Identifier assigned to a repertoire edge connecting two positions.
    EdgeId,
    "Identifier assigned to a repertoire edge connecting two positions."
);

define_id_type!(
    /// Identifier representing a move applied to a position.
    MoveId,
    "Identifier representing a move applied to a position."
);

define_id_type!(
    /// Identifier assigned to a review card instance.
    CardId,
    "Identifier assigned to a review card instance."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_inner_value() {
        let id = PositionId::from(99_u64);
        assert_eq!(id.to_string(), "99");
        assert_eq!(format!("{id}"), "99");
        assert_eq!(format!("{id:?}"), "PositionId(99)");
    }

    #[test]
    fn try_from_i64_rejects_negative_values() {
        let err = PositionId::try_from(-1_i64).expect_err("negative should fail");
        assert_eq!(
            err.to_string(),
            "out of range integral type conversion attempted"
        );
    }

    #[test]
    fn conversion_helpers_return_inner_values() {
        let id = EdgeId::from(5_u64);
        assert_eq!(id.as_u64(), 5);
        assert_eq!(id.into_inner(), 5);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_preserves_value() {
        let id = CardId::from(123_u64);
        let json = serde_json::to_string(&id).expect("serialize");
        assert_eq!(json, "123");
        let back: CardId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, id);
    }
}
