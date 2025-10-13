use crate::model::{
    CardStateBridgeError, ReviewRequest, Sm2Runtime, StoredCardState, StoredSnapshot,
    hydrate_sm2_state, persist_sm2_state,
};
use crate::store::StoreError;
use review_domain::{GradeError, ValidGrade};
use scheduler_core::domain::Sm2State;

pub(super) fn apply_review(
    state: &mut StoredCardState,
    review: &ReviewRequest,
) -> Result<(), StoreError> {
    let grade = ValidGrade::new(review.grade)
        .map_err(|error: review_domain::GradeError| map_grade_error(error))?;
    state.apply_review(grade, review.reviewed_on);
    Ok(())
}

pub(super) fn apply_review_and_hydrate(
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

pub(super) fn persist_scheduler_update(
    state: &mut StoredCardState,
    sm2: &Sm2State,
    snapshot: StoredSnapshot,
) -> Result<(), StoreError> {
    let updated = persist_sm2_state(sm2, snapshot).map_err(|error| map_bridge_error(error))?;
    *state = updated;
    Ok(())
}

fn map_grade_error(error: GradeError) -> StoreError {
    let grade = match error {
        GradeError::GradeOutsideRangeError { grade } | GradeError::InvalidGradeError { grade } => {
            grade
        }
    };
    StoreError::InvalidGrade { grade }
}

fn map_bridge_error(error: CardStateBridgeError) -> StoreError {
    StoreError::InvalidSchedulerState {
        reason: error.to_string(),
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
        StoredCardState::new(naive_date(2023, 1, 1), NonZeroU8::new(2).unwrap(), 2.5)
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

        let (sm2, snapshot) =
            apply_review_and_hydrate(&mut state, &review, runtime).expect("review should apply");

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
