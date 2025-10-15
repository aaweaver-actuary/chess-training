//! Persistence abstraction used by the scheduler along with an in-memory reference store.

pub mod candidate_ordering;
pub mod in_memory_store;
pub mod scheduler_store;

pub use candidate_ordering::candidate_ordering;
pub use in_memory_store::InMemoryStore;
pub use scheduler_store::SchedulerStore;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SchedulerConfig;
    use crate::domain::{
        SchedulerOpeningCard, SchedulerTacticCard, SchedulerUnlockDetail, new_card,
    };
    use crate::{CardKind, UnlockRecord};
    use chrono::NaiveDate;
    use review_domain::StudyStage;
    use std::cmp::Ordering;
    use uuid::Uuid;

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
        card.state.stage = StudyStage::Review;
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
        review_card.state.stage = StudyStage::Review;
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
