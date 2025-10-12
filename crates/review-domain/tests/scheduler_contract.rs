use review_domain::ReviewGrade;
use review_domain::scheduler_contract::{
    CardSummary, CardSummaryKind, CardSummaryMetaValue, GradeRequest, QueueRequest,
};

#[cfg(feature = "serde")]
use review_domain::scheduler_contract::{GradeResponse, QueueResponse, SessionStats};

#[test]
fn queue_request_exposes_user_id() {
    let request = QueueRequest::new("user-123");
    assert_eq!(request.user_id, "user-123");
}

#[test]
fn card_summary_helpers_populate_optional_fields() {
    let summary = CardSummary::new(
        "card-1",
        CardSummaryKind::Opening,
        "startpos",
        "Play the main line",
    )
    .with_expected_moves(["e2e4", "e7e5"])
    .with_pv(["e2e4", "e7e5", "g1f3"])
    .with_meta("teaching_note", "Focus on central control")
    .with_meta("difficulty", 0.5);

    assert_eq!(
        summary.expected_moves_uci,
        Some(vec!["e2e4".to_string(), "e7e5".to_string()])
    );
    assert_eq!(
        summary.pv_uci,
        Some(vec![
            "e2e4".to_string(),
            "e7e5".to_string(),
            "g1f3".to_string()
        ])
    );
    let meta = summary.meta.expect("meta should be present");
    assert_eq!(
        meta.get("teaching_note"),
        Some(&CardSummaryMetaValue::Text(
            "Focus on central control".to_string()
        ))
    );
    assert_eq!(
        meta.get("difficulty"),
        Some(&CardSummaryMetaValue::Number(0.5))
    );
}

#[test]
fn grade_request_captures_latency_and_grade() {
    let request = GradeRequest::new("session-1", "card-42", ReviewGrade::Good, 1_200);
    assert_eq!(request.session_id, "session-1");
    assert_eq!(request.card_id, "card-42");
    assert_eq!(request.grade, ReviewGrade::Good);
    assert_eq!(request.latency_ms, 1_200);
}

#[cfg(feature = "serde")]
use serde_json::json;

#[cfg(feature = "serde")]
#[test]
fn queue_response_serializes_expected_shape() {
    let summary = CardSummary::new(
        "card-99",
        CardSummaryKind::Tactic,
        "8/8/8/8/8/8/8/8 w - - 0 1",
        "Find the winning tactic",
    )
    .with_meta("theme", "Fork")
    .with_expected_moves(["e5f7"]);
    let response = QueueResponse::new(vec![summary]);

    let json = serde_json::to_value(response).expect("serialization should succeed");
    let expected = json!({
        "queue": [
            {
                "card_id": "card-99",
                "kind": "Tactic",
                "position_fen": "8/8/8/8/8/8/8/8 w - - 0 1",
                "prompt": "Find the winning tactic",
                "expected_moves_uci": ["e5f7"],
                "meta": { "theme": "Fork" }
            }
        ]
    });

    assert_eq!(json, expected);
}

#[cfg(feature = "serde")]
#[test]
fn grade_response_serializes_stats_and_next_card() {
    let stats = SessionStats::new(3, 0.75, 1_100)
        .with_due_count(25)
        .with_completed_count(12);
    let next_card = CardSummary::new(
        "card-100",
        CardSummaryKind::Opening,
        "startpos",
        "Remember the theory",
    )
    .with_meta("line", "Ruy Lopez");
    let response = GradeResponse::new(Some(next_card), stats);

    let json = serde_json::to_value(response).expect("serialization should succeed");
    let expected = json!({
        "next_card": {
            "card_id": "card-100",
            "kind": "Opening",
            "position_fen": "startpos",
            "prompt": "Remember the theory",
            "meta": { "line": "Ruy Lopez" }
        },
        "stats": {
            "reviews_today": 3,
            "accuracy": 0.75,
            "avg_latency_ms": 1_100,
            "due_count": 25,
            "completed_count": 12
        }
    });

    assert_eq!(json, expected);
}
