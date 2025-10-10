//! Scheduling metadata tracked for each stored review card.

use std::num::NonZeroU8;

use chrono::NaiveDate;

/// Mutable scheduling state of a card stored by review services.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredCardState {
    /// Date on which the card becomes due.
    pub due_on: NaiveDate,
    /// Current interval in days.
    pub interval: NonZeroU8,
    /// Ease factor controlling growth of the interval.
    pub ease_factor: f32,
    /// Consecutive correct reviews streak.
    pub consecutive_correct: u32,
    /// Date of the last successful review.
    pub last_reviewed_on: Option<NaiveDate>,
}

impl StoredCardState {
    /// Creates a new [`StoredCardState`] with sensible defaults.
    #[must_use]
    pub fn new(due_on: NaiveDate, interval: NonZeroU8, ease_factor: f32) -> Self {
        Self {
            due_on,
            interval,
            ease_factor,
            consecutive_correct: 0,
            last_reviewed_on: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn constructor_sets_defaults() {
        let interval = NonZeroU8::new(1).expect("non-zero interval");
        let state = StoredCardState::new(naive_date(2023, 1, 1), interval, 2.5);
        assert_eq!(state.due_on, naive_date(2023, 1, 1));
        assert_eq!(state.interval, interval);
        let ease_factor: f32 = 2.5;
        assert_eq!(state.ease_factor, ease_factor);
        assert_eq!(state.consecutive_correct, 0);
        assert!(state.last_reviewed_on.is_none());
    }

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }
}
