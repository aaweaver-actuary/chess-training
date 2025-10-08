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
    assert!(result.is_err());
}

#[test]
fn position_creation_fails_with_missing_fields() {
    // Missing side to move, castling, en passant, halfmove, fullmove
    let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(result.is_err());
}

#[test]
#[should_panic]
fn position_creation_fails_with_invalid_characters() {
    // Invalid character 'X' in FEN
    let result = Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1",
        0,
    );
    assert!(result.is_err());
}

#[test]
#[should_panic]
fn position_creation_fails_with_extra_whitespace() {
    // Extra whitespace between fields
    let result = Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR   w KQkq - 0 1",
        0,
    );
    assert!(result.is_err());
}

// This test should panic:
#[test]
#[should_panic]
fn position_creation_fails_with_too_many_fields() {
    // Too many fields in FEN
    let result = Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra",
        0,
    );
    assert!(result.is_err());
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
fn inserting_edge_requires_child_position() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let parent = sample_position();
    store.upsert_position(parent.clone()).unwrap();

    let edge_input = EdgeInput {
        parent_id: parent.id,
        move_uci: "e2e4".to_string(),
        move_san: "e4".to_string(),
        child_id: sample_child_position().id,
    };

    let result = store.upsert_edge(edge_input.clone());
    assert!(matches!(result, Err(StoreError::MissingPosition { id }) if id == edge_input.child_id));
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
fn unlock_same_edge_on_different_days() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let edge_id = 42;
    let day1 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let day2 = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();

    let record1 = UnlockRecord {
        owner_id: "andy".to_string(),
        edge_id,
        unlocked_on: day1,
    };
    let record2 = UnlockRecord {
        owner_id: "andy".to_string(),
        edge_id,
        unlocked_on: day2,
    };

    store.record_unlock(record1).unwrap();
    let result = store.record_unlock(record2);
    assert!(
        result.is_ok(),
        "Same edge should be unlockable on different days"
    );
}

#[test]
fn unlock_different_edges_on_same_day() {
    let store = InMemoryCardStore::new(StorageConfig::default());
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let edge_id1 = 42;
    let edge_id2 = 43;

    let record1 = UnlockRecord {
        owner_id: "andy".to_string(),
        edge_id: edge_id1,
        unlocked_on: date,
    };
    let record2 = UnlockRecord {
        owner_id: "andy".to_string(),
        edge_id: edge_id2,
        unlocked_on: date,
    };

    store.record_unlock(record1).unwrap();
    let result = store.record_unlock(record2);
    assert!(
        result.is_ok(),
        "Different edges should be unlockable on the same day"
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

#[test]
fn grade_0_resets_interval_and_decreases_ease_factor() {
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
    let initial_interval = NonZeroU8::new(5).unwrap();
    let initial_ease = 2.5;
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 0,
        })
        .unwrap();

    // Grade 0: interval resets to 1
    assert_eq!(updated_card.state.interval.get(), 1);
    // Grade 0: ease_factor decreases by -0.3, clamped to 1.3 minimum
    assert_eq!(updated_card.state.ease_factor, 2.2); // 2.5 - 0.3 = 2.2
    // Grade 0: consecutive_correct resets to 0
    assert_eq!(updated_card.state.consecutive_correct, 0);
    assert_eq!(updated_card.state.last_reviewed_on, Some(review_date));
    // Due date should be review_date + 1 day
    assert_eq!(
        updated_card.state.due_on,
        review_date + chrono::Duration::days(1)
    );
}

#[test]
fn grade_1_resets_interval_with_smaller_ease_penalty() {
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
    let initial_interval = NonZeroU8::new(10).unwrap();
    let initial_ease = 2.0;
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 1,
        })
        .unwrap();

    // Grade 1: interval resets to 1
    assert_eq!(updated_card.state.interval.get(), 1);
    // Grade 1: ease_factor decreases by -0.15
    assert_eq!(updated_card.state.ease_factor, 1.85); // 2.0 - 0.15 = 1.85
    // Grade 1: consecutive_correct resets to 0
    assert_eq!(updated_card.state.consecutive_correct, 0);
    assert_eq!(updated_card.state.last_reviewed_on, Some(review_date));
}

#[test]
fn grade_2_maintains_interval_with_small_ease_penalty() {
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
    let initial_interval = NonZeroU8::new(7).unwrap();
    let initial_ease = 2.5;
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 2,
        })
        .unwrap();

    // Grade 2: interval remains the same
    assert_eq!(updated_card.state.interval.get(), 7);
    // Grade 2: ease_factor decreases by -0.05
    assert_eq!(updated_card.state.ease_factor, 2.45); // 2.5 - 0.05 = 2.45
    // Grade 2: consecutive_correct resets to 0
    assert_eq!(updated_card.state.consecutive_correct, 0);
    assert_eq!(updated_card.state.last_reviewed_on, Some(review_date));
    // Due date should be review_date + 7 days
    assert_eq!(
        updated_card.state.due_on,
        review_date + chrono::Duration::days(7)
    );
}

#[test]
fn grade_3_increments_interval_and_streak() {
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
    let initial_interval = NonZeroU8::new(3).unwrap();
    let initial_ease = 2.5;
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 3,
        })
        .unwrap();

    // Grade 3: interval increments by 1
    assert_eq!(updated_card.state.interval.get(), 4); // 3 + 1 = 4
    // Grade 3: ease_factor remains unchanged (delta = 0.0)
    assert_eq!(updated_card.state.ease_factor, 2.5);
    // Grade 3: consecutive_correct increments
    assert_eq!(updated_card.state.consecutive_correct, 1);
    assert_eq!(updated_card.state.last_reviewed_on, Some(review_date));
    // Due date should be review_date + 4 days
    assert_eq!(
        updated_card.state.due_on,
        review_date + chrono::Duration::days(4)
    );
}

#[test]
fn grade_0_clamps_ease_factor_to_minimum() {
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
    let initial_interval = NonZeroU8::new(5).unwrap();
    // Start with ease_factor near minimum
    let initial_ease = 1.4;
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 0,
        })
        .unwrap();

    // Grade 0: ease_factor decreases by -0.3, but should be clamped to 1.3 minimum
    // 1.4 - 0.3 = 1.1, but clamped to 1.3
    assert_eq!(updated_card.state.ease_factor, 1.3);
}

#[test]
fn grade_4_clamps_ease_factor_to_maximum() {
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
    let initial_interval = NonZeroU8::new(5).unwrap();
    // Start with ease_factor near maximum
    let initial_ease = 2.7;
    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    let updated_card = store
        .record_review(ReviewRequest {
            card_id: card.id,
            reviewed_on: review_date,
            grade: 4,
        })
        .unwrap();

    // Grade 4: ease_factor increases by 0.15, but should be clamped to 2.8 maximum
    // 2.7 + 0.15 = 2.85, but clamped to 2.8
    assert_eq!(updated_card.state.ease_factor, 2.8);
}
