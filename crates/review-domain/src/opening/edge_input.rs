//! Input payload for inserting or updating an opening edge.

use crate::opening::OpeningEdge;
use crate::utils::hash_with_seed;

/// Input payload for inserting or updating an edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeInput {
    /// Parent position identifier.
    pub parent_id: u64,
    /// Move in UCI format.
    pub move_uci: String,
    /// Move in SAN format.
    pub move_san: String,
    /// Child position identifier.
    pub child_id: u64,
}

impl EdgeInput {
    /// Converts the input payload into a canonical [`OpeningEdge`].
    ///
    /// The canonical form computes a deterministic edge ID from the parent position and move,
    /// and returns an [`OpeningEdge`] with normalized fields.
    #[must_use]
    pub fn into_edge(self) -> OpeningEdge {
        let value_to_hash: String = format!("{}|{}", self.parent_id, self.move_uci);
        let id = hash_with_seed(&value_to_hash);
        OpeningEdge {
            id,
            parent_id: self.parent_id,
            child_id: self.child_id,
            move_uci: self.move_uci,
            move_san: self.move_san,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeInput;

    #[test]
    fn converts_to_edge() {
        let input = EdgeInput {
            parent_id: 1,
            move_uci: String::from("e2e4"),
            move_san: String::from("e4"),
            child_id: 2,
        };

        let edge = input.into_edge();

        assert_eq!(edge.parent_id, 1);
        assert_eq!(edge.child_id, 2);
        assert_eq!(edge.move_uci, "e2e4");
        assert_eq!(edge.move_san, "e4");
    }

    #[test]
    fn produces_same_id_for_identical_input() {
        let make_input = || EdgeInput {
            parent_id: 7,
            move_uci: String::from("g1f3"),
            move_san: String::from("Nf3"),
            child_id: 11,
        };

        let first = make_input().into_edge();
        let second = make_input().into_edge();

        assert_eq!(first.id, second.id);
    }

    #[test]
    fn id_changes_with_move() {
        let mut input = EdgeInput {
            parent_id: 3,
            move_uci: String::from("d2d4"),
            move_san: String::from("d4"),
            child_id: 4,
        };
        let first = input.clone().into_edge();
        input.move_uci = String::from("c2c4");
        input.move_san = String::from("c4");
        let second = input.into_edge();

        assert_ne!(first.id, second.id);
    }
}
