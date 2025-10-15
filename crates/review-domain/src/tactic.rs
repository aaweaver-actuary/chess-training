//! Shared tactic-specific data structures.

/// Payload carried by tactic review cards.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TacticCard {
    /// Identifier of the reviewed tactic.
    pub tactic_id: crate::TacticId,
}

impl TacticCard {
    /// Creates a new `TacticCard` payload.
    #[must_use]
    pub const fn new(tactic_id: crate::TacticId) -> Self {
        Self { tactic_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tactic_card_constructor_sets_fields() {
        let card = TacticCard::new(crate::TacticId::new(99));
        assert_eq!(card.tactic_id, crate::TacticId::new(99));
    }

    #[test]
    fn tactic_card_is_copy() {
        fn assert_impl_copy<T: Copy>() {}

        assert_impl_copy::<TacticCard>();
    }
}
