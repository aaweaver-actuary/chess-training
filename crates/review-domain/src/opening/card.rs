//! Payload carried by opening review cards and unlock records.

use crate::EdgeId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Shared handle referencing a specific opening edge.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpeningEdgeHandle {
    /// Identifier of the referenced opening edge.
    pub edge_id: EdgeId,
}

impl OpeningEdgeHandle {
    /// Creates a new handle for an opening edge.
    #[must_use]
    pub const fn new(edge_id: EdgeId) -> Self {
        Self { edge_id }
    }

    /// Returns the underlying [`EdgeId`].
    #[must_use]
    pub const fn edge_id(self) -> EdgeId {
        self.edge_id
    }
}

impl From<EdgeId> for OpeningEdgeHandle {
    fn from(edge_id: EdgeId) -> Self {
        Self::new(edge_id)
    }
}

impl From<OpeningEdgeHandle> for EdgeId {
    fn from(handle: OpeningEdgeHandle) -> Self {
        handle.edge_id
    }
}

/// Payload carried by opening review cards.
pub type OpeningCard = OpeningEdgeHandle;

#[cfg(test)]
mod tests {
    use super::OpeningCard;
    use crate::EdgeId;

    #[test]
    fn constructor_sets_fields() {
        let edge_id = EdgeId::new(42);
        let card = OpeningCard::new(edge_id);
        assert_eq!(card.edge_id, edge_id);
    }

    #[test]
    fn opening_card_is_copy() {
        fn assert_impl_copy<T: Copy>() {}

        assert_impl_copy::<OpeningCard>();
    }
}
