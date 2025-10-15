use review_domain::StudyStage;
use std::collections::BTreeMap;
use uuid::Uuid;

use chrono::NaiveDate;

use super::SchedulerStore;
use crate::store::candidate_ordering;
use crate::{Card, UnlockRecord};

/// Reference in-memory implementation of [`SchedulerStore`] used in tests.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    cards: BTreeMap<Uuid, Card>,
    unlock_log: Vec<UnlockRecord>,
}

impl InMemoryStore {
    /// Construct a new, empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SchedulerStore for InMemoryStore {
    fn get_card(&self, id: Uuid) -> Option<Card> {
        self.cards.get(&id).cloned()
    }

    fn upsert_card(&mut self, card: Card) {
        self.cards.insert(card.id, card);
    }

    fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card> {
        let mut due: Vec<Card> = self
            .cards
            .values()
            .filter(|card| {
                card.owner_id == owner_id
                    && card.state.due <= today
                    && !matches!(card.state.stage, StudyStage::New)
            })
            .cloned()
            .collect();
        due.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        due
    }

    fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card> {
        let mut candidates: Vec<Card> = self
            .cards
            .values()
            .filter(|card| card.owner_id == owner_id && matches!(card.state.stage, StudyStage::New))
            .cloned()
            .collect();
        candidates.sort_by(candidate_ordering);
        candidates
    }

    fn record_unlock(&mut self, record: UnlockRecord) {
        self.unlock_log.push(record);
    }

    fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord> {
        self.unlock_log
            .iter()
            .filter(|record| record.owner_id == owner_id && record.unlocked_on == day)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use crate::{CardKind, SchedulerUnlockDetail, domain::Sm2State};

    use super::*;

    fn make_card(id: Uuid, owner_id: Uuid) -> Card {
        let mut card = Card {
            id,
            owner_id,
            state: Sm2State::default(),
            kind: CardKind::Tactic(crate::domain::SchedulerTacticCard::new()),
        };
        // By default, set stage to Learning for due_cards tests to match implementation
        card.state.stage = StudyStage::Learning;
        card
    }

    fn make_unlock_record(owner_id: Uuid, card_id: Uuid, unlocked_on: NaiveDate) -> UnlockRecord {
        UnlockRecord {
            owner_id,
            detail: SchedulerUnlockDetail {
                card_id,
                parent_prefix: None,
            },
            unlocked_on,
        }
    }

    #[test]
    fn test_new_store_is_empty() {
        let store = InMemoryStore::new();
        assert!(store.cards.is_empty());
        assert!(store.unlock_log.is_empty());
    }

    #[test]
    fn test_upsert_and_get_card() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        let card = make_card(card_id, owner_id);

        assert!(store.get_card(card_id).is_none());
        store.upsert_card(card.clone());
        assert_eq!(store.get_card(card_id), Some(card.clone()));

        // Upsert with updated state
        let mut updated_card = card.clone();
        updated_card.state.stage = StudyStage::Review;
        store.upsert_card(updated_card.clone());
        assert_eq!(store.get_card(card_id), Some(updated_card));
    }

    #[test]
    fn test_due_cards_filters_and_sorts() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let other_owner = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        // Not due (future), wrong owner, and New stage should be excluded
        let card_due = make_card(Uuid::new_v4(), owner_id);
        let mut card_future = make_card(Uuid::new_v4(), owner_id);
        card_future.state.due = today.succ_opt().unwrap();
        let mut card_new = make_card(Uuid::new_v4(), owner_id);
        card_new.state.stage = StudyStage::New;
        let card_other_owner = make_card(Uuid::new_v4(), other_owner);
        // Multiple due cards for sorting
        let mut card_due_early = make_card(Uuid::new_v4(), owner_id);
        card_due_early.state.due = today.pred_opt().unwrap();

        store.upsert_card(card_due.clone());
        store.upsert_card(card_future);
        store.upsert_card(card_new);
        store.upsert_card(card_other_owner);
        store.upsert_card(card_due_early.clone());

        let mut due = store.due_cards(owner_id, today);
        let mut expected = vec![card_due_early, card_due];
        due.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        expected.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        assert_eq!(due, expected);
    }

    #[test]
    fn test_unlock_candidates_filters_and_sorts() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let other_owner = Uuid::new_v4();

        let mut card_new1 = make_card(Uuid::new_v4(), owner_id);
        card_new1.state.stage = StudyStage::New;
        let mut card_new2 = make_card(Uuid::new_v4(), owner_id);
        card_new2.state.stage = StudyStage::New;
        let mut card_learning = make_card(Uuid::new_v4(), owner_id);
        card_learning.state.stage = StudyStage::Learning;
        let mut card_new_other = make_card(Uuid::new_v4(), other_owner);
        card_new_other.state.stage = StudyStage::New;

        store.upsert_card(card_new1.clone());
        store.upsert_card(card_new2.clone());
        store.upsert_card(card_learning);
        store.upsert_card(card_new_other);

        let mut expected = vec![card_new1, card_new2];
        expected.sort_by(candidate_ordering);

        let candidates = store.unlock_candidates(owner_id);
        assert_eq!(candidates, expected);
    }

    #[test]
    fn test_record_unlock_and_unlocked_on() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let card_id1 = Uuid::new_v4();
        let card_id2 = Uuid::new_v4();
        let other_owner = Uuid::new_v4();
        let day = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let other_day = NaiveDate::from_ymd_opt(2024, 6, 11).unwrap();

        let record1 = make_unlock_record(owner_id, card_id1, day);
        let record2 = make_unlock_record(owner_id, card_id2, day);
        let record3 = make_unlock_record(owner_id, card_id1, other_day);
        let record4 = make_unlock_record(other_owner, card_id1, day);

        store.record_unlock(record1.clone());
        store.record_unlock(record2.clone());
        store.record_unlock(record3);
        store.record_unlock(record4);

        let unlocked = store.unlocked_on(owner_id, day);
        assert_eq!(unlocked, vec![record1, record2]);
        let unlocked_other = store.unlocked_on(owner_id, other_day);
        assert_eq!(unlocked_other.len(), 1);
    }

    #[test]
    fn test_empty_due_and_unlock_candidates_and_unlocked_on() {
        let store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        assert!(store.due_cards(owner_id, today).is_empty());
        assert!(store.unlock_candidates(owner_id).is_empty());
        assert!(store.unlocked_on(owner_id, today).is_empty());
    }

    //     /// Fetch a card by identifier if it exists.
    //     fn get_card(&self, id: Uuid) -> Option<Card>;

    #[test]
    fn test_implements_get_card() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        let card = make_card(card_id, owner_id);

        assert!(store.get_card(card_id).is_none());
        store.upsert_card(card.clone());
        assert_eq!(store.get_card(card_id), Some(card));
    }

    #[test]
    fn test_implements_upsert_card() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        let card = make_card(card_id, owner_id);

        assert!(store.get_card(card_id).is_none());
        store.upsert_card(card.clone());
        assert_eq!(store.get_card(card_id), Some(card));
    }

    //     /// Insert or update a card in the backing store.
    //     fn upsert_card(&mut self, card: Card);

    #[test]
    fn test_implements_due_cards() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let card = make_card(Uuid::new_v4(), owner_id);

        store.upsert_card(card.clone());
        assert_eq!(store.due_cards(owner_id, today), vec![card]);
    }

    //     /// Retrieve cards due for review on the given day.
    //     fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card>;

    #[test]
    fn test_implements_unlock_candidates() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let mut card = make_card(Uuid::new_v4(), owner_id);
        card.state.stage = StudyStage::New;
        store.upsert_card(card.clone());
        assert_eq!(store.unlock_candidates(owner_id), vec![card]);
    }
    //     /// Fetch cards eligible to be unlocked for future study.
    //     fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card>;

    #[test]
    fn test_implements_record_unlock() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        let day = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let record = make_unlock_record(owner_id, card_id, day);
        store.record_unlock(record.clone());
        assert_eq!(store.unlocked_on(owner_id, day), vec![record]);
    }
    //     /// Record a newly unlocked card.
    //     fn record_unlock(&mut self, record: UnlockRecord);

    #[test]
    fn test_implements_unlocked_on() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        let day = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let record = make_unlock_record(owner_id, card_id, day);
        store.record_unlock(record.clone());
        assert_eq!(store.unlocked_on(owner_id, day), vec![record]);
    }
    //     /// Retrieve unlock events that occurred on the provided day.
    //     fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord>;
    //

    //            .filter(|card| {
    //     card.owner_id == owner_id
    //         && card.state.due <= today
    //         && !matches!(card.state.stage, StudyStage::New)
    // })
    #[test]
    fn test_correctly_filters_cards_when_more_than_one_owner() {
        let mut store = InMemoryStore::new();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        let card1 = make_card(Uuid::new_v4(), owner1);
        let card2 = make_card(Uuid::new_v4(), owner2);

        store.upsert_card(card1.clone());
        store.upsert_card(card2.clone());

        assert_eq!(store.due_cards(owner1, today), vec![card1]);
        assert_eq!(store.due_cards(owner2, today), vec![card2]);
    }

    #[test]
    fn test_correctly_filters_cards_when_dates_before_after_and_equal_to_today() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        let card_due = make_card(Uuid::new_v4(), owner_id);
        let mut card_future = make_card(Uuid::new_v4(), owner_id);
        card_future.state.due = today.succ_opt().unwrap();
        let mut card_past = make_card(Uuid::new_v4(), owner_id);
        card_past.state.due = today.pred_opt().unwrap();

        store.upsert_card(card_due.clone());
        store.upsert_card(card_future);
        store.upsert_card(card_past.clone());

        let mut actual = store.due_cards(owner_id, today);
        let mut expected = vec![card_past, card_due];
        actual.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        expected.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_correctly_filters_cards_when_new_and_not_new() {
        let mut store = InMemoryStore::new();
        let owner_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        let mut card_new = make_card(Uuid::new_v4(), owner_id);
        card_new.state.stage = StudyStage::New;
        let mut card_learning = make_card(Uuid::new_v4(), owner_id);
        card_learning.state.stage = StudyStage::Learning;

        store.upsert_card(card_new);
        store.upsert_card(card_learning.clone());

        let mut actual = store.due_cards(owner_id, today);
        let mut expected = vec![card_learning];
        actual.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        expected.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
        assert_eq!(actual, expected);
    }
}
