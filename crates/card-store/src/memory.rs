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

    fn canonicalize_position_for_storage(position: Position) -> Result<Position, StoreError> {
        Ok(Position::new(position.fen, position.ply)?)
    }

    fn store_canonical_position(
        positions: &mut HashMap<u64, Position>,
        canonical: Position,
    ) -> Result<Position, StoreError> {
        Ok(match positions.entry(canonical.id) {
            Entry::Occupied(entry) => {
                Self::validate_position_collision(entry.get(), &canonical)?;
                entry.get().clone()
            }
            Entry::Vacant(slot) => {
                slot.insert(canonical.clone());
                canonical
            }
        })
    }

    fn validate_position_collision(
        existing: &Position,
        canonical: &Position,
    ) -> Result<(), StoreError> {
        if existing.fen == canonical.fen {
            Ok(())
        } else {
            Err(StoreError::HashCollision { entity: "position" })
        }
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

    fn store_canonical_edge(
        edges: &mut HashMap<u64, Edge>,
        canonical: Edge,
    ) -> Result<Edge, StoreError> {
        Ok(match edges.entry(canonical.id) {
            Entry::Occupied(entry) => {
                Self::validate_edge_collision(entry.get(), &canonical)?;
                entry.get().clone()
            }
            Entry::Vacant(slot) => {
                slot.insert(canonical.clone());
                canonical
            }
        })
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

    fn store_opening_card(
        cards: &mut HashMap<u64, Card>,
        owner_id: &str,
        edge: &Edge,
        state: CardState,
        card_id: u64,
    ) -> Result<Card, StoreError> {
        Ok(match cards.entry(card_id) {
            Entry::Occupied(entry) => {
                Self::validate_existing_opening_card(entry.get(), owner_id, edge)?;
                entry.get().clone()
            }
            Entry::Vacant(slot) => {
                let card = Self::build_opening_card(owner_id, edge, state, card_id);
                slot.insert(card.clone());
                card
            }
        })
    }

    fn validate_existing_opening_card(
        card: &Card,
        owner_id: &str,
        edge: &Edge,
    ) -> Result<(), StoreError> {
        if card.owner_id == owner_id
            && matches!(card.kind, CardKind::Opening { edge_id } if edge_id == edge.id)
        {
            Ok(())
        } else {
            Err(StoreError::HashCollision { entity: "card" })
        }
    }

    fn build_opening_card(owner_id: &str, edge: &Edge, state: CardState, card_id: u64) -> Card {
        Card {
            id: card_id,
            owner_id: owner_id.to_string(),
            kind: CardKind::Opening { edge_id: edge.id },
            state,
        }
    }

    fn collect_due_cards_for_owner(
        cards: &HashMap<u64, Card>,
        owner_id: &str,
        as_of: NaiveDate,
    ) -> Vec<Card> {
        let mut result: Vec<Card> = cards
            .values()
            .filter(|card| card.owner_id == owner_id && card.state.due_on <= as_of)
            .cloned()
            .collect();
        result.sort_by_key(|card| (card.state.due_on, card.id));
        result
    }

    fn borrow_card_for_review<'a>(
        cards: &'a mut HashMap<u64, Card>,
        review: &ReviewRequest,
    ) -> Result<&'a mut Card, StoreError> {
        cards
            .get_mut(&review.card_id)
            .ok_or(StoreError::MissingCard { id: review.card_id })
    }

    fn insert_unlock_or_error(
        unlocks: &mut HashSet<UnlockRecord>,
        unlock: UnlockRecord,
    ) -> Result<(), StoreError> {
        if unlocks.insert(unlock.clone()) {
            Ok(())
        } else {
            Err(StoreError::DuplicateUnlock {
                edge: unlock.edge_id,
                day: unlock.unlocked_on,
            })
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
                let next = interval.get().saturating_add(1);
                NonZeroU8::new(next).unwrap()
            }
            4 => {
                let doubled = interval.get().saturating_mul(2);
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

    fn apply_review(state: &mut CardState, review: &ReviewRequest) -> Result<(), StoreError> {
        let transition = Self::derive_review_transition(state, review)?;
        Self::commit_review_transition(state, review.reviewed_on, transition);
        Ok(())
    }

    fn derive_review_transition(
        state: &CardState,
        review: &ReviewRequest,
    ) -> Result<ReviewTransition, StoreError> {
        Self::validate_grade(review.grade)?;
        let interval = Self::interval_after_grade(state.interval, review.grade);
        let ease = Self::ease_after_grade(state.ease_factor, review.grade);
        Self::finalize_transition(state, review, interval, ease)
    }

    fn ease_after_grade(current: f32, grade: u8) -> f32 {
        let delta = Self::ease_delta_for_grade(grade);
        (current + delta).clamp(1.3, 2.8)
    }

    fn finalize_transition(
        state: &CardState,
        review: &ReviewRequest,
        interval: NonZeroU8,
        ease: f32,
    ) -> Result<ReviewTransition, StoreError> {
        let streak = Self::next_streak(state.consecutive_correct, review.grade);
        let due_on = Self::due_date_for_review(review.reviewed_on, interval);
        Ok(ReviewTransition {
            interval,
            ease,
            streak,
            due_on,
        })
    }

    fn next_streak(current: u32, grade: u8) -> u32 {
        if grade >= 3 {
            current.saturating_add(1)
        } else {
            0
        }
    }

    fn due_date_for_review(reviewed_on: NaiveDate, interval: NonZeroU8) -> NaiveDate {
        reviewed_on + Duration::days(i64::from(interval.get()))
    }

    fn commit_review_transition(
        state: &mut CardState,
        reviewed_on: NaiveDate,
        transition: ReviewTransition,
    ) {
        state.interval = transition.interval;
        state.ease_factor = transition.ease;
        state.consecutive_correct = transition.streak;
        state.last_reviewed_on = Some(reviewed_on);
        state.due_on = transition.due_on;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ReviewTransition {
    interval: NonZeroU8,
    ease: f32,
    streak: u32,
    due_on: NaiveDate,
}

impl CardStore for InMemoryCardStore {
    fn upsert_position(&self, position: Position) -> Result<Position, StoreError> {
        let canonical = Self::canonicalize_position_for_storage(position)?;
        let mut positions = self.positions_write()?;
        Self::store_canonical_position(&mut *positions, canonical)
    }

    fn upsert_edge(&self, edge: EdgeInput) -> Result<Edge, StoreError> {
        self.ensure_position_exists(edge.parent_id)?;
        self.ensure_position_exists(edge.child_id)?;
        let canonical = Edge::from_input(edge);
        let mut edges = self.edges_write()?;
        Self::store_canonical_edge(&mut *edges, canonical)
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
        Self::store_opening_card(&mut *cards, owner_id, edge, state, card_id)
    }

    fn fetch_due_cards(&self, owner_id: &str, as_of: NaiveDate) -> Result<Vec<Card>, StoreError> {
        let cards = self.cards_read()?;
        Ok(Self::collect_due_cards_for_owner(&*cards, owner_id, as_of))
    }

    fn record_review(&self, review: ReviewRequest) -> Result<Card, StoreError> {
        let mut cards = self.cards_write()?;
        let card = Self::borrow_card_for_review(&mut *cards, &review)?;
        Self::apply_review(&mut card.state, &review)?;
        Ok(card.clone())
    }

    fn record_unlock(&self, unlock: UnlockRecord) -> Result<(), StoreError> {
        let mut unlocks = self.unlocks_write()?;
        Self::insert_unlock_or_error(&mut *unlocks, unlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::num::NonZeroU8;
    use std::panic::catch_unwind;

    const START_FEN_WHITE: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const AFTER_E4_BLACK: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    const AFTER_E4_E5_WHITE: &str = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";

    fn sample_position(fen: &str, ply: u32) -> Position {
        Position::new(fen, ply).expect("valid fen")
    }

    fn sample_edge(parent: &Position, child: &Position, uci: &str, san: &str) -> Edge {
        Edge::from_input(EdgeInput {
            parent_id: parent.id,
            move_uci: uci.into(),
            move_san: san.into(),
            child_id: child.id,
        })
    }

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

        let position = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        )
        .expect("valid FEN");
        let err = store.upsert_position(position).unwrap_err();
        assert!(matches!(err, StoreError::PoisonedLock { resource } if resource == "positions"));
    }

    #[test]
    fn canonicalize_position_rejects_invalid_side_to_move() {
        let position = Position {
            id: 1,
            fen: "invalid fen".into(),
            side_to_move: 'w',
            ply: 0,
        };
        let err = InMemoryCardStore::canonicalize_position_for_storage(position).unwrap_err();
        assert!(matches!(err, StoreError::InvalidPosition(_)));
    }

    #[test]
    fn store_canonical_position_detects_hash_collisions() {
        let mut positions = HashMap::new();
        let base = sample_position(START_FEN_WHITE, 0);
        positions.insert(base.id, base.clone());
        let conflicting = Position {
            id: base.id,
            fen: AFTER_E4_E5_WHITE.into(),
            side_to_move: 'w',
            ply: 0,
        };
        let err =
            InMemoryCardStore::store_canonical_position(&mut positions, conflicting).unwrap_err();
        assert!(matches!(err, StoreError::HashCollision { entity } if entity == "position"));
    }

    #[test]
    fn store_canonical_edge_detects_collisions() {
        let mut edges = HashMap::new();
        let parent = sample_position(START_FEN_WHITE, 0);
        let child = sample_position(AFTER_E4_BLACK, 1);
        let base = sample_edge(&parent, &child, "e2e4", "e4");
        edges.insert(base.id, base.clone());
        let conflicting = Edge {
            id: base.id,
            parent_id: parent.id,
            child_id: child.id,
            move_uci: "d2d4".into(),
            move_san: "d4".into(),
        };
        let err = InMemoryCardStore::store_canonical_edge(&mut edges, conflicting).unwrap_err();
        assert!(matches!(err, StoreError::HashCollision { entity } if entity == "edge"));
    }

    #[test]
    fn store_opening_card_reuses_existing_records() {
        let mut cards = HashMap::new();
        let parent = sample_position(START_FEN_WHITE, 0);
        let child = sample_position(AFTER_E4_BLACK, 1);
        let edge = sample_edge(&parent, &child, "e2e4", "e4");
        let state = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        let original = InMemoryCardStore::build_opening_card("owner", &edge, state.clone(), 42);
        cards.insert(original.id, original.clone());
        let stored = InMemoryCardStore::store_opening_card(&mut cards, "owner", &edge, state, 42)
            .expect("card should be returned");
        assert_eq!(stored.id, original.id);
    }

    #[test]
    fn store_opening_card_detects_owner_mismatch() {
        let mut cards = HashMap::new();
        let parent = sample_position(START_FEN_WHITE, 0);
        let child = sample_position(AFTER_E4_BLACK, 1);
        let edge = sample_edge(&parent, &child, "e2e4", "e4");
        let state = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        let original = InMemoryCardStore::build_opening_card("owner", &edge, state.clone(), 42);
        cards.insert(original.id, original);
        let err =
            InMemoryCardStore::store_opening_card(&mut cards, "other-owner", &edge, state, 42)
                .unwrap_err();
        assert!(matches!(err, StoreError::HashCollision { entity } if entity == "card"));
    }

    #[test]
    fn collect_due_cards_sorts_by_due_date_then_id() {
        let mut cards = HashMap::new();
        let mut earlier = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        earlier.consecutive_correct = 0;
        let later = CardState::new(naive_date(2023, 1, 2), NonZeroU8::new(1).unwrap(), 2.5);
        cards.insert(
            1,
            Card {
                id: 1,
                owner_id: "owner".into(),
                kind: CardKind::Opening { edge_id: 1 },
                state: later,
            },
        );
        cards.insert(
            2,
            Card {
                id: 2,
                owner_id: "owner".into(),
                kind: CardKind::Opening { edge_id: 2 },
                state: earlier,
            },
        );
        let due =
            InMemoryCardStore::collect_due_cards_for_owner(&cards, "owner", naive_date(2023, 1, 2));
        assert_eq!(due[0].id, 2);
        assert_eq!(due[1].id, 1);
    }

    #[test]
    fn borrow_card_for_review_errors_when_missing() {
        let mut cards = HashMap::new();
        let review = ReviewRequest {
            card_id: 999,
            reviewed_on: naive_date(2023, 1, 1),
            grade: 2,
        };
        let err = InMemoryCardStore::borrow_card_for_review(&mut cards, &review).unwrap_err();
        assert!(matches!(err, StoreError::MissingCard { id } if id == 999));
    }

    #[test]
    fn insert_unlock_or_error_prevents_duplicates() {
        let mut unlocks = HashSet::new();
        let record = UnlockRecord {
            owner_id: "owner".into(),
            edge_id: 7,
            unlocked_on: naive_date(2023, 1, 1),
        };
        InMemoryCardStore::insert_unlock_or_error(&mut unlocks, record.clone())
            .expect("first insert should succeed");
        let err = InMemoryCardStore::insert_unlock_or_error(&mut unlocks, record).unwrap_err();
        assert!(matches!(err, StoreError::DuplicateUnlock { edge, .. } if edge == 7));
    }

    #[test]
    fn validate_grade_rejects_out_of_range_values() {
        let err = InMemoryCardStore::validate_grade(5).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 5));
        assert!(InMemoryCardStore::validate_grade(4).is_ok());
    }

    #[test]
    fn interval_after_grade_adjusts_spacing() {
        let interval = NonZeroU8::new(3).unwrap();
        assert_eq!(
            InMemoryCardStore::interval_after_grade(interval, 2),
            interval
        );
        assert_eq!(
            InMemoryCardStore::interval_after_grade(interval, 4).get(),
            6
        );
    }

    #[test]
    fn ease_delta_for_grade_matches_expectations() {
        assert!(InMemoryCardStore::ease_delta_for_grade(0) < 0.0);
        assert!(InMemoryCardStore::ease_delta_for_grade(4) > 0.0);
    }

    #[test]
    fn ease_after_grade_clamps_results() {
        let eased = InMemoryCardStore::ease_after_grade(2.7, 4);
        assert!((eased - 2.8).abs() < f32::EPSILON);
    }

    #[test]
    fn next_streak_tracks_correct_answers() {
        assert_eq!(InMemoryCardStore::next_streak(2, 4), 3);
        assert_eq!(InMemoryCardStore::next_streak(5, 1), 0);
    }

    #[test]
    fn due_date_for_review_offsets_by_interval() {
        let date = naive_date(2023, 1, 1);
        let interval = NonZeroU8::new(3).unwrap();
        assert_eq!(
            InMemoryCardStore::due_date_for_review(date, interval),
            naive_date(2023, 1, 4)
        );
    }

    #[test]
    fn finalize_transition_collects_components() {
        let state = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        let review = ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 1),
            grade: 3,
        };
        let interval = NonZeroU8::new(2).unwrap();
        let transition = InMemoryCardStore::finalize_transition(&state, &review, interval, 2.3)
            .expect("transition should succeed");
        assert_eq!(transition.interval, interval);
        assert_eq!(transition.ease, 2.3);
        assert_eq!(transition.due_on, naive_date(2023, 1, 3));
    }

    #[test]
    fn commit_review_transition_updates_state() {
        let mut state = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        let transition = ReviewTransition {
            interval: NonZeroU8::new(3).unwrap(),
            ease: 2.1,
            streak: 4,
            due_on: naive_date(2023, 1, 4),
        };
        InMemoryCardStore::commit_review_transition(&mut state, naive_date(2023, 1, 1), transition);
        assert_eq!(state.interval.get(), 3);
        assert_eq!(state.ease_factor, 2.1);
        assert_eq!(state.consecutive_correct, 4);
        assert_eq!(state.last_reviewed_on, Some(naive_date(2023, 1, 1)));
        assert_eq!(state.due_on, naive_date(2023, 1, 4));
    }

    #[test]
    fn derive_review_transition_validates_grades() {
        let state = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        let review = ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 1),
            grade: 9,
        };
        let err = InMemoryCardStore::derive_review_transition(&state, &review).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 9));
    }

    #[test]
    fn apply_review_updates_state_fields() {
        let mut state = CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5);
        let review = ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 1),
            grade: 4,
        };
        InMemoryCardStore::apply_review(&mut state, &review).expect("review should succeed");
        assert_eq!(state.consecutive_correct, 1);
        assert_eq!(state.last_reviewed_on, Some(naive_date(2023, 1, 1)));
        assert!(state.due_on > naive_date(2023, 1, 1));
    }

    #[test]
    fn ensure_position_exists_errors_when_missing() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let err = store.ensure_position_exists(999).unwrap_err();
        assert!(matches!(err, StoreError::MissingPosition { id } if id == 999));
    }

    #[test]
    fn ensure_edge_exists_errors_when_missing() {
        let store = InMemoryCardStore::new(StorageConfig::default());
        let err = store.ensure_edge_exists(123).unwrap_err();
        assert!(matches!(err, StoreError::MissingEdge { id } if id == 123));
    }
}
