//! Payload carried by opening review cards.

/// Payload carried by opening review cards.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OpeningCard {
    /// Identifier of the reviewed opening edge.
    pub edge_id: u64,
}

impl OpeningCard {
    /// Creates a new `OpeningCard` payload.
    #[must_use]
    pub fn new(edge_id: u64) -> Self {
        Self { edge_id }
    }
}

#[cfg(test)]
mod tests {
    use super::OpeningCard;

    #[test]
    fn constructor_sets_fields() {
        let card = OpeningCard::new(42);
        assert_eq!(card.edge_id, 42);
    }

    #[test]
    fn opening_card_is_copy() {
        fn assert_impl_copy<T: Copy>() {}

        assert_impl_copy::<OpeningCard>();
    }
}
