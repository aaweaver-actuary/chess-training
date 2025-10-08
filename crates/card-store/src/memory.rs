//! In-memory implementation of the [`CardStore`](crate::store::CardStore) trait.

use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::num::NonZeroU8;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::StorageConfig;
use crate::model::{
    Card, CardKind, CardState, Edge, EdgeInput, Position, ReviewRequest, UnlockRecord,
    card_id_for_opening,
};
use crate::store::{CardStore, StoreError};
use chrono::{Duration, NaiveDate};

/// Thread-safe in-memory reference implementation of the storage trait.
#[derive(Debug)]
pub struct InMemoryCardStore {
    _config: StorageConfig,
    positions: RwLock<HashMap<u64, Position>>,
    edges: RwLock<HashMap<u64, Edge>>,
    cards: RwLock<HashMap<u64, Card>>,
    unlocks: RwLock<HashSet<UnlockRecord>>,
}

impl InMemoryCardStore {
    /// Construct a new [`InMemoryCardStore`] with the provided [`StorageConfig`].
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
    pub fn position_count(&self) -> Result<usize, StoreError> {
        Ok(self.positions_read()?.len())
    }

    fn positions_read(&self) -> Result<RwLockReadGuard<'_, HashMap<u64, Position>>, StoreError> {
        self.positions.read().map_err(|_| StoreError::PoisonedLock {
            resource: "positions",
        })
    }

    fn positions_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<u64, Position>>, StoreError> {
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

    fn validate_grade(grade: u8) -> Result<(), StoreError> {
        if grade > 4 {
            Err(StoreError::InvalidGrade { grade })
        } else {
            Ok(())
        }
    }

    fn interval_after_grade(interval: NonZeroU8, grade: u8) -> NonZeroU8 {
        match grade {
            0 | 1 => NonZeroU8::new(1).unwrap(),
            2 => interval,
            3 => {
                let next = interval.get().saturating_add(1).min(u8::MAX);
                NonZeroU8::new(next).unwrap()
            }
            4 => {
                let doubled = interval.get().saturating_mul(2).clamp(1, u8::MAX);
                NonZeroU8::new(doubled).unwrap()
            }
            _ => unreachable!(),
        }
    }

    fn ease_delta_for_grade(grade: u8) -> f32 {
        match grade {
            0 => -0.3,
            1 => -0.15,
            2 => -0.05,
            3 => 0.0,
            4 => 0.15,
            _ => unreachable!(),
        }
    }

    fn update_streak(state: &mut CardState, grade: u8) {
        if grade >= 3 {
            state.consecutive_correct = state.consecutive_correct.saturating_add(1);
        } else {
            state.consecutive_correct = 0;
        }
    }

    fn apply_review(state: &mut CardState, review: &ReviewRequest) -> Result<(), StoreError> {
        let ReviewRequest {
            reviewed_on, grade, ..
        } = *review;
        Self::validate_grade(grade)?;

        let interval = state.interval;
        state.interval = Self::interval_after_grade(interval, grade);

        let ease_delta = Self::ease_delta_for_grade(grade);
        state.ease_factor = (state.ease_factor + ease_delta).clamp(1.3, 2.8);

        Self::update_streak(state, grade);
        state.last_reviewed_on = Some(reviewed_on);
        state.due_on = reviewed_on + Duration::days(i64::from(state.interval.get()));
        Ok(())
    }
}

impl CardStore for InMemoryCardStore {
    fn upsert_position(&self, position: Position) -> Result<Position, StoreError> {
        let canonical = Position::new(position.fen, position.ply)?;
        let mut positions = self.positions_write()?;
        Ok(match positions.entry(canonical.id) {
            Entry::Occupied(entry) => {
                if entry.get().fen != canonical.fen {
                    return Err(StoreError::HashCollision { entity: "position" });
                }
                entry.get().clone()
            }
            Entry::Vacant(slot) => {
                slot.insert(canonical.clone());
                canonical
            }
        })
    }

    fn upsert_edge(&self, edge: EdgeInput) -> Result<Edge, StoreError> {
        self.ensure_position_exists(edge.parent_id)?;
        self.ensure_position_exists(edge.child_id)?;
        let canonical = Edge::from_input(edge);
        let mut edges = self.edges_write()?;
        Ok(match edges.entry(canonical.id) {
            Entry::Occupied(entry) => {
                let existing = entry.get();
                if existing.parent_id != canonical.parent_id
                    || existing.child_id != canonical.child_id
                    || existing.move_uci != canonical.move_uci
                {
                    return Err(StoreError::HashCollision { entity: "edge" });
                }
                existing.clone()
            }
            Entry::Vacant(slot) => {
                slot.insert(canonical.clone());
                canonical
            }
        })
    }

    fn create_opening_card(
        &self,
        owner_id: &str,
        edge: &Edge,
        state: CardState,
    ) -> Result<Card, StoreError> {
        self.ensure_edge_exists(edge.id)?;
        let card_id = card_id_for_opening(owner_id, edge.id);
        let mut cards = self.cards_write()?;
        Ok(match cards.entry(card_id) {
            Entry::Occupied(entry) => {
                let existing = entry.get();
                if existing.owner_id != owner_id
                    || !matches!(existing.kind, CardKind::Opening { edge_id } if edge_id == edge.id)
                {
                    return Err(StoreError::HashCollision { entity: "card" });
                }
                existing.clone()
            }
            Entry::Vacant(slot) => {
                let card = Card {
                    id: card_id,
                    owner_id: owner_id.to_string(),
                    kind: CardKind::Opening { edge_id: edge.id },
                    state,
                };
                slot.insert(card.clone());
                card
            }
        })
    }

    fn fetch_due_cards(&self, owner_id: &str, as_of: NaiveDate) -> Result<Vec<Card>, StoreError> {
        let cards = self.cards_read()?;
        let mut result: Vec<Card> = cards
            .values()
            .filter(|card| card.owner_id == owner_id && card.state.due_on <= as_of)
            .cloned()
            .collect();
        result.sort_by_key(|card| (card.state.due_on, card.id));
        Ok(result)
    }

    fn record_review(&self, review: ReviewRequest) -> Result<Card, StoreError> {
        let mut cards = self.cards_write()?;
        let card = cards
            .get_mut(&review.card_id)
            .ok_or(StoreError::MissingCard { id: review.card_id })?;
        Self::apply_review(&mut card.state, &review)?;
        Ok(card.clone())
    }

    fn record_unlock(&self, unlock: UnlockRecord) -> Result<(), StoreError> {
        let mut unlocks = self.unlocks_write()?;
        if !unlocks.insert(unlock.clone()) {
            return Err(StoreError::DuplicateUnlock {
                edge: unlock.edge_id,
                day: unlock.unlocked_on,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    #[test]
    fn poisoned_locks_surface_as_store_errors() {
        let store = InMemoryCardStore::new(StorageConfig::default());

        let result = catch_unwind(|| {
            let _guard = store.positions.write().unwrap();
            panic!("poison");
        });
        assert!(result.is_err());

        let position = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .expect("valid FEN");
        let err = store.upsert_position(position).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "positions"));
    }
}
