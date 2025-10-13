use std::{
    collections::{HashMap, HashSet},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use chrono::NaiveDate;

use crate::{
    ReviewCardStore, StoreError,
    chess_position::ChessPosition,
    config::StorageConfig,
    memory::{
        apply_review, borrow_card_for_review, canonicalize_position_for_storage,
        collect_due_cards_for_owner, insert_unlock_or_error, store_canonical_edge,
        store_canonical_position, store_opening_card,
    },
    model::{
        Card, CardMap, Edge, EdgeInput, EdgeMap, PositionMap, ReviewRequest, StoredCardState,
        UnlockRecord, UnlockSet, card_id_for_opening,
    },
};

/// Thread-safe in-memory reference implementation of the storage trait.
#[derive(Debug)]
pub struct InMemoryCardStore {
    _config: StorageConfig,
    positions: RwLock<PositionMap>,
    edges: RwLock<EdgeMap>,
    cards: RwLock<CardMap>,
    unlocks: RwLock<UnlockSet>,
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

    fn positions_read(&self) -> Result<RwLockReadGuard<'_, PositionMap>, StoreError> {
        self.positions.read().map_err(|_| StoreError::PoisonedLock {
            resource: "positions",
        })
    }

    fn positions_write(&self) -> Result<RwLockWriteGuard<'_, PositionMap>, StoreError> {
        self.positions
            .write()
            .map_err(|_| StoreError::PoisonedLock {
                resource: "positions",
            })
    }

    fn edges_read(&self) -> Result<RwLockReadGuard<'_, EdgeMap>, StoreError> {
        self.edges
            .read()
            .map_err(|_| StoreError::PoisonedLock { resource: "edges" })
    }

    fn edges_write(&self) -> Result<RwLockWriteGuard<'_, EdgeMap>, StoreError> {
        self.edges
            .write()
            .map_err(|_| StoreError::PoisonedLock { resource: "edges" })
    }

    fn cards_read(&self) -> Result<RwLockReadGuard<'_, CardMap>, StoreError> {
        self.cards
            .read()
            .map_err(|_| StoreError::PoisonedLock { resource: "cards" })
    }

    fn cards_write(&self) -> Result<RwLockWriteGuard<'_, CardMap>, StoreError> {
        self.cards
            .write()
            .map_err(|_| StoreError::PoisonedLock { resource: "cards" })
    }

    fn unlocks_write(&self) -> Result<RwLockWriteGuard<'_, UnlockSet>, StoreError> {
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

impl ReviewCardStore for InMemoryCardStore {
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
impl InMemoryCardStore {
    pub(crate) fn positions_lock(&self) -> &RwLock<PositionMap> {
        &self.positions
    }

    pub(crate) fn edges_lock(&self) -> &RwLock<EdgeMap> {
        &self.edges
    }

    pub(crate) fn cards_lock(&self) -> &RwLock<CardMap> {
        &self.cards
    }

    pub(crate) fn unlocks_lock(&self) -> &RwLock<UnlockSet> {
        &self.unlocks
    }

    pub(crate) fn ensure_position_exists_for_test(&self, id: u64) -> Result<(), StoreError> {
        self.ensure_position_exists(id)
    }

    pub(crate) fn ensure_edge_exists_for_test(&self, id: u64) -> Result<(), StoreError> {
        self.ensure_edge_exists(id)
    }
}
