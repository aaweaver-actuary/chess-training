use std::num::NonZeroU8;

use chrono::NaiveDate;

use review_domain::{
    CardAggregate, CardId, CardKind, EdgeId, GradeError, IdConversionError, LearnerId, OpeningCard,
    ReviewRequest, StoredCardState, TacticCard, TacticId, ValidGrade,
};

fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

fn sample_state() -> StoredCardState {
    let interval = NonZeroU8::new(2).expect("non-zero interval");
    StoredCardState::new(naive_date(2024, 1, 10), interval, 2.3)
}

#[test]
fn new_opening_aggregate_wraps_underlying_card() {
    let state = sample_state();
    let aggregate = CardAggregate::new_opening(
        CardId::new(11),
        LearnerId::new(29),
        EdgeId::new(97),
        state.clone(),
    );

    assert_eq!(aggregate.id(), CardId::new(11));
    assert_eq!(aggregate.owner_id(), LearnerId::new(29));
    match aggregate.kind() {
        CardKind::Opening(payload) => assert_eq!(*payload, OpeningCard::new(EdgeId::new(97))),
        CardKind::Tactic(_) => panic!("expected opening card"),
    }
    assert_eq!(aggregate.state(), &state);

    let card = aggregate.as_card();
    assert_eq!(card.id, CardId::new(11));
    assert_eq!(card.owner_id, LearnerId::new(29));
    assert_eq!(
        card.kind,
        CardKind::Opening(OpeningCard::new(EdgeId::new(97)))
    );
}

#[test]
fn new_tactic_aggregate_wraps_underlying_card() {
    let aggregate = CardAggregate::new_tactic(
        CardId::new(7),
        LearnerId::new(5),
        TacticId::new(321),
        sample_state(),
    );
    match aggregate.kind() {
        CardKind::Tactic(payload) => assert_eq!(*payload, TacticCard::new(TacticId::new(321))),
        CardKind::Opening(_) => panic!("expected tactic card"),
    }
}

#[test]
fn apply_review_updates_internal_state() {
    let interval = NonZeroU8::new(3).unwrap();
    let state = StoredCardState::new(naive_date(2024, 2, 1), interval, 2.5);
    let mut aggregate = CardAggregate::new_tactic(
        CardId::new(99),
        LearnerId::new(42),
        TacticId::new(88),
        state,
    );
    let review_day = naive_date(2024, 2, 10);

    aggregate.apply_valid_grade(ValidGrade::Four, review_day);
    let updated = aggregate.state();
    assert_eq!(updated.interval.get(), 6);
    assert_eq!(updated.due_on, naive_date(2024, 2, 16));
    assert_eq!(updated.last_reviewed_on, Some(review_day));
    assert_eq!(updated.consecutive_correct, 1);
}

#[test]
fn apply_review_validates_raw_grade() {
    let mut aggregate = CardAggregate::new_tactic(
        CardId::new(33),
        LearnerId::new(28),
        TacticId::new(14),
        sample_state(),
    );
    let reviewed_on = naive_date(2024, 3, 1);

    aggregate
        .apply_review(4, reviewed_on)
        .expect("grade should be accepted");
    assert_eq!(aggregate.state().last_reviewed_on, Some(reviewed_on));

    let mut aggregate = CardAggregate::new_tactic(
        CardId::new(34),
        LearnerId::new(28),
        TacticId::new(15),
        sample_state(),
    );
    let original_state = aggregate.state().clone();
    let error = aggregate
        .apply_review(9, reviewed_on)
        .expect_err("grade should be rejected");
    assert_eq!(error, GradeError::GradeOutsideRangeError { grade: 9 });
    assert_eq!(aggregate.state(), &original_state);
}

#[test]
fn apply_review_request_delegates_to_helper() {
    let mut aggregate = CardAggregate::new_opening(
        CardId::new(55),
        LearnerId::new(12),
        EdgeId::new(77),
        sample_state(),
    );
    let reviewed_on = naive_date(2024, 4, 2);
    let review = ReviewRequest {
        card_id: aggregate.id().get(),
        reviewed_on,
        grade: 4,
    };

    aggregate
        .apply_review_request(&review)
        .expect("grade should be accepted");

    assert_eq!(aggregate.state().last_reviewed_on, Some(reviewed_on));
    assert_eq!(aggregate.state().due_on, naive_date(2024, 4, 6));
}

#[test]
fn into_card_recovers_generic_representation() {
    let state = sample_state();
    let aggregate = CardAggregate::new_opening(
        CardId::new(1),
        LearnerId::new(2),
        EdgeId::new(3),
        state.clone(),
    );

    let card = aggregate.into_card();
    assert_eq!(
        card,
        review_domain::Card {
            id: CardId::new(1),
            owner_id: LearnerId::new(2),
            kind: CardKind::Opening(OpeningCard::new(EdgeId::new(3))),
            state,
        }
    );
}

#[test]
fn overflow_values_fail_to_convert_into_card_ids() {
    let overflow =
        CardId::try_from(u128::from(u64::MAX) + 1).expect_err("overflowed identifiers should fail");
    assert!(matches!(
        overflow,
        IdConversionError::Overflow { value } if value == u128::from(u64::MAX) + 1
    ));
}
