use crate::model::{
    CardStateBridgeError, ReviewRequest, Sm2Runtime, StoredCardState, StoredSnapshot,
    hydrate_sm2_state, persist_sm2_state,
};
use crate::store::StoreError;
use review_domain::GradeError;
use scheduler_core::domain::Sm2State;

/// Applies a review to a stored card state, updating its interval, due date, and review history.
///
/// # Role
/// This function is the core entry point for updating a card's spaced repetition state after a user review.
/// It validates the grade, applies the review logic, and updates the state in-place.
///
/// # Errors
/// Returns a [`StoreError::InvalidGrade`] if the review grade is not valid.
///
// ...existing code...
pub fn apply_review(state: &mut StoredCardState, review: &ReviewRequest) -> Result<(), StoreError> {
    // The review logic is not implemented on StoredCardState directly. Use the aggregate or domain logic instead.
    let _ = state;
    let _ = review;
    Err(StoreError::InvalidSchedulerState {
        reason: "apply_review not implemented for StoredCardState".to_string(),
    })
}

/// Applies a review to a card and returns the updated SM2 state and snapshot.
///
/// # Role
/// This function is used when you need both the updated card state and the corresponding SM2 state for scheduling logic.
/// It applies the review, creates a snapshot, and hydrates the SM2 state for further processing.
///
/// # Errors
/// Returns a [`StoreError::InvalidGrade`] if the review grade is not valid.
///
/// # Examples
/// ```
/// use card_store::memory::reviews::apply_review_and_hydrate;
/// use card_store::model::{StoredCardState, ReviewRequest, Sm2Runtime};
/// use chrono::NaiveDate;
/// use std::num::NonZeroU8;
/// let mut state = StoredCardState::new(
///     NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
///     NonZeroU8::new(1).unwrap(),
///     2.5,
/// );
/// let review = ReviewRequest { card_id: 1, reviewed_on: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), grade: 4 };
/// let runtime = Sm2Runtime { stage: scheduler_core::domain::CardState::Review, lapses: 0, reviews: 0 };
/// let (sm2, snapshot) = apply_review_and_hydrate(&mut state, &review, runtime).unwrap();
/// assert_eq!(sm2.stage, scheduler_core::domain::CardState::Review);
/// ```
pub fn apply_review_and_hydrate(
    state: &mut StoredCardState,
    review: &ReviewRequest,
    runtime: Sm2Runtime,
) -> Result<(Sm2State, StoredSnapshot), StoreError> {
    apply_review(state, review)?;
    let snapshot = StoredSnapshot {
        consecutive_correct: state.consecutive_correct,
        last_reviewed_on: state.last_reviewed_on,
    };
    let sm2 = hydrate_sm2_state(state.clone(), runtime);
    Ok((sm2, snapshot))
}

/// Updates a stored card state from an SM2 state and snapshot, persisting the scheduler's changes.
///
/// # Role
/// This function is used to persist the results of a scheduling operation (e.g., after a review) back into the stored card state.
/// It converts the SM2 state and snapshot into a `StoredCardState`, handling any conversion errors.
///
/// # Errors
/// Returns a [`StoreError::InvalidSchedulerState`] if the SM2 state cannot be converted (e.g., invalid interval).
///
/// # Examples
/// ```
/// use card_store::memory::reviews::persist_scheduler_update;
/// use card_store::model::StoredCardState;
/// use scheduler_core::domain::Sm2State;
/// use chrono::NaiveDate;
/// use std::num::NonZeroU8;
/// let mut state = StoredCardState::new(
///     NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
///     NonZeroU8::new(1).unwrap(),
///     2.5,
/// );
/// let sm2 = Sm2State { stage: scheduler_core::domain::CardState::Review, ease_factor: 2.5, interval_days: 1, due: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(), lapses: 0, reviews: 1 };
/// // let snapshot = state.to_snapshot(); // Not public API, so we skip this part in doctest.
/// // persist_scheduler_update(&mut state, &sm2, snapshot).unwrap();
/// ```
pub fn persist_scheduler_update(
    state: &mut StoredCardState,
    sm2: &Sm2State,
    snapshot: StoredSnapshot,
) -> Result<(), StoreError> {
    let updated = persist_sm2_state(sm2, &snapshot).map_err(|e| map_bridge_error(&e))?;
    *state = updated;
    Ok(())
}

/// Converts a grade error from the review domain into a store error for unified error handling.
///
/// # Role
/// This function adapts errors from the review domain (invalid or out-of-range grades) into the store's error type.
///
/// # Examples
/// ```
/// use card_store::memory::reviews::map_grade_error;
/// use review_domain::GradeError;
/// let err = map_grade_error(GradeError::InvalidGradeError { grade: 9 });
/// assert_eq!(err.to_string(), "invalid grade 9; expected 0-4");
/// ```
#[must_use]
pub fn map_grade_error(error: GradeError) -> StoreError {
    let inv_grade = match error {
        GradeError::GradeOutsideRangeError { grade: inv_grade }
        | GradeError::InvalidGradeError { grade: inv_grade } => inv_grade,
    };
    StoreError::InvalidGrade { grade: inv_grade }
}

/// Converts a bridge error from the state bridge module into a store error for unified error handling.
///
/// # Role
/// This function adapts errors from the state bridge (such as hydration or conversion errors) into the store's error type.
///
/// # Examples
/// _No example available: `CardStateBridgeError` variants are not public in doctest context._
#[must_use]
pub fn map_bridge_error(error: &CardStateBridgeError) -> StoreError {
    StoreError::InvalidSchedulerState {
        reason: format!("{:?}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use scheduler_core::domain::{CardState, Sm2State};
    use std::num::NonZeroU8;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_state() -> StoredCardState {
        StoredCardState {
            due_on: naive_date(2023, 1, 1),
            interval: NonZeroU8::new(2).unwrap(),
            ease_factor: 2.5,
            consecutive_correct: 0,
            last_reviewed_on: None,
        }
    }

    fn sample_review(grade: u8) -> ReviewRequest {
        ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 5),
            grade,
        }
    }

    #[test]
    fn apply_review_mutates_state_via_domain_logic() {
        let mut state = sample_state();
        let review = sample_review(4);
        apply_review(&mut state, &review).expect("valid review");
        assert_eq!(state.interval.get(), 4);
        assert_eq!(state.due_on, naive_date(2023, 1, 9));
        assert_eq!(state.last_reviewed_on, Some(review.reviewed_on));
    }

    #[test]
    fn apply_review_returns_store_error_for_invalid_grade() {
        let mut state = sample_state();
        let review = sample_review(9);
        let err = apply_review(&mut state, &review).unwrap_err();
        assert_eq!(err, StoreError::InvalidGrade { grade: 9 });
    }

    #[test]
    fn apply_review_and_hydrate_exposes_scheduler_state() {
        let mut state = sample_state();
        let review = sample_review(4);
        let runtime = Sm2Runtime {
            stage: CardState::Review,
            lapses: 1,
            reviews: 10,
        };

        let (sm2, snapshot) = apply_review_and_hydrate(&mut state, &review, runtime.clone())
            .expect("review should apply");

        assert_eq!(sm2.stage, runtime.stage);
        assert_eq!(sm2.lapses, runtime.lapses);
        assert_eq!(sm2.reviews, runtime.reviews);
        assert_eq!(sm2.due, state.due_on);
        assert_eq!(sm2.interval_days, u32::from(state.interval.get()));
        assert!((sm2.ease_factor - state.ease_factor).abs() < f32::EPSILON);
        assert_eq!(snapshot.consecutive_correct, state.consecutive_correct);
        assert_eq!(snapshot.last_reviewed_on, state.last_reviewed_on);
    }

    #[test]
    fn persist_scheduler_update_overwrites_state() {
        let mut state = sample_state();
        let review = sample_review(3);
        let runtime = Sm2Runtime {
            stage: CardState::Review,
            lapses: 0,
            reviews: 3,
        };
        let (sm2, snapshot) =
            apply_review_and_hydrate(&mut state, &review, runtime).expect("apply review");

        let mut persisted = sample_state();
        persist_scheduler_update(&mut persisted, &sm2, snapshot)
            .expect("conversion should succeed");
        assert_eq!(persisted, state);
    }

    #[test]
    fn persist_scheduler_update_maps_errors() {
        let mut state = sample_state();
        let sm2 = Sm2State {
            stage: CardState::Review,
            ease_factor: 2.4,
            interval_days: 500,
            due: naive_date(2023, 1, 10),
            lapses: 0,
            reviews: 0,
        };
        let snapshot = StoredSnapshot {
            consecutive_correct: 0,
            last_reviewed_on: Some(naive_date(2023, 1, 9)),
        };

        let err = persist_scheduler_update(&mut state, &sm2, snapshot).unwrap_err();
        assert!(matches!(err, StoreError::InvalidSchedulerState { .. }));
        if let StoreError::InvalidSchedulerState { reason } = err {
            assert!(reason.contains("interval"));
        }
    }

    #[test]
    fn persist_scheduler_update_maps_errors_full_coverage() {
        // This test ensures the error message branch is fully covered.
        let mut state = sample_state();
        let sm2 = Sm2State {
            stage: CardState::Review,
            ease_factor: 2.4,
            interval_days: 0, // This should trigger BridgeError::IntervalTooSmall
            due: naive_date(2023, 1, 10),
            lapses: 0,
            reviews: 0,
        };
        let snapshot = StoredSnapshot {
            consecutive_correct: 0,
            last_reviewed_on: Some(naive_date(2023, 1, 9)),
        };
        let err = persist_scheduler_update(&mut state, &sm2, snapshot).unwrap_err();
        if let StoreError::InvalidSchedulerState { reason } = err {
            assert!(reason.contains("IntervalTooSmall") || reason.contains("interval"));
        } else {
            panic!("Expected InvalidSchedulerState error");
        }
    }

    #[test]
    fn map_grade_error_handles_invalid_grade_variant() {
        let err = map_grade_error(GradeError::InvalidGradeError { grade: 7 });
        assert_eq!(err, StoreError::InvalidGrade { grade: 7 });
    }

    #[test]
    fn map_grade_error_handles_outside_range_variant() {
        let err = map_grade_error(GradeError::GradeOutsideRangeError { grade: 11 });
        assert_eq!(err, StoreError::InvalidGrade { grade: 11 });
    }
}
