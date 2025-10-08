//! Core scheduler domain structs shared across modules.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::config::SchedulerConfig;
use crate::grade::ReviewGrade;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKind {
    Opening { parent_prefix: String },
    Tactic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockRecord {
    pub card_id: Uuid,
    pub owner_id: Uuid,
    pub parent_prefix: Option<String>,
    pub day: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub kind: CardKind,
    pub state: CardState,
    pub ease_factor: f32,
    pub interval_days: u32,
    pub due: NaiveDate,
    pub lapses: u32,
    pub reviews: u32,
}

impl Card {
    pub fn new(owner_id: Uuid, kind: CardKind, today: NaiveDate, config: &SchedulerConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            kind,
            state: CardState::New,
            ease_factor: config.initial_ease_factor,
            interval_days: 0,
            due: today,
            lapses: 0,
            reviews: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewOutcome {
    pub card: Card,
    pub previous_due: NaiveDate,
    pub grade: ReviewGrade,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_new_initializes_expected_defaults() {
        let config = SchedulerConfig::default();
        let owner = Uuid::new_v4();
        let today = chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let card = Card::new(owner, CardKind::Tactic, today, &config);
        assert_eq!(card.owner_id, owner);
        assert_eq!(card.state, CardState::New);
        assert_eq!(card.due, today);
        assert_eq!(card.reviews, 0);
    }
}
