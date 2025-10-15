//! Core scheduler domain structs shared across modules.

/// Card definitions specialized for the scheduler.
pub mod card;
/// Card kind payloads used by the scheduler.
pub mod card_kind;
/// State transitions and learning stages tracked by the scheduler.
pub mod card_state;
/// Spaced repetition metadata stored alongside each card.
pub mod sm2_state;
/// Shared conversions between stored and scheduler card states.
pub mod state_bridge;

/// Scheduler-specific card wrapper and constructor helpers.
pub use card::{Card, new_card};
/// Card kind payloads exposed to scheduler consumers.
pub use card_kind::{CardKind, SchedulerOpeningCard, SchedulerTacticCard};
/// Scheduler-specific card state enumeration.
pub use card_state::CardState;
/// SM-2 state tracked for each scheduled card.
pub use sm2_state::Sm2State;
pub use state_bridge::{
    BridgeError as CardStateBridgeError, Sm2Runtime, StoredSnapshot, hydrate_sm2_state,
    persist_sm2_state,
};

use chrono::NaiveDate;
use uuid::Uuid;

use review_domain::ReviewGrade;

use review_domain::UnlockRecord as GenericUnlockRecord;

/// Domain-specific payload stored for scheduler unlock events.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchedulerUnlockDetail {
    /// Identifier of the card that became available for study.
    pub card_id: Uuid,
    /// Optional prefix used to group unlocks by their parent line.
    pub parent_prefix: Option<String>,
}

/// Unlock events emitted by the scheduler.
pub type UnlockRecord = GenericUnlockRecord<Uuid, SchedulerUnlockDetail>;

/// Result of recording a review, including the updated card state.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewOutcome {
    /// Card after applying the review outcome.
    pub card: Card,
    /// Due date before the review was processed.
    pub previous_due: NaiveDate,
    /// Grade provided by the learner for the review.
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
