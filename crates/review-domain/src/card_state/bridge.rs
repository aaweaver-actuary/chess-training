use chrono::NaiveDate;
use thiserror::Error;

use crate::StudyStage;

/// Runtime metadata required to hydrate the scheduler's SM-2 state representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sm2Runtime {
    /// Stage tracked by the scheduler for the card.
    pub stage: StudyStage,
    /// Total number of lapses recorded for the card.
    pub lapses: u32,
    /// Total number of reviews recorded for the card.
    pub reviews: u32,
}

/// Persistence metadata that only exists in [`StoredCardState`](crate::StoredCardState).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredSnapshot {
    /// Consecutive correct answers tracked for the card.
    pub consecutive_correct: u32,
    /// Last day on which the card was reviewed.
    pub last_reviewed_on: Option<NaiveDate>,
}

/// Errors returned when converting between persisted and scheduler SM-2 states.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BridgeError {
    /// Scheduler interval exceeded the storage representation.
    #[error("SM-2 interval {interval_days} exceeds storable maximum of {max}")]
    IntervalOverflow { interval_days: u32, max: u8 },
    /// Scheduler interval dropped to zero which cannot be represented.
    #[error("SM-2 interval must be at least one day to persist")]
    IntervalTooSmall,
}
