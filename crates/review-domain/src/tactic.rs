//! Shared tactic-specific data structures.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Payload carried by tactic review cards.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TacticCard {
    /// Identifier of the reviewed tactic.
    pub tactic_id: u64,
}

impl TacticCard {
    /// Creates a new `TacticCard` payload.
    #[must_use]
    pub fn new(tactic_id: u64) -> Self {
        Self { tactic_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tactic_card_constructor_sets_fields() {
        let card = TacticCard::new(99);
        assert_eq!(card.tactic_id, 99);
    }

    #[test]
    fn tactic_card_is_copy() {
        fn assert_impl_copy<T: Copy>() {}

        assert_impl_copy::<TacticCard>();
    }
}
