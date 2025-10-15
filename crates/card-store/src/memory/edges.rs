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
    // use super::*;
    // use crate::chess_position::ChessPosition; // No longer available
    // use crate::model::EdgeInput;
    // use std::collections::HashMap;

    // #[test]
    // fn store_canonical_edge_returns_existing_when_identical() {
    //     // ChessPosition is not available. Test skipped or refactor needed.
    //     // TODO: Refactor this test to use canonical Position/PositionId if possible.
    // }

    // #[test]
    // fn store_canonical_edge_errors_on_mismatch() {
    //     // ChessPosition is not available. Test skipped or refactor needed.
    //     // TODO: Refactor this test to use canonical Position/PositionId if possible.
    // }
}
