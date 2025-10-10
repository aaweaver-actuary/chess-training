//! In-memory implementation of the [`CardStore`](crate::store::CardStore) trait organized by
//! storage concern for readability.

use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use chrono::NaiveDate;

use crate::chess_position::ChessPosition;
use crate::config::StorageConfig;
use crate::model::{
    Card, Edge, EdgeInput, ReviewRequest, StoredCardState, UnlockRecord, card_id_for_opening,
};
use crate::store::{CardStore, StoreError};

mod cards;
mod edges;
mod position_helpers;
mod reviews;
mod unlocks;

use cards::{borrow_card_for_review, collect_due_cards_for_owner, store_opening_card};
use edges::store_canonical_edge;
use position_helpers::{canonicalize_position_for_storage, store_canonical_position};
use reviews::apply_review;
use unlocks::insert_unlock_or_error;

/// Thread-safe in-memory reference implementation of the storage trait.
#[derive(Debug)]
pub struct InMemoryCardStore {
    _config: StorageConfig,
    positions: RwLock<HashMap<u64, ChessPosition>>,
    edges: RwLock<HashMap<u64, Edge>>,
    cards: RwLock<HashMap<u64, Card>>,
    unlocks: RwLock<HashSet<UnlockRecord>>,
}

impl InMemoryCardStore {
    /// Construct a new [`InMemoryCardStore`] with the provided [`StorageConfig`].
    #[must_use]
    pub fn new(config: StorageConfig) -> Self {
        Self {
            _config: config,
            positions: RwLock::new(HashMap::new()),
            edges: RwLock::new(HashMap::new()),
            cards: RwLock::new(HashMap::new()),
            unlocks: RwLock::new(HashSet::new()),
        }
    }

    /// Number of unique positions currently stored. Useful for tests.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::PoisonedLock`] when the underlying position lock is poisoned.
    #[must_use = "handle potential store errors when counting positions"]
    pub fn position_count(&self) -> Result<usize, StoreError> {
        Ok(self.positions_read()?.len())
    }

    fn positions_read(
        &self,
    ) -> Result<RwLockReadGuard<'_, HashMap<u64, ChessPosition>>, StoreError> {
        self.positions.read().map_err(|_| StoreError::PoisonedLock {
            resource: "positions",
        })
    }

    fn positions_write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, HashMap<u64, ChessPosition>>, StoreError> {
        self.positions
            .write()
            .map_err(|_| StoreError::PoisonedLock {
                resource: "positions",
            })
    }

    fn edges_read(&self) -> Result<RwLockReadGuard<'_, HashMap<u64, Edge>>, StoreError> {
        self.edges
            .read()
            .map_err(|_| StoreError::PoisonedLock { resource: "edges" })
    }

    fn edges_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<u64, Edge>>, StoreError> {
        self.edges
            .write()
            .map_err(|_| StoreError::PoisonedLock { resource: "edges" })
    }

    fn cards_read(&self) -> Result<RwLockReadGuard<'_, HashMap<u64, Card>>, StoreError> {
        self.cards
            .read()
            .map_err(|_| StoreError::PoisonedLock { resource: "cards" })
    }

    fn cards_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<u64, Card>>, StoreError> {
        self.cards
            .write()
            .map_err(|_| StoreError::PoisonedLock { resource: "cards" })
    }

    fn unlocks_write(&self) -> Result<RwLockWriteGuard<'_, HashSet<UnlockRecord>>, StoreError> {
        self.unlocks.write().map_err(|_| StoreError::PoisonedLock {
            resource: "unlocks",
        })
    }

    fn ensure_position_exists(&self, id: u64) -> Result<(), StoreError> {
        if !self.positions_read()?.contains_key(&id) {
            return Err(StoreError::MissingPosition { id });
        }
        Ok(())
    }

    fn ensure_edge_exists(&self, id: u64) -> Result<(), StoreError> {
        if !self.edges_read()?.contains_key(&id) {
            return Err(StoreError::MissingEdge { id });
        }
        Ok(())
    }
}

impl CardStore for InMemoryCardStore {
    fn upsert_position(&self, position: ChessPosition) -> Result<ChessPosition, StoreError> {
        let canonical = canonicalize_position_for_storage(position)?;
        let mut positions = self.positions_write()?;
        store_canonical_position(&mut positions, canonical)
    }

    fn upsert_edge(&self, edge: EdgeInput) -> Result<Edge, StoreError> {
        self.ensure_position_exists(edge.parent_id)?;
        self.ensure_position_exists(edge.child_id)?;
        let canonical = edge.into_edge();
        let mut edges = self.edges_write()?;
        store_canonical_edge(&mut edges, canonical)
    }

    fn create_opening_card(
        &self,
        owner_id: &str,
        edge: &Edge,
        state: StoredCardState,
    ) -> Result<Card, StoreError> {
        self.ensure_edge_exists(edge.id)?;
        let card_id = card_id_for_opening(owner_id, edge.id);
        let mut cards = self.cards_write()?;
        store_opening_card(&mut cards, owner_id, edge, state, card_id)
    }

    fn fetch_due_cards(&self, owner_id: &str, as_of: NaiveDate) -> Result<Vec<Card>, StoreError> {
        let cards = self.cards_read()?;
        Ok(collect_due_cards_for_owner(&cards, owner_id, as_of))
    }

    fn record_review(&self, review: ReviewRequest) -> Result<Card, StoreError> {
        let mut cards = self.cards_write()?;
        let card = borrow_card_for_review(&mut cards, &review)?;
        apply_review(&mut card.state, &review)?;
        Ok(card.clone())
    }

    fn record_unlock(&self, unlock: UnlockRecord) -> Result<(), StoreError> {
        let mut unlocks = self.unlocks_write()?;
        insert_unlock_or_error(&mut unlocks, &unlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UnlockDetail;
    use chrono::NaiveDate;
    use std::sync::RwLock;
    use std::thread;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn start_position() -> ChessPosition {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        ChessPosition {
            id: crate::hash64(&[fen.as_bytes()]),
            fen: fen.into(),
            side_to_move: 'w',
            ply: 0,
        }
    }

    fn poison_write_lock<T>(lock: &RwLock<T>)
    where
        T: Send + Sync,
    {
        thread::scope(|scope| {
            let success = scope.spawn(|| {
                let _guard = lock.write().unwrap();
            });
            assert!(success.join().is_ok());

            let failure = scope.spawn(|| {
                let _guard = lock.write().unwrap();
                panic!("poison lock");
            });
            assert!(failure.join().is_err());
        });
    }

    #[test]
    fn poisoned_locks_surface_as_store_errors() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        poison_write_lock(&store.positions);

        let position = start_position();
        let err = store.upsert_position(position).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "positions"));
    }

    #[test]
    fn position_count_reports_poisoned_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        poison_write_lock(&store.positions);

        let err = store.position_count().unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "positions"));
    }

    #[test]
    fn position_count_reports_stored_positions() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        assert_eq!(store.position_count().unwrap(), 0);

        let position = start_position();
        store.upsert_position(position).unwrap();
        assert_eq!(store.position_count().unwrap(), 1);
    }

    #[test]
    fn ensure_position_exists_surfaces_missing_positions() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let err = store.ensure_position_exists(42).unwrap_err();
        assert!(matches!(err, StoreError::MissingPosition { id } if id == 42));
    }

    #[test]
    fn ensure_position_exists_reports_poisoned_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        poison_write_lock(&store.positions);

        let err = store.ensure_position_exists(1).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "positions"));
    }

    #[test]
    fn upsert_position_rejects_invalid_positions() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let invalid = ChessPosition {
            id: 99,
            fen: "invalid fen".into(),
            side_to_move: 'w',
            ply: 0,
        };
        let err = store.upsert_position(invalid).unwrap_err();
        match err {
            StoreError::InvalidPosition(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ensure_position_exists_accepts_existing_positions() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let position = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        store.upsert_position(position.clone()).unwrap();
        assert!(store.ensure_position_exists(position.id).is_ok());
    }

    #[test]
    fn ensure_edge_exists_surfaces_missing_edges() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let err = store.ensure_edge_exists(24).unwrap_err();
        assert!(matches!(err, StoreError::MissingEdge { id } if id == 24));
    }

    #[test]
    fn ensure_edge_exists_reports_poisoned_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        poison_write_lock(&store.edges);

        let err = store.ensure_edge_exists(1).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "edges"));
    }

    #[test]
    fn upsert_edge_requires_existing_positions() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let parent = start_position();
        let child = ChessPosition::new(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            1,
        )
        .unwrap();
        store.upsert_position(child.clone()).unwrap();

        let missing_parent = EdgeInput {
            parent_id: parent.id,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
            child_id: child.id,
        };
        let err = store.upsert_edge(missing_parent).unwrap_err();
        assert!(matches!(err, StoreError::MissingPosition { id } if id == parent.id));

        store.upsert_position(parent.clone()).unwrap();
        let missing_child = EdgeInput {
            parent_id: parent.id,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
            child_id: 999,
        };
        let err = store.upsert_edge(missing_child).unwrap_err();
        assert!(matches!(err, StoreError::MissingPosition { id } if id == 999));
    }

    #[test]
    fn upsert_edge_reports_poisoned_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        let parent = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let child = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            1,
        )
        .unwrap();
        store.upsert_position(parent.clone()).unwrap();
        store.upsert_position(child.clone()).unwrap();

        poison_write_lock(&store.edges);

        let edge = EdgeInput {
            parent_id: parent.id,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
            child_id: child.id,
        };
        let err = store.upsert_edge(edge).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "edges"));
    }

    #[test]
    fn record_review_updates_cards() {
        let position = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .unwrap();
        let store = InMemoryCardStore::new(StorageConfig::default());
        store.upsert_position(position.clone()).unwrap();
        let edge = store
            .upsert_edge(EdgeInput {
                parent_id: position.id,
                move_uci: "e2e4".into(),
                move_san: "e4".into(),
                child_id: position.id,
            })
            .unwrap();
        let state = StoredCardState::new(
            naive_date(2023, 1, 1),
            std::num::NonZeroU8::new(1).unwrap(),
            2.5,
        );
        let card = store
            .create_opening_card("owner", &edge, state.clone())
            .unwrap();
        let updated = store
            .record_review(ReviewRequest {
                card_id: card.id,
                reviewed_on: naive_date(2023, 1, 2),
                grade: 3,
            })
            .unwrap();
        assert_eq!(updated.id, card.id);
        assert!(updated.state.last_reviewed_on.is_some());
    }

    #[test]
    fn record_review_requires_existing_card() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let review = ReviewRequest {
            card_id: 999,
            reviewed_on: naive_date(2023, 1, 2),
            grade: 3,
        };
        let err = store.record_review(review).unwrap_err();
        assert!(matches!(err, StoreError::MissingCard { id } if id == 999));
    }

    #[test]
    fn fetch_due_cards_returns_due_entries() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let position = start_position();
        store.upsert_position(position.clone()).unwrap();
        let edge = store
            .upsert_edge(EdgeInput {
                parent_id: position.id,
                move_uci: "e2e4".into(),
                move_san: "e4".into(),
                child_id: position.id,
            })
            .unwrap();
        let state = StoredCardState::new(
            naive_date(2023, 1, 1),
            std::num::NonZeroU8::new(1).unwrap(),
            2.5,
        );
        store
            .create_opening_card("owner", &edge, state.clone())
            .unwrap();

        let cards = store
            .fetch_due_cards("owner", naive_date(2023, 1, 1))
            .unwrap();
        assert!(!cards.is_empty());
    }

    #[test]
    fn record_review_validates_grade() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let position = start_position();
        store.upsert_position(position.clone()).unwrap();
        let edge = store
            .upsert_edge(EdgeInput {
                parent_id: position.id,
                move_uci: "e2e4".into(),
                move_san: "e4".into(),
                child_id: position.id,
            })
            .unwrap();
        let state = StoredCardState::new(
            naive_date(2023, 1, 1),
            std::num::NonZeroU8::new(1).unwrap(),
            2.5,
        );
        let card = store
            .create_opening_card("owner", &edge, state)
            .expect("create card");

        let err = store
            .record_review(ReviewRequest {
                card_id: card.id,
                reviewed_on: naive_date(2023, 1, 2),
                grade: 9,
            })
            .unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 9));
    }

    #[test]
    fn record_unlock_reports_poisoned_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        poison_write_lock(&store.unlocks);

        let unlock = UnlockRecord {
            owner_id: "owner".to_string(),
            detail: UnlockDetail::new(42),
            unlocked_on: naive_date(2023, 1, 3),
        };
        let err = store.record_unlock(unlock).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "unlocks"));
    }

    #[test]
    fn record_unlock_stores_entry() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let unlock = UnlockRecord {
            owner_id: "owner".to_string(),
            detail: UnlockDetail::new(7),
            unlocked_on: naive_date(2023, 1, 2),
        };
        store.record_unlock(unlock.clone()).unwrap();

        let unlocks = store.unlocks.read().unwrap();
        assert!(unlocks.contains(&unlock));
    }

    #[test]
    fn create_opening_card_requires_existing_edge() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let edge = Edge::new(7, 1, 2, "e2e4", "e4");
        let state = StoredCardState::new(
            naive_date(2023, 1, 1),
            std::num::NonZeroU8::new(1).unwrap(),
            2.5,
        );
        let err = store
            .create_opening_card("owner", &edge, state)
            .unwrap_err();
        assert!(matches!(err, StoreError::MissingEdge { id } if id == 7));
    }

    #[test]
    fn create_opening_card_reports_poisoned_cards_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let position = start_position();
        store.upsert_position(position.clone()).unwrap();
        let edge = store
            .upsert_edge(EdgeInput {
                parent_id: position.id,
                move_uci: "e2e4".into(),
                move_san: "e4".into(),
                child_id: position.id,
            })
            .unwrap();

        poison_write_lock(&store.cards);

        let state = StoredCardState::new(
            naive_date(2023, 1, 1),
            std::num::NonZeroU8::new(1).unwrap(),
            2.5,
        );
        let err = store
            .create_opening_card("owner", &edge, state)
            .unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "cards"));
    }

    #[test]
    fn fetch_due_cards_reports_poisoned_cards_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        poison_write_lock(&store.cards);

        let err = store
            .fetch_due_cards("owner", naive_date(2023, 1, 1))
            .unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "cards"));
    }

    #[test]
    fn record_review_reports_poisoned_cards_lock() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let position = start_position();
        store.upsert_position(position.clone()).unwrap();
        store.upsert_position(position.clone()).unwrap();
        let edge = store
            .upsert_edge(EdgeInput {
                parent_id: position.id,
                move_uci: "e2e4".into(),
                move_san: "e4".into(),
                child_id: position.id,
            })
            .unwrap();
        let state = StoredCardState::new(
            naive_date(2023, 1, 1),
            std::num::NonZeroU8::new(1).unwrap(),
            2.5,
        );
        let card = store
            .create_opening_card("owner", &edge, state)
            .expect("create card");

        poison_write_lock(&store.cards);

        let err = store
            .record_review(ReviewRequest {
                card_id: card.id,
                reviewed_on: naive_date(2023, 1, 2),
                grade: 3,
            })
            .unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "cards"));
    }
}
