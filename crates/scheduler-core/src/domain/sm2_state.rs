use chrono::NaiveDate;
use super::CardState;

/// Mutable SM-2 scheduling data tracked for a card.
#[derive(Debug, Clone, PartialEq)]
pub struct Sm2State {
    /// Conceptual study stage for the card.
    pub stage: CardState,
    /// Ease factor controlling interval growth.
    pub ease_factor: f32,
    /// Current interval in days.
    pub interval_days: u32,
    /// Next due date for the card.
    pub due: NaiveDate,
    /// Total number of lapses recorded.
    pub lapses: u32,
    /// Total number of reviews completed.
    pub reviews: u32,
}

impl Sm2State {
    /// Constructs a new SM-2 state for a freshly created card.
    pub fn new(stage: CardState, today: NaiveDate, initial_ease: f32) -> Self {
        Self {
            stage,
            ease_factor: initial_ease,
            interval_days: 0,
            due: today,
            lapses: 0,
            reviews: 0,
        }
    }
}
