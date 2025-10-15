use crate::domain::{Card, UnlockRecord};
use chrono::NaiveDate;
use uuid::Uuid;

/// Storage abstraction required by the scheduler to retrieve and persist cards.
pub trait SchedulerStore {
    /// Fetch a card by identifier if it exists.
    fn get_card(&self, id: Uuid) -> Option<Card>;
    /// Insert or update a card in the backing store.
    fn upsert_card(&mut self, card: Card);
    /// Retrieve cards due for review on the given day.
    fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card>;
    /// Fetch cards eligible to be unlocked for future study.
    fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card>;
    /// Record a newly unlocked card.
    fn record_unlock(&mut self, record: UnlockRecord);
    /// Retrieve unlock events that occurred on the provided day.
    fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord>;
}

#[cfg(test)]
mod tests {
    use crate::domain::{CardKind, CardState, SchedulerTacticCard, Sm2State};

    use super::*;
    use std::collections::HashMap;

    // Dummy implementations for Card and UnlockRecord for testing
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct DummyCard {
        id: Uuid,
        owner_id: Uuid,
        due: NaiveDate,
    }

    impl DummyCard {
        fn new(owner_id: Uuid, due: NaiveDate) -> Self {
            Self {
                id: Uuid::new_v4(),
                owner_id,
                due,
            }
        }
    }

    impl From<&DummyCard> for Card {
        fn from(dc: &DummyCard) -> Self {
            Card {
                kind: CardKind::Tactic(SchedulerTacticCard::new()),
                state: Sm2State::new(CardState::New, dc.due, 2.5),
                id: dc.id,
                owner_id: dc.owner_id,
            }
        }
    }

    struct DummyUnlockRecord {
        card_id: Uuid,
        owner_id: Uuid,
        day: NaiveDate,
    }

    impl From<&DummyUnlockRecord> for UnlockRecord {
        fn from(dr: &DummyUnlockRecord) -> Self {
            UnlockRecord {
                owner_id: dr.owner_id,
                unlocked_on: dr.day,
                detail: crate::domain::SchedulerUnlockDetail {
                    card_id: dr.card_id,
                    parent_prefix: None,
                },
            }
        }
    }

    // In-memory implementation for testing
    struct InMemorySchedulerStore {
        cards: HashMap<Uuid, Card>,
        unlocks: Vec<UnlockRecord>,
    }

    impl InMemorySchedulerStore {
        fn new() -> Self {
            Self {
                cards: HashMap::new(),
                unlocks: Vec::new(),
            }
        }
    }

    impl SchedulerStore for InMemorySchedulerStore {
        fn get_card(&self, id: Uuid) -> Option<Card> {
            self.cards.get(&id).cloned()
        }

        fn upsert_card(&mut self, card: Card) {
            self.cards.insert(card.id, card);
        }

        fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card> {
            self.cards
                .values()
                .filter(|c| c.owner_id == owner_id && c.state.due <= today)
                .cloned()
                .collect()
        }

        fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card> {
            self.cards
                .values()
                .filter(|c| c.owner_id == owner_id)
                .cloned()
                .collect()
        }

        fn record_unlock(&mut self, record: UnlockRecord) {
            self.unlocks.push(record);
        }

        fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord> {
            self.unlocks
                .iter()
                .filter(|r| r.owner_id == owner_id && r.unlocked_on == day)
                .cloned()
                .collect()
        }
    }

    fn make_card(owner_id: Uuid, due: NaiveDate) -> Card {
        (&DummyCard::new(owner_id, due)).into()
    }

    fn make_unlock(card_id: Uuid, owner_id: Uuid, day: NaiveDate) -> UnlockRecord {
        (&DummyUnlockRecord {
            card_id,
            owner_id,
            day,
        })
            .into()
    }

    #[test]
    fn test_get_card_none_when_empty() {
        let store = InMemorySchedulerStore::new();
        let id = Uuid::new_v4();
        assert_eq!(store.get_card(id), None);
    }

    #[test]
    fn test_upsert_and_get_card() {
        let mut store = InMemorySchedulerStore::new();
        let owner_id = Uuid::new_v4();
        let card = make_card(owner_id, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        let id = card.id;
        store.upsert_card(card.clone());
        assert_eq!(store.get_card(id), Some(card.clone()));

        // Upsert with changed due date
        let mut updated_card = card.clone();
        updated_card.state.due = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        store.upsert_card(updated_card.clone());
        assert_eq!(store.get_card(id), Some(updated_card));
    }

    #[test]
    fn test_due_cards_filters_by_owner_and_date() {
        let mut store = InMemorySchedulerStore::new();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        let card1 = make_card(owner1, today); // due today
        let card2 = make_card(owner1, today.pred_opt().unwrap()); // due before today
        let card3 = make_card(owner1, today.succ_opt().unwrap()); // due after today
        let card4 = make_card(owner2, today); // different owner

        for card in [&card1, &card2, &card3, &card4] {
            store.upsert_card(card.clone());
        }

        let due = store.due_cards(owner1, today);
        assert!(due.contains(&card1));
        assert!(due.contains(&card2));
        assert!(!due.contains(&card3));
        assert!(!due.contains(&card4));
    }

    #[test]
    fn test_unlock_candidates_returns_all_for_owner() {
        let mut store = InMemorySchedulerStore::new();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        let card1 = make_card(owner1, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        let card2 = make_card(owner1, NaiveDate::from_ymd_opt(2024, 6, 2).unwrap());
        let card3 = make_card(owner2, NaiveDate::from_ymd_opt(2024, 6, 3).unwrap());

        for card in [&card1, &card2, &card3] {
            store.upsert_card(card.clone());
        }

        let candidates = store.unlock_candidates(owner1);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&card1));
        assert!(candidates.contains(&card2));
        assert!(!candidates.contains(&card3));
    }

    #[test]
    fn test_record_unlock_and_unlocked_on() {
        let mut store = InMemorySchedulerStore::new();
        let owner_id = Uuid::new_v4();
        let card_id1 = Uuid::new_v4();
        let card_id2 = Uuid::new_v4();
        let day1 = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2024, 6, 11).unwrap();

        let unlock1 = make_unlock(card_id1, owner_id, day1);
        let unlock2 = make_unlock(card_id2, owner_id, day1);
        let unlock3 = make_unlock(card_id1, owner_id, day2);

        store.record_unlock(unlock1.clone());
        store.record_unlock(unlock2.clone());
        store.record_unlock(unlock3.clone());

        let unlocked_day1 = store.unlocked_on(owner_id, day1);
        assert_eq!(unlocked_day1.len(), 2);
        assert!(unlocked_day1.contains(&unlock1));
        assert!(unlocked_day1.contains(&unlock2));

        let unlocked_day2 = store.unlocked_on(owner_id, day2);
        assert_eq!(unlocked_day2.len(), 1);
        assert!(unlocked_day2.contains(&unlock3));
    }

    #[test]
    fn test_unlocked_on_filters_by_owner() {
        let mut store = InMemorySchedulerStore::new();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        let day = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        let unlock1 = make_unlock(card_id, owner1, day);
        let unlock2 = make_unlock(card_id, owner2, day);

        store.record_unlock(unlock1.clone());
        store.record_unlock(unlock2.clone());

        let unlocked_owner1 = store.unlocked_on(owner1, day);
        assert_eq!(unlocked_owner1, vec![unlock1]);
        let unlocked_owner2 = store.unlocked_on(owner2, day);
        assert_eq!(unlocked_owner2, vec![unlock2]);
    }

    #[test]
    fn test_edge_cases_empty_store() {
        let store = InMemorySchedulerStore::new();
        let owner_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        assert!(store.due_cards(owner_id, today).is_empty());
        assert!(store.unlock_candidates(owner_id).is_empty());
        assert!(store.unlocked_on(owner_id, today).is_empty());
    }
}
