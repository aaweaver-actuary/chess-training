//! Scheduling metadata tracked for each stored review card.

use std::num::NonZeroU8;

use chrono::{Duration, NaiveDate};

use crate::ValidGrade;

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

    /// Compute the next review interval based on the provided [`ValidGrade`].
    #[must_use]
    pub fn next_interval(&self, grade: ValidGrade) -> NonZeroU8 {
        match grade {
            ValidGrade::Zero | ValidGrade::One => NonZeroU8::new(1).expect("non zero"),
            ValidGrade::Two => self.interval,
            ValidGrade::Three => {
                let next = self.interval.get().saturating_add(1);
                NonZeroU8::new(next).expect("saturating_add should be non zero")
            }
            ValidGrade::Four => {
                let next = self.interval.get().saturating_mul(2);
                NonZeroU8::new(next).expect(&format!(
                    "Expected saturating_mul(2) of interval {} to be non-zero, but got {}. This should be impossible for NonZeroU8.",
                    self.interval.get(),
                    next
                ))
            }
        }
    }

    /// Compute the next ease factor after applying the [`ValidGrade`].
    #[must_use]
    pub fn next_ease_factor(&self, grade: ValidGrade) -> f32 {
        (self.ease_factor + grade.to_grade_delta()).clamp(1.3, 2.8)
    }

    /// Compute the consecutive streak after applying the [`ValidGrade`].
    #[must_use]
    pub fn next_streak(&self, grade: ValidGrade) -> u32 {
        if grade.is_correct() {
            self.consecutive_correct.saturating_add(1)
        } else {
            0
        }
    }

    /// Apply the review to the current state, mutating it in place.
    pub fn apply_review(&mut self, grade: ValidGrade, reviewed_on: NaiveDate) {
        let next_interval = self.next_interval(grade);
        self.interval = next_interval;
        self.ease_factor = self.next_ease_factor(grade);
        self.consecutive_correct = self.next_streak(grade);
        self.last_reviewed_on = Some(reviewed_on);
        self.due_on = reviewed_on + Duration::days(i64::from(next_interval.get()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    use crate::ValidGrade;

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

    #[test]
    fn next_interval_applies_grade_rules() {
        let interval = NonZeroU8::new(3).unwrap();
        let state = StoredCardState::new(naive_date(2023, 1, 1), interval, 2.5);
        assert_eq!(state.next_interval(ValidGrade::Zero).get(), 1);
        assert_eq!(state.next_interval(ValidGrade::One).get(), 1);
        assert_eq!(state.next_interval(ValidGrade::Two).get(), 3);
        assert_eq!(state.next_interval(ValidGrade::Three).get(), 4);
        assert_eq!(state.next_interval(ValidGrade::Four).get(), 6);
    }

    #[test]
    fn next_ease_factor_clamps_values() {
        let interval = NonZeroU8::new(1).unwrap();
        let mut state = StoredCardState::new(naive_date(2023, 1, 1), interval, 2.7);
        assert!((state.next_ease_factor(ValidGrade::Four) - 2.8).abs() < f32::EPSILON);
        state.ease_factor = 1.2;
        assert!((state.next_ease_factor(ValidGrade::Zero) - 1.3).abs() < f32::EPSILON);
        state.ease_factor = 2.0;
        assert!((state.next_ease_factor(ValidGrade::Three) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn next_streak_increments_only_for_correct_answers() {
        let interval = NonZeroU8::new(1).unwrap();
        let mut state = StoredCardState::new(naive_date(2023, 1, 1), interval, 2.5);
        state.consecutive_correct = 2;
        assert_eq!(state.next_streak(ValidGrade::Zero), 0);
        assert_eq!(state.next_streak(ValidGrade::Three), 3);
    }

    #[test]
    fn apply_review_updates_all_fields() {
        let interval = NonZeroU8::new(2).unwrap();
        let mut state = StoredCardState::new(naive_date(2023, 1, 1), interval, 2.5);
        let review_day = naive_date(2023, 1, 10);
        state.consecutive_correct = 1;
        state.apply_review(ValidGrade::Four, review_day);
        assert_eq!(state.interval.get(), 4);
        assert_eq!(state.due_on, naive_date(2023, 1, 14));
        assert_eq!(state.last_reviewed_on, Some(review_day));
        assert_eq!(state.consecutive_correct, 2);
        assert!((state.ease_factor - 2.65).abs() < f32::EPSILON);
    }
}
