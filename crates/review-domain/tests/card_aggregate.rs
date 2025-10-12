use std::num::NonZeroU8;

use chrono::NaiveDate;

use review_domain::{
    CardAggregate, CardKind, OpeningCard, StoredCardState, TacticCard, ValidGrade,
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
    let aggregate = CardAggregate::new_opening(11, 29, 97, state.clone());

    assert_eq!(aggregate.id(), 11);
    assert_eq!(aggregate.owner_id(), 29);
    match aggregate.kind() {
        CardKind::Opening(payload) => assert_eq!(*payload, OpeningCard::new(97)),
        CardKind::Tactic(_) => panic!("expected opening card"),
    }
    assert_eq!(aggregate.state(), &state);

    let card = aggregate.as_card();
    assert_eq!(card.id, 11);
    assert_eq!(card.owner_id, 29);
    assert_eq!(card.kind, CardKind::Opening(OpeningCard::new(97)));
}

#[test]
fn new_tactic_aggregate_wraps_underlying_card() {
    let aggregate = CardAggregate::new_tactic(7, 5, 321, sample_state());
    match aggregate.kind() {
        CardKind::Tactic(payload) => assert_eq!(*payload, TacticCard::new(321)),
        CardKind::Opening(_) => panic!("expected tactic card"),
    }
}

#[test]
fn apply_review_updates_internal_state() {
    let interval = NonZeroU8::new(3).unwrap();
    let state = StoredCardState::new(naive_date(2024, 2, 1), interval, 2.5);
    let mut aggregate = CardAggregate::new_tactic(99, 42, 88, state);
    let review_day = naive_date(2024, 2, 10);

    aggregate.apply_review(ValidGrade::Four, review_day);
    let updated = aggregate.state();
    assert_eq!(updated.interval.get(), 6);
    assert_eq!(updated.due_on, naive_date(2024, 2, 16));
    assert_eq!(updated.last_reviewed_on, Some(review_day));
    assert_eq!(updated.consecutive_correct, 1);
}

#[test]
fn into_card_recovers_generic_representation() {
    let state = sample_state();
    let aggregate = CardAggregate::new_opening(1, 2, 3, state.clone());

    let card = aggregate.into_card();
    assert_eq!(
        card,
        review_domain::Card {
            id: 1,
            owner_id: 2,
            kind: CardKind::Opening(OpeningCard::new(3)),
            state,
        }
    );
}
