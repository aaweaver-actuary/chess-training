//! Persistence abstraction used by the scheduler along with an in-memory reference store.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::{Card, CardKind, CardState, UnlockRecord};

pub trait CardStore {
    fn get_card(&self, id: Uuid) -> Option<Card>;
    fn upsert_card(&mut self, card: Card);
    fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card>;
    fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card>;
    fn record_unlock(&mut self, record: UnlockRecord);
    fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord>;
}

#[derive(Debug, Default)]
pub struct InMemoryStore {
    cards: BTreeMap<Uuid, Card>,
    unlock_log: Vec<UnlockRecord>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CardStore for InMemoryStore {
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
                    && card.due <= today
                    && !matches!(card.state, CardState::New)
            })
            .cloned()
            .collect();
        due.sort_by(|a, b| (a.due, a.id).cmp(&(b.due, b.id)));
        due
    }

    fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card> {
        let mut candidates: Vec<Card> = self
            .cards
            .values()
            .filter(|card| card.owner_id == owner_id && matches!(card.state, CardState::New))
            .cloned()
            .collect();
        candidates.sort_by(|a, b| match (&a.kind, &b.kind) {
            (
                CardKind::Opening {
                    parent_prefix: a_prefix,
                },
                CardKind::Opening {
                    parent_prefix: b_prefix,
                },
            ) => (a_prefix, &a.id).cmp(&(b_prefix, &b.id)),
            (CardKind::Opening { .. }, _) => std::cmp::Ordering::Less,
            (_, CardKind::Opening { .. }) => std::cmp::Ordering::Greater,
            _ => a.id.cmp(&b.id),
        });
        candidates
    }

    fn record_unlock(&mut self, record: UnlockRecord) {
        self.unlock_log.push(record);
    }

    fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord> {
        self.unlock_log
            .iter()
            .filter(|record| record.owner_id == owner_id && record.day == day)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SchedulerConfig;
    use chrono::NaiveDate;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn due_cards_filters_by_owner_and_due_date() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let mut card = Card::new(
            owner,
            CardKind::Tactic,
            naive_date(2023, 1, 1),
            &SchedulerConfig::default(),
        );
        card.state = CardState::Review;
        card.due = naive_date(2023, 1, 2);
        store.upsert_card(card.clone());
        let due = store.due_cards(owner, naive_date(2023, 1, 2));
        assert_eq!(due, vec![card]);
    }

    #[test]
    fn unlock_candidates_only_returns_new_cards() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let new_card = Card::new(
            owner,
            CardKind::Tactic,
            naive_date(2023, 1, 1),
            &SchedulerConfig::default(),
        );
        let mut review_card = Card::new(
            owner,
            CardKind::Tactic,
            naive_date(2023, 1, 1),
            &SchedulerConfig::default(),
        );
        review_card.state = CardState::Review;
        store.upsert_card(new_card.clone());
        store.upsert_card(review_card);
        let candidates = store.unlock_candidates(owner);
        assert_eq!(candidates, vec![new_card]);
    }

    #[test]
    fn unlocked_on_filters_by_owner_and_day() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let record = UnlockRecord {
            card_id: Uuid::new_v4(),
            owner_id: owner,
            parent_prefix: Some("e4".into()),
            day: naive_date(2023, 1, 1),
        };
        store.record_unlock(record.clone());
        let logs = store.unlocked_on(owner, naive_date(2023, 1, 1));
        assert_eq!(logs, vec![record]);
    }
}
