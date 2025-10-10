use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::chess_position::ChessPosition;
use crate::store::StoreError;

pub(super) fn canonicalize_position_for_storage(
    position: ChessPosition,
) -> Result<ChessPosition, StoreError> {
    ChessPosition::new(position.fen, position.ply).map_err(StoreError::from)
}

pub(super) fn store_canonical_position(
    positions: &mut HashMap<u64, ChessPosition>,
    canonical: ChessPosition,
) -> Result<ChessPosition, StoreError> {
    match positions.entry(canonical.id) {
        Entry::Occupied(entry) => {
            validate_position_collision(entry.get(), &canonical)?;
            Ok(entry.get().clone())
        }
        Entry::Vacant(slot) => {
            slot.insert(canonical.clone());
            Ok(canonical)
        }
    }
}

fn validate_position_collision(
    existing: &ChessPosition,
    canonical: &ChessPosition,
) -> Result<(), StoreError> {
    if existing.fen == canonical.fen {
        Ok(())
    } else {
        Err(StoreError::HashCollision { entity: "position" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::PositionError;

    fn is_invalid_position(err: &StoreError) -> bool {
        matches!(err, StoreError::InvalidPosition(_))
    }

    #[test]
    fn canonicalize_position_rejects_invalid_side_to_move() {
        let position = ChessPosition {
            id: 1,
            fen: "invalid fen".into(),
            side_to_move: 'w',
            ply: 0,
        };
        let err = canonicalize_position_for_storage(position).unwrap_err();
        assert!(is_invalid_position(&err));
    }

    #[test]
    fn invalid_position_helper_distinguishes_variants() {
        let err = StoreError::InvalidPosition(PositionError::MalformedFen);
        assert!(is_invalid_position(&err));

        let other = StoreError::InvalidGrade { grade: 5 };
        assert!(!is_invalid_position(&other));
    }

    #[test]
    fn canonicalize_position_recomputes_identifier() {
        let original = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let canonical = canonicalize_position_for_storage(original.clone()).unwrap();
        assert_eq!(canonical.id, original.id);
    }

    #[test]
    fn store_canonical_position_inserts_when_missing() {
        let mut positions = HashMap::new();
        let position = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let stored = store_canonical_position(&mut positions, position.clone()).unwrap();
        assert_eq!(stored, position);
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn store_canonical_position_returns_existing_on_collision() {
        let mut positions = HashMap::new();
        let first = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let second = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            10,
        )
        .unwrap();
        positions.insert(first.id, first.clone());

        let stored = store_canonical_position(&mut positions, second.clone()).unwrap();
        assert_eq!(stored, first);
    }

    #[test]
    fn validate_position_collision_errors_on_conflicting_fen() {
        let mut positions = HashMap::new();
        let first = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        positions.insert(first.id, first.clone());
        let mut conflicting = first.clone();
        conflicting.fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2".into();

        let err = store_canonical_position(&mut positions, conflicting).unwrap_err();
        assert!(matches!(err, StoreError::HashCollision { entity } if entity == "position"));
    }
}
