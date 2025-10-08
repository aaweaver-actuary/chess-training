use std::num::NonZeroU8;

use card_store::config::StorageConfig;
use card_store::memory::InMemoryCardStore;
use card_store::model::{CardState, EdgeInput, Position, ReviewRequest, UnlockRecord};
use card_store::store::{CardStore, StoreError};
use chrono::NaiveDate;

fn sample_position() -> Position {
    Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        0,
    )
    .expect("valid starting position")
}

fn sample_child_position() -> Position {
    Position::new(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        1,
    )
    .expect("valid child position")
}

#[test]
fn position_creation_requires_valid_side_to_move() {
    let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(matches!(result, Err(_)));
}

#[test]
fn position_creation_fails_with_missing_fields() {
    // Missing side to move, castling, en passant, halfmove, fullmove
    let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(matches!(result, Err(_)));
}

#[test]
fn position_creation_fails_with_invalid_characters() {
    // Invalid character 'X' in FEN
    let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1", 0);
    assert!(matches!(result, Err(_)));
}

#[test]
fn position_creation_fails_with_extra_whitespace() {
    // Extra whitespace between fields
    let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR   w KQkq - 0 1", 0);
    assert!(matches!(result, Err(_)));
}

#[test]
fn position_creation_fails_with_too_many_fields() {
    // Too many fields in FEN
    let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra", 0);
    assert!(matches!(result, Err(_)));
}

#[test]
fn upsert_position_is_idempotent() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let position = sample_position();

    let first = store.upsert_position(position.clone()).unwrap();
    let second = store.upsert_position(position.clone()).unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(first.fen, position.fen);
    assert_eq!(store.position_count().unwrap(), 1);
}

#[test]
fn inserting_edge_requires_parent_position() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let child = sample_child_position();
    store.upsert_position(child.clone()).unwrap();

    let edge_input = EdgeInput {
        parent_id: sample_position().id,
        move_uci: "e2e4".to_string(),
        move_san: "e4".to_string(),
        child_id: child.id,
    };

    let result = store.upsert_edge(edge_input.clone());
    assert!(
        matches!(result, Err(StoreError::MissingPosition { id }) if id == edge_input.parent_id)
    );
}

#[test]
fn due_cards_filter_out_future_entries() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let position = store.upsert_position(sample_position()).unwrap();
    let child = store.upsert_position(sample_child_position()).unwrap();
    let edge = store
        .upsert_edge(EdgeInput {
            parent_id: position.id,
            move_uci: "e2e4".to_string(),
            move_san: "e4".to_string(),
            child_id: child.id,
        })
        .unwrap();

    let past = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
    let future = NaiveDate::from_ymd_opt(2023, 12, 5).unwrap();

    store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(past, NonZeroU8::new(1).unwrap(), 2.5),
        )
        .unwrap();
    store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(future, NonZeroU8::new(1).unwrap(), 2.5),
        )
        .unwrap();

    let due_cards = store.fetch_due_cards("andy", past).unwrap();
    assert_eq!(due_cards.len(), 1);
    assert!(due_cards[0].state.due_on <= past);
}

#[test]
fn unlock_records_are_unique_per_day() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let edge_id = 42;
    let record = UnlockRecord {
        owner_id: "andy".to_string(),
        edge_id,
        unlocked_on: date,
    };

    store.record_unlock(record.clone()).unwrap();
    let second = store.record_unlock(record);
    assert!(
        matches!(second, Err(StoreError::DuplicateUnlock { edge, day }) if edge == edge_id && day == date)
    );
}

#[test]
fn reviews_update_due_date_using_grade_logic() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let position = store.upsert_position(sample_position()).unwrap();
    let child = store.upsert_position(sample_child_position()).unwrap();
    let edge = store
        .upsert_edge(EdgeInput {
            parent_id: position.id,
            move_uci: "e2e4".to_string(),
            move_san: "e4".to_string(),
            child_id: child.id,
        })
        .unwrap();

    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, NonZeroU8::new(1).unwrap(), 2.5),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 4,
        })
        .unwrap();

    assert!(updated_card.state.due_on > review_date);
    assert_eq!(updated_card.state.last_reviewed_on, Some(review_date));
    assert_eq!(updated_card.state.consecutive_correct, 1);
}
