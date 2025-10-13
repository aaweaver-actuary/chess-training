use std::collections::HashMap;
use std::num::NonZeroU8;

use card_store::chess_position::ChessPosition;
use card_store::config::StorageConfig;
use card_store::memory::InMemoryCardStore;
use card_store::model::{
    Card, CardKind, Edge, EdgeInput, ReviewRequest, StoredCardState, UnlockDetail, UnlockRecord,
};
use card_store::store::{CardStore, StoreError};
use chrono::{Duration, NaiveDate};
use review_domain::EdgeId;

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
            StoredCardState::new(review_date, initial_interval, initial_ease),
        )
        .unwrap();

    (store, card)
}

fn store_position_with_label(
    store: &InMemoryCardStore,
    fen: &str,
    ply: u32,
    label: &str,
) -> ChessPosition {
    let position =
        ChessPosition::new(fen, ply).unwrap_or_else(|_| panic!("invalid position {label}"));
    store
        .upsert_position(position)
        .unwrap_or_else(|_| panic!("store {label} position"))
}

fn store_edge_with_label(
    store: &InMemoryCardStore,
    parent: u64,
    child: u64,
    uci: &str,
    san: &str,
    label: &str,
) -> Edge {
    store
        .upsert_edge(EdgeInput {
            parent_id: parent,
            move_uci: uci.to_string(),
            move_san: san.to_string(),
            child_id: child,
        })
        .unwrap_or_else(|_| panic!("store {label} edge"))
}

fn upsert_positions(
    store: &InMemoryCardStore,
    definitions: &[(&'static str, &'static str, u32)],
) -> HashMap<&'static str, ChessPosition> {
    let mut positions = HashMap::new();
    for (label, fen, ply) in definitions {
        let stored = store_position_with_label(store, fen, *ply, label);
        positions.insert(*label, stored);
    }
    positions
}

fn upsert_edges(
    store: &InMemoryCardStore,
    positions: &HashMap<&'static str, ChessPosition>,
    definitions: &[(
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    )],
) -> HashMap<&'static str, Edge> {
    let mut edges = HashMap::new();
    for (label, parent, child, uci, san) in definitions {
        let parent_id = positions[parent].id;
        let child_id = positions[child].id;
        let edge = store_edge_with_label(store, parent_id, child_id, uci, san, label);
        edges.insert(*label, edge);
    }
    edges
}

fn extend_positions(
    store: &InMemoryCardStore,
    positions: &mut HashMap<&'static str, ChessPosition>,
    definitions: &[(&'static str, &'static str, u32)],
) {
    for (label, fen, ply) in definitions {
        let stored = store_position_with_label(store, fen, *ply, label);
        positions.insert(*label, stored);
    }
}

fn build_baseline(
    store: &InMemoryCardStore,
    owner: &str,
    edges: &[Edge],
    start_day: NaiveDate,
    initial_interval: NonZeroU8,
    initial_ease: f32,
) -> HashMap<EdgeId, Card> {
    let mut baseline = HashMap::new();
    for edge in edges {
        let mut card = store
            .create_opening_card(
                owner,
                edge,
                StoredCardState::new(start_day, initial_interval, initial_ease),
            )
            .expect("create opening card");
        let mut review_day = start_day;
        for _ in 0..3 {
            card = store
                .record_review(ReviewRequest {
                    card_id: card.id,
                    reviewed_on: review_day,
                    grade: 4,
                })
                .expect("record review");
            review_day = card.state.due_on;
        }

        assert!(matches!(
            &card.kind,
            CardKind::Opening(opening) if opening.edge_id == edge.id
        ));
        baseline.insert(edge.id, card);
    }
    baseline
}

#[test]
fn position_creation_requires_valid_side_to_move() {
    let result = ChessPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(result.is_err());
}

#[test]
fn upsert_position_rejects_invalid_positions() {
    let store = new_store();
    let invalid = ChessPosition {
        id: 1,
        fen: "invalid".into(),
        side_to_move: 'x',
        ply: 0,
    };

    assert!(matches!(
        store.upsert_position(invalid),
        Err(StoreError::InvalidPosition(_))
    ));
}

#[test]
fn position_creation_fails_with_missing_fields() {
    // Missing side to move, castling, en passant, halfmove, fullmove
    let result = ChessPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert!(result.is_err());
}

#[test]
fn position_creation_fails_with_invalid_characters() {
    // Invalid character 'X' in FEN
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1",
        0,
    );
    assert!(result.is_err());
}

#[test]
fn position_creation_fails_with_extra_whitespace() {
    // Extra whitespace between fields
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR   w KQkq - 0 1",
        0,
    );
    assert!(result.is_err());
}

// This test ensures invalid FEN strings with too many fields are rejected.
#[test]
fn position_creation_fails_with_too_many_fields() {
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra",
        0,
    );
    assert!(result.is_err());
}

#[test]
fn position_creation_returns_invalid_piece_placement_for_invalid_characters() {
    use card_store::errors::PositionError;
    // Invalid character 'X' in piece placement field
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1",
        0,
    );
    assert_eq!(result, Err(PositionError::InvalidPiecePlacement));
}

#[test]
fn position_creation_returns_malformed_fen_for_missing_fields() {
    use card_store::errors::PositionError;
    let result = ChessPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0);
    assert_eq!(result, Err(PositionError::MalformedFen));
}

#[test]
fn position_creation_returns_invalid_side_to_move_for_invalid_side() {
    use card_store::errors::PositionError;
    let result = ChessPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
        0,
    );
    assert_eq!(result, Err(PositionError::InvalidSideToMove));
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
            StoredCardState::new(past, NonZeroU8::new(1).unwrap(), 2.5),
        )
        .unwrap();
    store
        .create_opening_card(
            "andy",
            &edge,
            StoredCardState::new(future, NonZeroU8::new(1).unwrap(), 2.5),
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
    let edge_id = EdgeId::new(42);
    let record = UnlockRecord {
        owner_id: "andy".to_string(),
        detail: UnlockDetail { edge_id },
        unlocked_on: date,
    };

    store.record_unlock(record.clone()).unwrap();
    let second = store.record_unlock(record);
    assert!(matches!(
        second,
        Err(StoreError::DuplicateUnlock { edge, day })
            if edge == edge_id.get() && day == date
    ));
}

#[test]
fn unlock_same_edge_on_different_days() {
    let store = new_store();
    let edge_id = EdgeId::new(42);
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

    let mut positions = upsert_positions(
        &store,
        &[
            ("start", START_FEN, 0),
            ("e4", E4_FEN, 1),
            ("e5", E5_FEN, 2),
            ("nf3", NF3_FEN, 3),
            ("nc6", NC6_FEN, 4),
            ("bc4", BC4_FEN, 5),
        ],
    );
    let initial_edges = upsert_edges(
        &store,
        &positions,
        &[
            ("e4", "start", "e4", "e2e4", "e4"),
            ("e5", "e4", "e5", "e7e5", "e5"),
            ("nf3", "e5", "nf3", "g1f3", "Nf3"),
            ("nc6", "nf3", "nc6", "b8c6", "Nc6"),
            ("bc4", "nc6", "bc4", "f1c4", "Bc4"),
        ],
    );
    let white_edges = vec![
        initial_edges["e4"].clone(),
        initial_edges["nf3"].clone(),
        initial_edges["bc4"].clone(),
    ];
    let start_day = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let baseline = build_baseline(
        &store,
        owner,
        &white_edges,
        start_day,
        initial_interval,
        2.5,
    );

    let import_day = start_day + Duration::days(7);
    let import_state = StoredCardState::new(import_day, initial_interval, 2.5);

    extend_positions(
        &store,
        &mut positions,
        &[("bc5", BC5_FEN, 6), ("c3", C3_FEN, 7)],
    );
    let new_edges = upsert_edges(
        &store,
        &positions,
        &[
            ("bc5", "bc4", "bc5", "f8c5", "Bc5"),
            ("c3", "bc5", "c3", "c2c3", "c3"),
        ],
    );
    let bc5_edge = new_edges["bc5"].clone();
    let c3_edge = new_edges["c3"].clone();

    for edge in &white_edges {
        let card = store
            .create_opening_card(owner, edge, import_state.clone())
            .expect("re-import opening card");
        let original = baseline
            .get(&edge.id)
            .expect("baseline card missing for edge");
        assert_eq!(card, *original, "card state changed for {}", edge.move_san);
    }

    let new_card = store
        .create_opening_card(owner, &c3_edge, import_state.clone())
        .expect("create new move card");
    assert_eq!(
        new_card.state,
        StoredCardState::new(import_day, initial_interval, 2.5),
        "new card should use default scheduling state",
    );

    let due_cards = store
        .fetch_due_cards(owner, import_day)
        .expect("fetch due cards");
    assert_eq!(due_cards.len(), 1, "only the new move should be due");
    assert_eq!(due_cards[0].id, new_card.id, "unexpected card queued");

    assert!(!baseline.contains_key(&bc5_edge.id));
}

#[test]
fn unlock_different_edges_on_same_day() {
    let store = new_store();
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let edge_id1 = EdgeId::new(42);
    let edge_id2 = EdgeId::new(43);

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
    assert!((updated_card.state.ease_factor - 2.2).abs() < f32::EPSILON); // 2.5 - 0.3 = 2.2
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
    assert!((updated_card.state.ease_factor - 1.85).abs() < f32::EPSILON); // 2.0 - 0.15 = 1.85
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
    assert!((updated_card.state.ease_factor - 2.45).abs() < f32::EPSILON); // 2.5 - 0.05 = 2.45
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
    assert!((updated_card.state.ease_factor - 2.5).abs() < f32::EPSILON);
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
    assert!((updated_card.state.ease_factor - 1.3).abs() < f32::EPSILON);
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
    assert!((updated_card.state.ease_factor - 2.8).abs() < f32::EPSILON);
}
