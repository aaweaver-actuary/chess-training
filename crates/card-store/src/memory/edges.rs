use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::model::Edge;
use crate::store::StoreError;

pub(super) fn store_canonical_edge(
    edges: &mut HashMap<u64, Edge>,
    canonical: Edge,
) -> Result<Edge, StoreError> {
    match edges.entry(canonical.id) {
        Entry::Occupied(entry) => {
            validate_edge_collision(entry.get(), &canonical)?;
            Ok(entry.get().clone())
        }
        Entry::Vacant(slot) => {
            slot.insert(canonical.clone());
            Ok(canonical)
        }
    }
}

fn validate_edge_collision(existing: &Edge, canonical: &Edge) -> Result<(), StoreError> {
    if existing.parent_id == canonical.parent_id
        && existing.child_id == canonical.child_id
        && existing.move_uci == canonical.move_uci
    {
        Ok(())
    } else {
        Err(StoreError::HashCollision { entity: "edge" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess_position::ChessPosition;
    use crate::model::EdgeInput;
    use std::collections::HashMap;

    #[test]
    fn store_canonical_edge_returns_existing_when_identical() {
        let parent = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let child = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            1,
        )
        .unwrap();
        let edge = Edge::from_input(EdgeInput {
            parent_id: parent.id,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
            child_id: child.id,
        });
        let mut edges = HashMap::new();
        edges.insert(edge.id, edge.clone());

        let stored = store_canonical_edge(&mut edges, edge.clone()).unwrap();
        assert_eq!(stored, edge);
    }

    #[test]
    fn store_canonical_edge_errors_on_mismatch() {
        let parent = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let child = ChessPosition::new(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2",
            2,
        )
        .unwrap();
        let first = Edge::from_input(EdgeInput {
            parent_id: parent.id,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
            child_id: child.id,
        });
        let mut edges = HashMap::new();
        edges.insert(first.id, first);

        let alternate_child = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            1,
        )
        .unwrap();
        let conflicting = Edge::from_input(EdgeInput {
            parent_id: parent.id,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
            child_id: alternate_child.id,
        });
        let err = store_canonical_edge(&mut edges, conflicting).unwrap_err();
        assert!(matches!(err, StoreError::HashCollision { entity } if entity == "edge"));
    }
}
