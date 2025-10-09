//! Core scheduler domain structs shared across modules.

pub mod card;
pub mod card_kind;
pub mod card_state;
pub mod sm2_state;

pub use card::{Card, new_card};
pub use card_kind::{CardKind, SchedulerOpeningCard, SchedulerTacticCard};
pub use card_state::CardState;
pub use sm2_state::Sm2State;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::grade::ReviewGrade;

use review_domain::UnlockRecord as GenericUnlockRecord;

/// Domain-specific payload stored for scheduler unlock events.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchedulerUnlockDetail {
    pub card_id: Uuid,
    pub parent_prefix: Option<String>,
}

/// Unlock events emitted by the scheduler.
pub type UnlockRecord = GenericUnlockRecord<Uuid, SchedulerUnlockDetail>;

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
        let today = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
            .expect("1 January 2023 should be representable");
        let card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            today,
            &config,
        );
        assert_eq!(card.owner_id, owner);
        assert_eq!(card.state.stage, CardState::New);
        assert_eq!(card.state.due, today);
        assert_eq!(card.state.reviews, 0);
    }
}
