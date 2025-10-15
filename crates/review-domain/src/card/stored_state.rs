use chrono::NaiveDate;
use std::num::NonZeroU8;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoredCardState {
    pub due_on: NaiveDate,
    pub interval: NonZeroU8,
    pub ease_factor: f32,
    pub consecutive_correct: u32,
    pub last_reviewed_on: Option<NaiveDate>,
}
