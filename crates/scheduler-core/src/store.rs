//! Persistence abstraction used by the scheduler along with an in-memory reference store.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::{Card, CardKind, CardState, UnlockRecord};

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
                    && !matches!(card.state.stage, CardState::New)
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
            .filter(|card| card.owner_id == owner_id && matches!(card.state.stage, CardState::New))
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

fn candidate_ordering(a: &Card, b: &Card) -> std::cmp::Ordering {
    match (&a.kind, &b.kind) {
        (CardKind::Opening(a_opening), CardKind::Opening(b_opening)) => {
            (&a_opening.parent_prefix, &a.id).cmp(&(&b_opening.parent_prefix, &b.id))
        }
        (CardKind::Opening(_), _) => std::cmp::Ordering::Less,
        (_, CardKind::Opening(_)) => std::cmp::Ordering::Greater,
        (CardKind::Tactic(_), CardKind::Tactic(_)) => a.id.cmp(&b.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SchedulerConfig;
    use crate::domain::{
        SchedulerOpeningCard, SchedulerTacticCard, SchedulerUnlockDetail, new_card,
    };
    use chrono::NaiveDate;
    use std::cmp::Ordering;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn due_cards_filters_by_owner_and_due_date() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let mut card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &SchedulerConfig::default(),
        );
        card.state.stage = CardState::Review;
        card.state.due = naive_date(2023, 1, 2);
        store.upsert_card(card.clone());
        let due = store.due_cards(owner, naive_date(2023, 1, 2));
        assert_eq!(due, vec![card]);
    }

    #[test]
    fn unlock_candidates_only_returns_new_cards() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let fresh_card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &SchedulerConfig::default(),
        );
        let mut review_card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &SchedulerConfig::default(),
        );
        review_card.state.stage = CardState::Review;
        store.upsert_card(fresh_card.clone());
        store.upsert_card(review_card);
        let candidates = store.unlock_candidates(owner);
        assert_eq!(candidates, vec![fresh_card]);
    }

    #[test]
    fn unlocked_on_filters_by_owner_and_day() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let record = UnlockRecord {
            owner_id: owner,
            detail: SchedulerUnlockDetail {
                card_id: Uuid::new_v4(),
                parent_prefix: Some("e4".into()),
            },
            unlocked_on: naive_date(2023, 1, 1),
        };
        store.record_unlock(record.clone());
        let logs = store.unlocked_on(owner, naive_date(2023, 1, 1));
        assert_eq!(logs, vec![record]);
    }

    #[test]
    fn unlock_candidates_prioritizes_openings_and_orders_tactics() {
        let mut store = InMemoryStore::new();
        let owner = Uuid::new_v4();
        let config = SchedulerConfig::default();
        let mut opening = new_card(
            owner,
            CardKind::Opening(SchedulerOpeningCard::new("a")),
            naive_date(2023, 1, 1),
            &config,
        );
        let mut later_opening = new_card(
            owner,
            CardKind::Opening(SchedulerOpeningCard::new("b")),
            naive_date(2023, 1, 1),
            &config,
        );
        let mut tactic = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );
        let mut another_tactic = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );
        opening.id = Uuid::from_u128(2);
        later_opening.id = Uuid::from_u128(4);
        tactic.id = Uuid::from_u128(3);
        another_tactic.id = Uuid::from_u128(1);
        store.upsert_card(opening.clone());
        store.upsert_card(later_opening.clone());
        store.upsert_card(tactic.clone());
        store.upsert_card(another_tactic.clone());

        let candidates = store.unlock_candidates(owner);
        let prefixes: Vec<_> = candidates
            .iter()
            .filter_map(|card| match &card.kind {
                CardKind::Opening(opening) => Some(opening.parent_prefix.clone()),
                CardKind::Tactic(_) => None,
            })
            .collect();
        assert_eq!(prefixes, vec!["a".to_string(), "b".to_string()]);
        let tactic_ids: Vec<_> = candidates
            .iter()
            .filter(|card| matches!(card.kind, CardKind::Tactic(_)))
            .map(|card| card.id)
            .collect();
        let mut expected_ids = vec![tactic.id, another_tactic.id];
        expected_ids.sort();
        assert_eq!(tactic_ids, expected_ids);
    }

    #[test]
    fn candidate_ordering_handles_mixed_kinds() {
        let owner = Uuid::new_v4();
        let config = SchedulerConfig::default();
        let mut opening = new_card(
            owner,
            CardKind::Opening(SchedulerOpeningCard::new("a")),
            naive_date(2023, 1, 1),
            &config,
        );
        let mut tactic = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );
        let mut second_tactic = tactic.clone();
        opening.id = Uuid::from_u128(1);
        tactic.id = Uuid::from_u128(2);
        second_tactic.id = Uuid::from_u128(3);

        assert_eq!(super::candidate_ordering(&opening, &tactic), Ordering::Less);
        assert_eq!(
            super::candidate_ordering(&tactic, &opening),
            Ordering::Greater
        );
        assert_eq!(
            super::candidate_ordering(&tactic, &second_tactic),
            tactic.id.cmp(&second_tactic.id)
        );
    }
}
