//! Core scheduler domain structs shared across modules.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::config::SchedulerConfig;
use crate::grade::ReviewGrade;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    /// The card has never been studied; it is new to the learner.
    New,
    /// The card is in the initial learning phase and is being introduced to the learner.
    Learning,
    /// The card has been learned and is being reviewed at increasing intervals.
    Review,
    /// The card was previously learned but has lapsed and is being re-learned.
    Relearning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents the type of a card in the scheduler.
///
/// - `Opening`: A card representing an opening position or concept, typically associated with a parent entity.
/// - `Tactic`: A card representing a tactical motif or problem, not associated with a parent.
pub enum CardKind {
    /// An opening card, associated with a parent entity.
    ///
    /// `parent_prefix` identifies the parent (e.g., a specific opening line or group) to which this card belongs.
    Opening { parent_prefix: String },
    /// A tactic card, representing a standalone tactical motif or problem.
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
