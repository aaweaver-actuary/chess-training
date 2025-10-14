use chrono::NaiveDate;
use review_domain::{Grade, StoredCardState};

fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

#[test]
fn next_interval_walks_all_grade_branches() {
    let interval = std::num::NonZeroU8::new(3).expect("non-zero interval");
    let state = StoredCardState::new(naive_date(2024, 1, 1), interval, 2.5);

    assert_eq!(state.next_interval(Grade::Zero).get(), 1);
    assert_eq!(state.next_interval(Grade::One).get(), 1);
    assert_eq!(state.next_interval(Grade::Two).get(), 3);
    assert_eq!(state.next_interval(Grade::Three).get(), 4);
    assert_eq!(state.next_interval(Grade::Four).get(), 6);
}

#[test]
fn apply_review_mutates_state_consistently() {
    let interval = std::num::NonZeroU8::new(2).expect("non-zero interval");
    let mut state = StoredCardState::new(naive_date(2024, 2, 2), interval, 2.4);

    let review_day = naive_date(2024, 2, 10);
    state.consecutive_correct = 1;
    state.apply_review(Grade::Four, review_day);

    assert_eq!(state.interval.get(), 4);
    assert_eq!(state.due_on, naive_date(2024, 2, 14));
    assert_eq!(state.last_reviewed_on, Some(review_day));
    assert_eq!(state.consecutive_correct, 2);
    assert!(state.ease_factor >= 2.4);
}
