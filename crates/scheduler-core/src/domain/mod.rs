//! Core scheduler domain structs shared across modules.

pub mod card;
pub mod card_kind;
pub mod card_state;

pub use card::Card;
pub use card_kind::CardKind;
pub use card_state::CardState;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::grade::ReviewGrade;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockRecord {
    pub card_id: Uuid,
    pub owner_id: Uuid,
    pub parent_prefix: Option<String>,
    pub day: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewOutcome {
    pub card: Card,
    pub previous_due: NaiveDate,
    pub grade: ReviewGrade,
}

#[cfg(test)]
mod tests {
    use crate::SchedulerConfig;

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
