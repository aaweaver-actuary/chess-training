//! Type-safe identifier wrappers shared across review domain modules.

use core::fmt;

/// Errors that can occur when converting raw values into identifier wrappers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdConversionError {
    /// Provided value exceeded the range representable by the identifier type.
    Overflow { value: u128 },
}

macro_rules! define_id {
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident;
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
                u64::try_from(value)
                    .map(Self::new)
                    .map_err(|_| IdConversionError::Overflow { value })
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
    pub struct PositionId;
);
define_id!(
    pub struct EdgeId;
);
define_id!(
    pub struct MoveId;
);
define_id!(
    pub struct CardId;
);
define_id!(
    pub struct LearnerId;
);
define_id!(
    pub struct TacticId;
);
define_id!(
    pub struct UnlockId;
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

        let tactic = TacticId::default();
        assert_eq!(tactic.get(), 0);
    }
}
