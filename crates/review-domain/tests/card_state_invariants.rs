use std::num::NonZeroU8;

use chrono::NaiveDate;
use review_domain::StoredCardState;
use review_domain::card_state::invariants::{CardStateInvariantError, CardStateInvariants};

fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

fn base_state() -> StoredCardState {
    let interval = NonZeroU8::new(3).expect("non-zero interval");
    let mut state = StoredCardState::new(naive_date(2024, 1, 1), interval, 2.5);
    state.consecutive_correct = 2;
    state.last_reviewed_on = Some(naive_date(2023, 12, 25));
    state.due_on = naive_date(2023, 12, 28);
    state
}

#[test]
fn default_invariants_match_expected_ranges() {
    let invariants = CardStateInvariants::default();
    assert_eq!(invariants.min_interval_days().get(), 1);
    assert_eq!(invariants.ease_factor_bounds(), (1.3, 2.8));
}

#[test]
fn validate_rejects_due_date_before_last_review() {
    let invariants = CardStateInvariants::default();
    let mut state = base_state();
    state.due_on = naive_date(2023, 12, 24);

    let err = invariants
        .validate(&state)
        .expect_err("due date before review should fail");
    assert!(matches!(
        err,
        CardStateInvariantError::DueDateBeforeLastReview { .. }
    ));
}

#[test]
fn validate_rejects_ease_factor_outside_range() {
    let invariants = CardStateInvariants::default();
    let mut state = base_state();
    state.ease_factor = 3.1;

    let err = invariants
        .validate(&state)
        .expect_err("ease factor outside range should fail");
    assert!(matches!(
        err,
        CardStateInvariantError::EaseFactorOutOfRange { .. }
    ));
}

#[test]
fn validate_rejects_due_date_not_matching_interval() {
    let invariants = CardStateInvariants::default();
    let mut state = base_state();
    state.due_on = naive_date(2023, 12, 30);

    let err = invariants
        .validate(&state)
        .expect_err("due date that ignores interval should fail");
    assert!(matches!(
        err,
        CardStateInvariantError::DueDateMismatch { .. }
    ));
}

#[test]
fn validate_accepts_consistent_state() {
    let invariants = CardStateInvariants::default();
    let mut state = base_state();
    state.due_on = naive_date(2023, 12, 28);

    invariants
        .validate(&state)
        .expect("state that meets invariants should validate");
}
