use chrono::{Duration, NaiveDate};
use std::num::{NonZero, NonZeroU8};

use crate::model::{ReviewRequest, StoredCardState};
use crate::store::StoreError;
use review_domain::{GradeError, ValidGrade};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ReviewTransition {
    interval: NonZeroU8,
    ease: f32,
    streak: u32,
    due_on: NaiveDate,
}

pub(super) fn apply_review(
    state: &mut StoredCardState,
    review: &ReviewRequest,
) -> Result<(), StoreError> {
    let transition = derive_review_transition(state, review);
    commit_review_transition(state, review.reviewed_on, transition);
    Ok(())
}

fn derive_review_transition(
    state: &StoredCardState,
    review: &ReviewRequest,
) -> Result<ReviewTransition, GradeError> {
    let grade = ValidGrade::new(review.grade)?;
    let interval = interval_after_grade_validated(state.interval, grade.as_u8());
    let ease = ease_after_grade_validated(state.ease_factor, grade.as_u8());
    Ok(finalize_transition(state, review, grade, interval, ease))
}

#[cfg_attr(not(test), allow(dead_code))]
fn interval_after_grade(_interval: NonZeroU8, grade: u8) -> Result<NonZeroU8, GradeError> {
    let valid_grade = ValidGrade::new(grade)?;
    Ok(NonZero::new(valid_grade.to_interval_increment()).unwrap())
}

fn interval_after_grade_validated(interval: NonZeroU8, grade: u8) -> NonZeroU8 {
    if grade <= 1 {
        NonZeroU8::new(1).unwrap()
    } else if grade == 2 {
        interval
    } else if grade == 3 {
        let next = interval.get().saturating_add(1);
        NonZeroU8::new(next).unwrap()
    } else {
        let doubled = interval.get().saturating_mul(2);
        NonZeroU8::new(doubled).unwrap()
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn ease_after_grade(current: f32, grade: u8) -> Result<f32, StoreError> {
    validate_grade(grade)?;
    Ok(ease_after_grade_validated(current, grade))
}

fn ease_after_grade_validated(current: f32, grade: u8) -> f32 {
    let delta = ease_delta_for_grade_validated(grade);
    (current + delta).clamp(1.3, 2.8)
}

#[cfg_attr(not(test), allow(dead_code))]
fn ease_delta_for_grade(grade: u8) -> Result<f32, StoreError> {
    validate_grade(grade)?;
    Ok(ease_delta_for_grade_validated(grade))
}

fn ease_delta_for_grade_validated(grade: u8) -> f32 {
    if grade == 0 {
        -0.3
    } else if grade == 1 {
        -0.15
    } else if grade == 2 {
        -0.05
    } else if grade == 3 {
        0.0
    } else {
        0.15
    }
}

fn finalize_transition(
    state: &StoredCardState,
    review: &ReviewRequest,
    grade: ValidGrade,
    interval: NonZeroU8,
    ease: f32,
) -> ReviewTransition {
    let streak = next_streak(state.consecutive_correct, grade);
    let due_on = due_date_for_review(review.reviewed_on, interval);
    ReviewTransition {
        interval,
        ease,
        streak,
        due_on,
    }
}

fn next_streak(current: u32, grade: ValidGrade) -> u32 {
    if grade.is_correct() {
        current.saturating_add(1)
    } else {
        0
    }
}

fn due_date_for_review(reviewed_on: NaiveDate, interval: NonZeroU8) -> NaiveDate {
    reviewed_on + Duration::days(i64::from(interval.get()))
}

fn commit_review_transition(
    state: &mut StoredCardState,
    reviewed_on: NaiveDate,
    transition: ReviewTransition,
) {
    state.interval = transition.interval;
    state.ease_factor = transition.ease;
    state.consecutive_correct = transition.streak;
    state.last_reviewed_on = Some(reviewed_on);
    state.due_on = transition.due_on;
}

#[cfg(test)]
mod tests {
    use super::*;
    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_state() -> StoredCardState {
        StoredCardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5)
    }

    fn sample_review(grade: u8) -> ReviewRequest {
        ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 1),
            grade,
        }
    }

    fn valid_grade(grade: u8) -> ValidGrade {
        ValidGrade::try_from(grade).expect("valid grade")
    }

    #[test]
    fn apply_review_updates_state_fields() {
        let mut state = sample_state();
        let review = sample_review(3);
        apply_review(&mut state, &review).expect("valid review");
        assert_eq!(state.last_reviewed_on, Some(naive_date(2023, 1, 1)));
        assert_eq!(state.consecutive_correct, 1);
    }

    #[test]
    fn apply_review_propagates_invalid_grade_error() {
        let mut state = sample_state();
        let review = sample_review(9);
        let err = apply_review(&mut state, &review).unwrap_err();
        assert_eq!(err, StoreError::InvalidGrade { grade: 9 });
    }

    #[test]
    fn derive_review_transition_validates_grade() {
        let state = sample_state();
        let review = sample_review(7);
        let err = derive_review_transition(&state, &review).unwrap_err();
        assert_eq!(err, StoreError::InvalidGrade { grade: 7 });
    }

    #[test]
    fn derive_review_transition_returns_expected_values() {
        let state = sample_state();
        let review = sample_review(3);
        let transition = derive_review_transition(&state, &review).expect("valid transition");
        assert_eq!(transition.interval.get(), 2);
        assert!((transition.ease - 2.5).abs() < f32::EPSILON);
        assert_eq!(transition.streak, 1);
    }

    #[test]
    fn validate_grade_rejects_out_of_range_values() {
        let err = validate_grade(5).unwrap_err();
        assert_eq!(err, StoreError::InvalidGrade { grade: 5 });
        assert!(validate_grade(4).is_ok());
    }

    fn get_clean_interval_after_grade(interval: NonZeroU8, grade: ValidGrade) -> u8 {
        interval_after_grade(interval, grade.as_u8())
            .expect("valid grade")
            .get()
    }

    #[test]
    fn interval_after_grade_adjusts_spacing() {
        let interval = NonZeroU8::new(3).unwrap();
        assert_eq!(get_clean_interval_after_grade(interval, valid_grade(0)), 1);
        assert_eq!(get_clean_interval_after_grade(interval, valid_grade(1)), 1);
        assert_eq!(get_clean_interval_after_grade(interval, valid_grade(2)), 1);
        assert_eq!(get_clean_interval_after_grade(interval, valid_grade(3)), 4);
        assert_eq!(get_clean_interval_after_grade(interval, valid_grade(4)), 6);
    }

    #[test]
    fn ease_delta_for_grade_matches_expectations() {
        assert!(ease_delta_for_grade(valid_grade(0).as_u8()).unwrap() < 0.0);
        assert!(ease_delta_for_grade(valid_grade(2).as_u8()).unwrap() < 0.0);
        assert!(ease_delta_for_grade(valid_grade(4).as_u8()).unwrap() > 0.0);
    }

    #[test]
    fn ease_after_grade_clamps_results() {
        let eased_high = ease_after_grade(2.7, 4).unwrap();
        assert!((eased_high - 2.8).abs() < f32::EPSILON);

        let eased_low = ease_after_grade(1.4, 0).unwrap();
        assert!((eased_low - 1.3).abs() < f32::EPSILON);

        let eased_mid = ease_after_grade(2.0, 3).unwrap();
        assert!((eased_mid - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ease_after_grade_errors_on_out_of_range_values() {
        let err = ease_after_grade(1.5, 9).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 9));
    }

    #[test]
    fn interval_after_grade_errors_on_out_of_range_values() {
        let interval = NonZeroU8::new(3).unwrap();
        let err = interval_after_grade(interval, 9).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 9));
    }

    #[test]
    fn ease_delta_for_grade_errors_on_out_of_range_values() {
        let err = ease_delta_for_grade(9).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 9));
    }

    #[test]
    fn ease_after_grade_clamps_extremes_correctly() {
        let eased = ease_after_grade(1.2, valid_grade(0).as_u8()).expect("valid");
        assert!((eased - 1.3).abs() < f32::EPSILON);
        let eased = ease_after_grade(2.7, valid_grade(4).as_u8()).expect("valid");
        assert!((eased - 2.8).abs() < f32::EPSILON);
    }

    #[test]
    fn next_streak_tracks_correct_answers() {
        assert_eq!(next_streak(2, valid_grade(4)), 3);
        assert_eq!(next_streak(5, valid_grade(1)), 0);
    }

    #[test]
    fn due_date_for_review_offsets_by_interval() {
        let date = naive_date(2023, 1, 1);
        let interval = NonZeroU8::new(3).unwrap();
        assert_eq!(due_date_for_review(date, interval), naive_date(2023, 1, 4));
    }

    #[test]
    fn finalize_transition_collects_components() {
        let state = sample_state();
        let review = sample_review(3);
        let interval = NonZeroU8::new(2).unwrap();
        let transition = finalize_transition(&state, &review, valid_grade(3), interval, 2.3);
        assert_eq!(transition.interval, interval);
        assert!((transition.ease - 2.3).abs() < f32::EPSILON);
        assert_eq!(transition.due_on, naive_date(2023, 1, 3));
    }

    #[test]
    fn commit_review_transition_updates_state() {
        let mut state = sample_state();
        let transition = ReviewTransition {
            interval: NonZeroU8::new(3).unwrap(),
            ease: 2.1,
            streak: 4,
            due_on: naive_date(2023, 1, 4),
        };
        commit_review_transition(&mut state, naive_date(2023, 1, 2), transition);
        assert_eq!(state.interval.get(), 3);
        assert!((state.ease_factor - 2.1).abs() < f32::EPSILON);
        assert_eq!(state.due_on, naive_date(2023, 1, 4));
    }

    #[test]
    fn apply_review_updates_state() {
        let mut state = sample_state();
        let review = sample_review(3);
        apply_review(&mut state, &review).unwrap();
        assert_eq!(state.consecutive_correct, 1);
        assert_eq!(state.last_reviewed_on, Some(review.reviewed_on));
    }

    #[test]
    fn apply_review_returns_error_for_invalid_grade() {
        let mut state = sample_state();
        let review = sample_review(9);
        let err = apply_review(&mut state, &review).unwrap_err();
        assert_eq!(err, StoreError::InvalidGrade { grade: 9 });
    }

    #[test]
    fn validate_grade_handles_bounds() {
        assert!(validate_grade(4).is_ok());
        assert_eq!(
            validate_grade(6).unwrap_err(),
            StoreError::InvalidGrade { grade: 6 }
        );
    }

    #[test]
    fn interval_after_grade_covers_branches() {
        let base = NonZeroU8::new(3).unwrap();
        assert_eq!(
            interval_after_grade(base, 0).unwrap(),
            NonZeroU8::new(1).unwrap()
        );
        assert_eq!(
            interval_after_grade(base, 1).unwrap(),
            NonZeroU8::new(1).unwrap()
        );
        assert_eq!(interval_after_grade(base, 2).unwrap(), base);
        assert_eq!(
            interval_after_grade(base, 3).unwrap(),
            NonZeroU8::new(4).unwrap()
        );
        assert_eq!(
            interval_after_grade(base, 4).unwrap(),
            NonZeroU8::new(6).unwrap()
        );
    }

    #[test]
    fn ease_delta_for_grade_covers_values() {
        assert!(ease_delta_for_grade(0).unwrap() < 0.0);
        assert!(ease_delta_for_grade(1).unwrap() < 0.0);
        assert!(ease_delta_for_grade(2).unwrap() < 0.0);
        assert_eq!(ease_delta_for_grade(3).unwrap(), 0.0);
        assert!(ease_delta_for_grade(4).unwrap() > 0.0);
    }

    #[test]
    fn ease_after_grade_clamps_extremes() {
        assert_eq!(ease_after_grade(1.2, 0).unwrap(), 1.3);
        assert_eq!(ease_after_grade(2.9, 4).unwrap(), 2.8);
    }

    #[test]
    fn finalize_transition_collects_components_part2() {
        let state = sample_state();
        let review = sample_review(3);
        let interval = NonZeroU8::new(2).unwrap();
        let grade = valid_grade(3);
        let transition = finalize_transition(&state, &review, grade, interval, 2.1);
        assert_eq!(transition.interval, interval);
        assert!((transition.ease - 2.1).abs() < f32::EPSILON);
        assert_eq!(transition.due_on, review.reviewed_on + Duration::days(2));
    }

    #[test]
    fn next_streak_and_commit_review_transition() {
        let mut state = sample_state();
        let transition = ReviewTransition {
            interval: NonZeroU8::new(4).unwrap(),
            ease: 2.0,
            streak: next_streak(5, valid_grade(4)),
            due_on: NaiveDate::from_ymd_opt(2023, 1, 5).unwrap(),
        };
        commit_review_transition(&mut state, transition.due_on, transition);
        assert_eq!(state.interval.get(), 4);
        assert_eq!(state.consecutive_correct, 6);
        assert_eq!(state.due_on, NaiveDate::from_ymd_opt(2023, 1, 5).unwrap());
    }

    #[test]
    fn due_date_for_review_offsets_days() {
        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let interval = NonZeroU8::new(3).unwrap();
        assert_eq!(
            due_date_for_review(start, interval),
            start + Duration::days(3)
        );
    }

    #[test]
    fn interval_after_grade_panics_on_out_of_range_values() {
        let interval = NonZeroU8::new(3).unwrap();
        assert!(interval_after_grade(interval, 9).is_err());
    }

    #[test]
    fn ease_delta_for_grade_panics_on_out_of_range_values() {
        assert!(ease_delta_for_grade(9).is_err());
    }
}
