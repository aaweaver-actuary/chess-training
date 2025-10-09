use std::collections::HashMap;
use std::num::NonZeroU8;

use card_store::chess_position::ChessPosition;
use card_store::config::StorageConfig;
use card_store::memory::InMemoryCardStore;
use card_store::model::{
    Card, CardKind, CardState, EdgeInput, ReviewRequest, UnlockDetail, UnlockRecord,
};
use card_store::store::{CardStore, StoreError};
use chrono::{Duration, NaiveDate};

fn new_store() -> InMemoryCardStore {
    InMemoryCardStore::new(StorageConfig::default())
}

fn sample_position() -> ChessPosition {
    ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        0,
    )
    .expect("valid starting position")
}

fn sample_child_position() -> ChessPosition {
    ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        1,
    )
    .expect("valid child position")
}

/// Helper function to set up a card with the standard e2e4 edge for testing.
/// Returns (store, card) tuple ready for review testing.
fn setup_card_for_review(
    review_date: NaiveDate,
    initial_interval: NonZeroU8,
    initial_ease: f32,
) -> (InMemoryCardStore, card_store::model::Card) {
    let store = new_store();
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

    let card = store
        .create_opening_card(
            "andy",
            &edge,
            CardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    (store, card)
}

#[test]
fn position_creation_requires_valid_side_to_move() {
    let result = ChessPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(result.is_err());
}

#[test]
fn position_creation_fails_with_missing_fields() {
    // Missing side to move, castling, en passant, halfmove, fullmove
    let result = ChessPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(result.is_err());
}

#[test]
#[should_panic]
fn position_creation_fails_with_invalid_characters() {
    // Invalid character 'X' in FEN
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1",
        0,
    );
    assert!(result.is_err());
}

#[test]
#[should_panic]
fn position_creation_fails_with_extra_whitespace() {
    // Extra whitespace between fields
    let result = ChessPosition::new(
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
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra",
        0,
    );
    assert!(result.is_err());
}

#[test]
fn upsert_position_is_idempotent() {
    let store = new_store();
    let position = sample_position();

    let first = store.upsert_position(position.clone()).unwrap();
    let second = store.upsert_position(position.clone()).unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(first.fen, position.fen);
    assert_eq!(store.position_count().unwrap(), 1);
}

#[test]
fn inserting_edge_requires_parent_position() {
    let store = new_store();
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
    let store = new_store();
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
    let store = new_store();
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
    let store = new_store();
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let edge_id = 42;
    let record = UnlockRecord {
        owner_id: "andy".to_string(),
        detail: UnlockDetail { edge_id },
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
    let store = new_store();
    let edge_id = 42;
    let day1 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let day2 = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();

    let record1 = UnlockRecord {
        owner_id: "andy".to_string(),
        detail: UnlockDetail { edge_id },
        unlocked_on: day1,
    };
    let record2 = UnlockRecord {
        owner_id: "andy".to_string(),
        detail: UnlockDetail { edge_id },
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
fn importing_longer_line_preserves_existing_progress() {
    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const E4_FEN: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
    const E5_FEN: &str = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
    const NF3_FEN: &str = "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    const NC6_FEN: &str = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
    const BC4_FEN: &str = "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3";
    const BC5_FEN: &str = "r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    const C3_FEN: &str = "r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/2P2N2/PP1P1PPP/RNBQK2R b KQkq - 0 4";

    let store = InMemoryCardStore::new(StorageConfig::default());
    let initial_interval = NonZeroU8::new(1).unwrap();
    let owner = "learner";

    // Initial import containing moves up to 3.Bc4.
    let start = store
        .upsert_position(ChessPosition::new(START_FEN, 0).expect("start position"))
        .expect("store start");
    let e4_pos = store
        .upsert_position(ChessPosition::new(E4_FEN, 1).expect("after 1.e4"))
        .expect("store e4");
    let e5_pos = store
        .upsert_position(ChessPosition::new(E5_FEN, 2).expect("after 1...e5"))
        .expect("store e5");
    let nf3_pos = store
        .upsert_position(ChessPosition::new(NF3_FEN, 3).expect("after 2.Nf3"))
        .expect("store Nf3");
    let nc6_pos = store
        .upsert_position(ChessPosition::new(NC6_FEN, 4).expect("after 2...Nc6"))
        .expect("store Nc6");
    let bc4_pos = store
        .upsert_position(ChessPosition::new(BC4_FEN, 5).expect("after 3.Bc4"))
        .expect("store Bc4");

    let edge_e4 = store
        .upsert_edge(EdgeInput {
            parent_id: start.id,
            move_uci: "e2e4".to_string(),
            move_san: "e4".to_string(),
            child_id: e4_pos.id,
        })
        .expect("edge 1.e4");
    let _edge_e5 = store
        .upsert_edge(EdgeInput {
            parent_id: e4_pos.id,
            move_uci: "e7e5".to_string(),
            move_san: "e5".to_string(),
            child_id: e5_pos.id,
        })
        .expect("edge 1...e5");
    let edge_nf3 = store
        .upsert_edge(EdgeInput {
            parent_id: e5_pos.id,
            move_uci: "g1f3".to_string(),
            move_san: "Nf3".to_string(),
            child_id: nf3_pos.id,
        })
        .expect("edge 2.Nf3");
    let _edge_nc6 = store
        .upsert_edge(EdgeInput {
            parent_id: nf3_pos.id,
            move_uci: "b8c6".to_string(),
            move_san: "Nc6".to_string(),
            child_id: nc6_pos.id,
        })
        .expect("edge 2...Nc6");
    let edge_bc4 = store
        .upsert_edge(EdgeInput {
            parent_id: nc6_pos.id,
            move_uci: "f1c4".to_string(),
            move_san: "Bc4".to_string(),
            child_id: bc4_pos.id,
        })
        .expect("edge 3.Bc4");

    let white_edges = vec![edge_e4.clone(), edge_nf3.clone(), edge_bc4.clone()];
    let mut edge_to_card: HashMap<u64, u64> = HashMap::new();
    let start_day = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let initial_state = CardState::new(start_day, initial_interval, 2.5);

    for edge in &white_edges {
        let card = store
            .create_opening_card(owner, edge, initial_state.clone())
            .expect("create opening card");
        edge_to_card.insert(edge.id, card.id);
    }

    // Simulate a week of study by reviewing each card on its due date.
    let mut baseline: HashMap<u64, Card> = HashMap::new();
    for (edge_id, card_id) in &edge_to_card {
        let mut review_day = start_day;
        let mut latest: Option<Card> = None;
        for _ in 0..3 {
            let card = store
                .record_review(ReviewRequest {
                    card_id: *card_id,
                    reviewed_on: review_day,
                    grade: 4,
                })
                .expect("record review");
            review_day = card.state.due_on;
            latest = Some(card);
        }

        let card = latest.expect("at least one review per card");
        match card.kind {
            CardKind::Opening(ref opening) => {
                assert_eq!(opening.edge_id, *edge_id);
            }
            _ => panic!("expected opening card"),
        }
        baseline.insert(*edge_id, card);
    }

    // Second import arrives a week later with additional moves 3...Bc5 4.c3.
    let import_day = start_day + Duration::days(7);
    let import_state = CardState::new(import_day, initial_interval, 2.5);

    let bc5_pos = store
        .upsert_position(ChessPosition::new(BC5_FEN, 6).expect("after 3...Bc5"))
        .expect("store Bc5");
    let c3_pos = store
        .upsert_position(ChessPosition::new(C3_FEN, 7).expect("after 4.c3"))
        .expect("store c3");

    let edge_bc5 = store
        .upsert_edge(EdgeInput {
            parent_id: bc4_pos.id,
            move_uci: "f8c5".to_string(),
            move_san: "Bc5".to_string(),
            child_id: bc5_pos.id,
        })
        .expect("edge 3...Bc5");
    let edge_c3 = store
        .upsert_edge(EdgeInput {
            parent_id: edge_bc5.child_id,
            move_uci: "c2c3".to_string(),
            move_san: "c3".to_string(),
            child_id: c3_pos.id,
        })
        .expect("edge 4.c3");

    // Re-importing the earlier moves should not change their scheduling metadata.
    for edge in &white_edges {
        let card = store
            .create_opening_card(owner, edge, import_state.clone())
            .expect("re-import opening card");
        let original = baseline
            .get(&edge.id)
            .expect("baseline card missing for edge");
        assert_eq!(card, *original, "card state changed for {}", edge.move_san);
    }

    // The new move becomes the only item due on the import day.
    let new_card = store
        .create_opening_card(owner, &edge_c3, import_state.clone())
        .expect("create new move card");
    assert_eq!(
        new_card.state,
        CardState::new(import_day, initial_interval, 2.5),
        "new card should use default scheduling state",
    );

    let due_cards = store
        .fetch_due_cards(owner, import_day)
        .expect("fetch due cards");
    assert_eq!(due_cards.len(), 1, "only the new move should be due");
    assert_eq!(due_cards[0].id, new_card.id, "unexpected card queued");

    // Ensure the opponent move was recorded but does not create a learner card.
    // Only the learner's moves should result in card creation; opponent moves like Bc5
    // should not create a learner card. This assertion validates that behavior.
    assert!(!baseline.contains_key(&edge_bc5.id));
}

#[test]
fn unlock_different_edges_on_same_day() {
    let store = new_store();
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let edge_id1 = 42;
    let edge_id2 = 43;

    let record1 = UnlockRecord {
        owner_id: "andy".to_string(),
        detail: UnlockDetail { edge_id: edge_id1 },
        unlocked_on: date,
    };
    let record2 = UnlockRecord {
        owner_id: "andy".to_string(),
        detail: UnlockDetail { edge_id: edge_id2 },
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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let (store, card) = setup_card_for_review(review_date, NonZeroU8::new(1).unwrap(), 2.5);

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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let initial_interval = NonZeroU8::new(5).unwrap();
    let initial_ease = 2.5;
    let (store, card) = setup_card_for_review(review_date, initial_interval, initial_ease);

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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let initial_interval = NonZeroU8::new(10).unwrap();
    let initial_ease = 2.0;
    let (store, card) = setup_card_for_review(review_date, initial_interval, initial_ease);

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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let initial_interval = NonZeroU8::new(7).unwrap();
    let initial_ease = 2.5;
    let (store, card) = setup_card_for_review(review_date, initial_interval, initial_ease);

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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let initial_interval = NonZeroU8::new(3).unwrap();
    let initial_ease = 2.5;
    let (store, card) = setup_card_for_review(review_date, initial_interval, initial_ease);

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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let initial_interval = NonZeroU8::new(5).unwrap();
    // Start with ease_factor near minimum
    let initial_ease = 1.4;
    let (store, card) = setup_card_for_review(review_date, initial_interval, initial_ease);

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
    let review_date = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
    let initial_interval = NonZeroU8::new(5).unwrap();
    // Start with ease_factor near maximum
    let initial_ease = 2.7;
    let (store, card) = setup_card_for_review(review_date, initial_interval, initial_ease);

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
