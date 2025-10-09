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
        if self.positions_read()?.contains_key(&id) {
            Ok(())
        } else {
            Err(StoreError::MissingPosition { id })
        }
    }

    fn ensure_edge_exists(&self, id: u64) -> Result<(), StoreError> {
        if self.edges_read()?.contains_key(&id) {
            Ok(())
        } else {
            Err(StoreError::MissingEdge { id })
        }
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
        insert_unlock_or_error(&mut unlocks, unlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::panic::catch_unwind;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn poisoned_locks_surface_as_store_errors() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        let result = catch_unwind(|| {
            let _guard = store.positions.write().unwrap();
            panic!("poison");
        });
        assert!(result.is_err());

        let position = ChessPosition::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .expect("valid FEN");
        let err = store.upsert_position(position).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "positions"));
    }

    #[test]
    fn ensure_position_exists_surfaces_missing_positions() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let err = store.ensure_position_exists(42).unwrap_err();
        assert!(matches!(err, StoreError::MissingPosition { id } if id == 42));
    }

    #[test]
    fn ensure_edge_exists_surfaces_missing_edges() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let err = store.ensure_edge_exists(24).unwrap_err();
        assert!(matches!(err, StoreError::MissingEdge { id } if id == 24));
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
}
