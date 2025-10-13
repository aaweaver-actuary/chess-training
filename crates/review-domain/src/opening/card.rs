//! Payload carried by opening review cards.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::ids::EdgeId;

/// Payload carried by opening review cards.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpeningCard {
    /// Identifier of the reviewed opening edge.
    pub edge_id: EdgeId,
}

impl OpeningCard {
    /// Creates a new `OpeningCard` payload.
    #[must_use]
    pub fn new(edge_id: EdgeId) -> Self {
        Self { edge_id }
    }
}

#[cfg(test)]
mod tests {
    use crate::ids::EdgeId;

    use super::OpeningCard;

    #[test]
    fn constructor_sets_fields() {
        let card = OpeningCard::new(EdgeId::new(42));
        assert_eq!(card.edge_id, EdgeId::new(42));
    }

    #[test]
    fn opening_card_is_copy() {
        fn assert_impl_copy<T: Copy>() {}

        assert_impl_copy::<OpeningCard>();
    }
}
