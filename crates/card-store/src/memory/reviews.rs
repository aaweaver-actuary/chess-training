use std::num::NonZeroU8;

use chrono::{Duration, NaiveDate};

use crate::model::{CardState, ReviewRequest};
use crate::store::StoreError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ReviewTransition {
    interval: NonZeroU8,
    ease: f32,
    streak: u32,
    due_on: NaiveDate,
}

pub(super) fn apply_review(
    state: &mut CardState,
    review: &ReviewRequest,
) -> Result<(), StoreError> {
    let transition = derive_review_transition(state, review)?;
    commit_review_transition(state, review.reviewed_on, transition);
    Ok(())
}

fn derive_review_transition(
    state: &CardState,
    review: &ReviewRequest,
) -> Result<ReviewTransition, StoreError> {
    validate_grade(review.grade)?;
    let interval = interval_after_grade(state.interval, review.grade);
    let ease = ease_after_grade(state.ease_factor, review.grade);
    Ok(finalize_transition(state, review, interval, ease))
}

fn validate_grade(grade: u8) -> Result<(), StoreError> {
    if grade > 4 {
        Err(StoreError::InvalidGrade { grade })
    } else {
        Ok(())
    }
}

fn interval_after_grade(interval: NonZeroU8, grade: u8) -> NonZeroU8 {
    match grade {
        0 | 1 => NonZeroU8::new(1).unwrap(),
        2 => interval,
        3 => {
            let next = interval.get().saturating_add(1);
            NonZeroU8::new(next).unwrap()
        }
        4 => {
            let doubled = interval.get().saturating_mul(2);
            NonZeroU8::new(doubled).unwrap()
        }
        _ => unreachable!(),
    }
}

fn ease_after_grade(current: f32, grade: u8) -> f32 {
    let delta = ease_delta_for_grade(grade);
    (current + delta).clamp(1.3, 2.8)
}

fn ease_delta_for_grade(grade: u8) -> f32 {
    match grade {
        0 => -0.3,
        1 => -0.15,
        2 => -0.05,
        3 => 0.0,
        4 => 0.15,
        _ => unreachable!(),
    }
}

fn finalize_transition(
    state: &CardState,
    review: &ReviewRequest,
    interval: NonZeroU8,
    ease: f32,
) -> ReviewTransition {
    let streak = next_streak(state.consecutive_correct, review.grade);
    let due_on = due_date_for_review(review.reviewed_on, interval);
    ReviewTransition {
        interval,
        ease,
        streak,
        due_on,
    }
}

fn next_streak(current: u32, grade: u8) -> u32 {
    if grade >= 3 {
        current.saturating_add(1)
    } else {
        0
    }
}

fn due_date_for_review(reviewed_on: NaiveDate, interval: NonZeroU8) -> NaiveDate {
    reviewed_on + Duration::days(i64::from(interval.get()))
}

fn commit_review_transition(
    state: &mut CardState,
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

    fn sample_state() -> CardState {
        CardState::new(naive_date(2023, 1, 1), NonZeroU8::new(1).unwrap(), 2.5)
    }

    fn sample_review(grade: u8) -> ReviewRequest {
        ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 1),
            grade,
        }
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
    fn derive_review_transition_validates_grade() {
        let state = sample_state();
        let review = sample_review(7);
        let err = derive_review_transition(&state, &review).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 7));
    }

    #[test]
    fn validate_grade_rejects_out_of_range_values() {
        let err = validate_grade(5).unwrap_err();
        assert!(matches!(err, StoreError::InvalidGrade { grade } if grade == 5));
        assert!(validate_grade(4).is_ok());
    }

    #[test]
    fn interval_after_grade_adjusts_spacing() {
        let interval = NonZeroU8::new(3).unwrap();
        assert_eq!(interval_after_grade(interval, 2), interval);
        assert_eq!(interval_after_grade(interval, 4).get(), 6);
    }

    #[test]
    fn ease_delta_for_grade_matches_expectations() {
        assert!(ease_delta_for_grade(0) < 0.0);
        assert!(ease_delta_for_grade(4) > 0.0);
    }

    #[test]
    fn ease_after_grade_clamps_results() {
        let eased = ease_after_grade(2.7, 4);
        assert!((eased - 2.8).abs() < f32::EPSILON);
    }

    #[test]
    fn next_streak_tracks_correct_answers() {
        assert_eq!(next_streak(2, 4), 3);
        assert_eq!(next_streak(5, 1), 0);
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
        let transition = finalize_transition(&state, &review, interval, 2.3);
        assert_eq!(transition.interval, interval);
        assert_eq!(transition.ease, 2.3);
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
        assert_eq!(state.ease_factor, 2.1);
        assert_eq!(state.due_on, naive_date(2023, 1, 4));
    }
}
