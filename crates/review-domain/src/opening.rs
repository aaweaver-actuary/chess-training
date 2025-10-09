//! Shared opening-specific data structures.

/// Payload carried by opening review cards.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
