use std::fmt;
use std::num::NonZeroU8;
use std::ops::RangeInclusive;

use chrono::{Duration, NaiveDate};

use super::StoredCardState;

/// Declarative representation of the invariants enforced for [`StoredCardState`].
#[derive(Clone, Debug, PartialEq)]
pub struct CardStateInvariants {
    min_interval_days: NonZeroU8,
    ease_factor_range: RangeInclusive<f32>,
}

impl CardStateInvariants {
    /// Builds a new invariant set with the provided minimum interval and ease factor range.
    #[must_use]
    pub fn new(min_interval_days: NonZeroU8, ease_factor_range: RangeInclusive<f32>) -> Self {
        Self {
            min_interval_days,
            ease_factor_range,
        }
    }

    /// Minimum interval in days that a card is allowed to have.
    #[must_use]
    pub fn min_interval_days(&self) -> NonZeroU8 {
        self.min_interval_days
    }

    /// Inclusive bounds on the ease factor stored with a card.
    #[must_use]
    pub fn ease_factor_bounds(&self) -> (f32, f32) {
        (
            *self.ease_factor_range.start(),
            *self.ease_factor_range.end(),
        )
    }

    /// Validates that the provided [`StoredCardState`] satisfies the invariants.
    ///
    /// # Errors
    ///
    /// Returns [`CardStateInvariantError`] when the state violates any of the
    /// encapsulated rules.
    pub fn validate(&self, state: &StoredCardState) -> Result<(), CardStateInvariantError> {
        self.ensure_interval(state)?;
        self.ensure_ease_factor(state)?;
        ensure_due_date(state)?;
        Ok(())
    }

    fn ensure_interval(&self, state: &StoredCardState) -> Result<(), CardStateInvariantError> {
        if state.interval < self.min_interval_days {
            return Err(CardStateInvariantError::IntervalBelowMinimum {
                interval: state.interval,
                minimum: self.min_interval_days,
            });
        }

        Ok(())
    }

    fn ensure_ease_factor(&self, state: &StoredCardState) -> Result<(), CardStateInvariantError> {
        let (min_ease, max_ease) = self.ease_factor_bounds();
        if !(min_ease..=max_ease).contains(&state.ease_factor) {
            return Err(CardStateInvariantError::EaseFactorOutOfRange {
                ease_factor: state.ease_factor,
                minimum: min_ease,
                maximum: max_ease,
            });
        }

        Ok(())
    }
}

impl Default for CardStateInvariants {
    fn default() -> Self {
        // SM-2 keeps ease factors clamped between 1.3 and 2.8 and never allows
        // the interval to hit zero days.
        Self::new(NonZeroU8::MIN, 1.3..=2.8)
    }
}

fn ensure_due_date(state: &StoredCardState) -> Result<(), CardStateInvariantError> {
    if let Some(last_reviewed_on) = state.last_reviewed_on {
        if state.due_on < last_reviewed_on {
            return Err(CardStateInvariantError::DueDateBeforeLastReview {
                due_on: state.due_on,
                last_reviewed_on,
            });
        }

        let expected_due_on = last_reviewed_on + Duration::days(i64::from(state.interval.get()));
        if state.due_on != expected_due_on {
            return Err(CardStateInvariantError::DueDateMismatch {
                due_on: state.due_on,
                expected_due_on,
                interval_days: state.interval,
            });
        }
    }

    Ok(())
}

/// Errors returned when a [`StoredCardState`] violates a scheduling invariant.
#[derive(Debug, PartialEq)]
pub enum CardStateInvariantError {
    /// The stored interval is less than the supported minimum.
    IntervalBelowMinimum {
        /// Interval present in the state being validated.
        interval: NonZeroU8,
        /// Required minimum interval.
        minimum: NonZeroU8,
    },
    /// The ease factor falls outside the clamped SM-2 range.
    EaseFactorOutOfRange {
        /// Ease factor present on the state.
        ease_factor: f32,
        /// Lower bound enforced by the invariants.
        minimum: f32,
        /// Upper bound enforced by the invariants.
        maximum: f32,
    },
    /// The due date precedes the last review timestamp, breaking monotonicity.
    DueDateBeforeLastReview {
        /// Date on which the card is scheduled to be reviewed next.
        due_on: NaiveDate,
        /// Date of the most recent review.
        last_reviewed_on: NaiveDate,
    },
    /// The due date is not aligned with the interval added to the last review date.
    DueDateMismatch {
        /// Date on which the card is scheduled to be reviewed next.
        due_on: NaiveDate,
        /// Due date implied by the interval and last review time.
        expected_due_on: NaiveDate,
        /// Interval stored with the state when the mismatch was detected.
        interval_days: NonZeroU8,
    },
}

impl fmt::Display for CardStateInvariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntervalBelowMinimum { interval, minimum } => {
                write!(
                    f,
                    "interval {interval} days is below the minimum {minimum} days"
                )
            }
            Self::EaseFactorOutOfRange {
                ease_factor,
                minimum,
                maximum,
            } => write!(
                f,
                "ease factor {ease_factor} is outside the allowed range {minimum}..={maximum}"
            ),
            Self::DueDateBeforeLastReview {
                due_on,
                last_reviewed_on,
            } => write!(
                f,
                "due date {due_on} cannot be before last review {last_reviewed_on}"
            ),
            Self::DueDateMismatch {
                due_on,
                expected_due_on,
                interval_days,
            } => write!(
                f,
                "due date {due_on} does not equal last review plus interval ({expected_due_on} expected from {interval_days} days)"
            ),
        }
    }
}

impl std::error::Error for CardStateInvariantError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValidGrade;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn validate_accepts_state_created_via_apply_review() {
        let invariants = CardStateInvariants::default();
        let interval = NonZeroU8::new(2).expect("non-zero interval");
        let mut state = StoredCardState::new(naive_date(2024, 1, 1), interval, 2.5);
        let review_day = naive_date(2024, 1, 10);

        state.apply_review(ValidGrade::Four, review_day);

        invariants
            .validate(&state)
            .expect("state from apply_review should be valid");
    }

    #[test]
    fn interval_less_than_minimum_fails_validation() {
        let invariants =
            CardStateInvariants::new(NonZeroU8::new(5).expect("non-zero interval"), 1.3..=2.8);
        let mut state = StoredCardState::new(
            naive_date(2024, 1, 1),
            NonZeroU8::new(3).expect("non-zero interval"),
            2.5,
        );
        state.last_reviewed_on = Some(naive_date(2023, 12, 28));
        state.due_on = naive_date(2023, 12, 31);

        let err = invariants
            .validate(&state)
            .expect_err("interval should be below minimum");
        assert!(matches!(
            err,
            CardStateInvariantError::IntervalBelowMinimum { .. }
        ));
    }
}
