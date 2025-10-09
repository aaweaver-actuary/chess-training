//! Shared opening-specific data structures.

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

/// Directed edge in an opening tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpeningEdge {
    /// Deterministic identifier for the edge.
    pub id: u64,
    /// Identifier of the parent position.
    pub parent_id: u64,
    /// Identifier of the child position.
    pub child_id: u64,
    /// Move in UCI notation.
    pub move_uci: String,
    /// Move in SAN notation.
    pub move_san: String,
}

impl OpeningEdge {
    /// Builds a new opening edge.
    #[must_use]
    pub fn new(
        id: u64,
        parent_id: u64,
        child_id: u64,
        move_uci: impl Into<String>,
        move_san: impl Into<String>,
    ) -> Self {
        Self {
            id,
            parent_id,
            child_id,
            move_uci: move_uci.into(),
            move_san: move_san.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_card_constructor_sets_fields() {
        let card = OpeningCard::new(42);
        assert_eq!(card.edge_id, 42);
    }

    #[test]
    fn opening_card_is_copy() {
        fn assert_impl_copy<T: Copy>() {}

        assert_impl_copy::<OpeningCard>();
    }

    #[test]
    fn opening_edge_constructor_copies_inputs() {
        let edge = OpeningEdge::new(1, 2, 3, "e2e4", String::from("e4"));
        assert_eq!(edge.id, 1);
        assert_eq!(edge.parent_id, 2);
        assert_eq!(edge.child_id, 3);
        assert_eq!(edge.move_uci, "e2e4");
        assert_eq!(edge.move_san, "e4");
    }
}
