#[cfg(test)]
mod coverage_minimal {
    use super::*;

    #[test]
    fn covers_new_constructor() {
        use crate::ids::{EdgeId, PositionId};

        let mv = RepertoireMove::new(
            EdgeId::new(42),
            PositionId::new(100),
            PositionId::new(101),
            "e2e4",
            "e4",
        );
        assert_eq!(mv.edge_id.get(), 42);
        assert_eq!(mv.parent_id.get(), 100);
        assert_eq!(mv.child_id.get(), 101);
        assert_eq!(mv.move_uci, "e2e4");
        assert_eq!(mv.move_san, "e4");
    }
}
use crate::ids::{EdgeId, PositionId};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single move stored within an opening repertoire.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RepertoireMove {
    /// Identifier of the originating position.
    pub parent_id: PositionId,
    /// Identifier of the resulting position.
    pub child_id: PositionId,
    /// Deterministic identifier of the represented opening edge.
    pub edge_id: EdgeId,
    /// Move encoded in UCI notation.
    pub move_uci: String,
    /// Move encoded in SAN notation.
    pub move_san: String,
}

impl RepertoireMove {
    /// Builds a new [`RepertoireMove`] from the constituent identifiers and move notation.
    #[must_use]
    pub fn new(
        edge_id: EdgeId,
        parent_id: PositionId,
        child_id: PositionId,
        move_uci: impl Into<String>,
        move_san: impl Into<String>,
    ) -> Self {
        Self {
            edge_id,
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
    fn test_repertoire_move_creation() {
        use crate::ids::{EdgeId, PositionId};

        let move_entry = RepertoireMove::new(
            EdgeId::new(1),
            PositionId::new(2),
            PositionId::new(3),
            "e2e4",
            "1. e4",
        );
        assert_eq!(move_entry.edge_id.get(), 1);
        assert_eq!(move_entry.parent_id.get(), 2);
        assert_eq!(move_entry.child_id.get(), 3);
        assert_eq!(move_entry.move_uci, "e2e4");
        assert_eq!(move_entry.move_san, "1. e4");

        let move_entry2 = RepertoireMove {
            parent_id: PositionId::new(2),
            child_id: PositionId::new(3),
            edge_id: EdgeId::new(1),
            move_uci: "e2e4".to_string(),
            move_san: "1. e4".to_string(),
        };

        assert_eq!(move_entry, move_entry2);
    }
}
