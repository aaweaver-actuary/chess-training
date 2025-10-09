//! High-level scheduler orchestrating SM-2 reviews and unlock queue construction.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::config::SchedulerConfig;
use crate::domain::{Card, ReviewOutcome};
use crate::errors::SchedulerError;
use crate::grade::ReviewGrade;
use crate::queue::build_queue_for_day;
use crate::sm2::apply_sm2;
use crate::store::CardStore;

pub struct Scheduler<S: CardStore> {
    store: S,
    config: SchedulerConfig,
}

impl<S: CardStore> Scheduler<S> {
    pub fn new(store: S, config: SchedulerConfig) -> Self {
        Self { store, config }
    }

    pub fn review(
        &mut self,
        card_id: Uuid,
        grade: ReviewGrade,
        today: NaiveDate,
    ) -> Result<ReviewOutcome, SchedulerError> {
        let mut card = self
            .store
            .get_card(card_id)
            .ok_or(SchedulerError::CardNotFound(card_id))?;
        let previous_due = card.state.due;
        apply_sm2(&mut card, grade, &self.config, today);
        self.store.upsert_card(card.clone());
        Ok(ReviewOutcome {
            card,
            previous_due,
            grade,
        })
    }

    pub fn build_queue(&mut self, owner_id: Uuid, today: NaiveDate) -> Vec<Card> {
        build_queue_for_day(&mut self.store, &self.config, owner_id, today)
    }

    pub fn into_store(self) -> S {
        self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{new_card, CardKind, CardState, SchedulerTacticCard};
    use crate::store::InMemoryStore;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn review_updates_store_with_new_state() {
        let mut store = InMemoryStore::new();
        let config = SchedulerConfig::default();
        let owner = Uuid::new_v4();
        let mut card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );
        card.state.stage = CardState::Review;
        store.upsert_card(card.clone());
        let mut scheduler = Scheduler::new(store, config.clone());
        let outcome = scheduler
            .review(card.id, ReviewGrade::Good, naive_date(2023, 1, 1))
            .expect("card exists");
        assert_eq!(outcome.grade, ReviewGrade::Good);
        assert!(outcome.card.state.due >= naive_date(2023, 1, 2));
    }

    #[test]
    fn build_queue_delegates_to_helper() {
        let store = InMemoryStore::new();
        let config = SchedulerConfig::default();
        let owner = Uuid::new_v4();
        let mut scheduler = Scheduler::new(store, config);
        let queue = scheduler.build_queue(owner, naive_date(2023, 1, 1));
        assert!(queue.is_empty());
    }
}
