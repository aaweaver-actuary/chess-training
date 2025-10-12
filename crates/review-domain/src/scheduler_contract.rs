//! Scheduler HTTP API contracts used by back-end services and gateways.

use std::collections::BTreeMap;

use crate::review_grade::ReviewGrade;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Request payload sent to the scheduler `/queue` endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QueueRequest {
    /// Identifier of the learner whose queue is being fetched.
    pub user_id: String,
}

impl QueueRequest {
    /// Construct a new queue request for the provided learner identifier.
    #[must_use]
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
        }
    }
}

/// Response body produced by the scheduler `/queue` endpoint.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QueueResponse {
    /// Ordered collection of cards that should be reviewed next.
    pub queue: Vec<CardSummary>,
}

impl QueueResponse {
    /// Create a queue response from the provided card summaries.
    #[must_use]
    pub fn new(queue: Vec<CardSummary>) -> Self {
        Self { queue }
    }
}

/// Minimal card description shared across the scheduler contract.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CardSummary {
    /// Identifier used to refer to the card in follow-up requests.
    pub card_id: String,
    /// High-level card classification.
    pub kind: CardSummaryKind,
    /// Chess position presented to the learner.
    pub position_fen: String,
    /// Prompt displayed to the learner when the card becomes active.
    pub prompt: String,
    /// Expected best-move sequence in UCI notation.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub expected_moves_uci: Option<Vec<String>>,
    /// Principal variation in UCI notation used for teaching overlays.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub pv_uci: Option<Vec<String>>,
    /// Optional metadata consumed by downstream clients.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub meta: Option<BTreeMap<String, CardSummaryMetaValue>>,
}

impl CardSummary {
    /// Create a new card summary with the required fields populated.
    #[must_use]
    pub fn new(
        card_id: impl Into<String>,
        kind: CardSummaryKind,
        position_fen: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            card_id: card_id.into(),
            kind,
            position_fen: position_fen.into(),
            prompt: prompt.into(),
            expected_moves_uci: None,
            pv_uci: None,
            meta: None,
        }
    }

    /// Attach the expected move list in UCI notation.
    #[must_use]
    pub fn with_expected_moves<I, S>(mut self, moves: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.expected_moves_uci = Some(moves.into_iter().map(Into::into).collect());
        self
    }

    /// Attach the principal variation in UCI notation used for teaching overlays.
    #[must_use]
    pub fn with_pv<I, S>(mut self, moves: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.pv_uci = Some(moves.into_iter().map(Into::into).collect());
        self
    }

    /// Attach metadata describing supplemental teaching content.
    #[must_use]
    pub fn with_meta(
        mut self,
        key: impl Into<String>,
        value: impl Into<CardSummaryMetaValue>,
    ) -> Self {
        let entry = value.into();
        if let Some(meta) = &mut self.meta {
            meta.insert(key.into(), entry);
        } else {
            let mut meta = BTreeMap::new();
            meta.insert(key.into(), entry);
            self.meta = Some(meta);
        }
        self
    }
}

/// Classification for card summaries used in the scheduler contract.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub enum CardSummaryKind {
    /// Card originating from an opening repertoire.
    Opening,
    /// Card representing a tactic pattern.
    Tactic,
}

/// Values permitted inside the metadata bag of a card summary.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum CardSummaryMetaValue {
    /// Textual metadata value.
    Text(String),
    /// Numeric metadata value.
    Number(f64),
}

impl From<String> for CardSummaryMetaValue {
    fn from(value: String) -> Self {
        CardSummaryMetaValue::Text(value)
    }
}

impl From<&str> for CardSummaryMetaValue {
    fn from(value: &str) -> Self {
        CardSummaryMetaValue::Text(value.to_owned())
    }
}

impl From<f64> for CardSummaryMetaValue {
    fn from(value: f64) -> Self {
        CardSummaryMetaValue::Number(value)
    }
}

impl From<f32> for CardSummaryMetaValue {
    fn from(value: f32) -> Self {
        CardSummaryMetaValue::Number(f64::from(value))
    }
}

impl From<u32> for CardSummaryMetaValue {
    fn from(value: u32) -> Self {
        CardSummaryMetaValue::Number(f64::from(value))
    }
}

impl From<u64> for CardSummaryMetaValue {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: u64) -> Self {
        CardSummaryMetaValue::Number(value as f64)
    }
}

impl From<i32> for CardSummaryMetaValue {
    fn from(value: i32) -> Self {
        CardSummaryMetaValue::Number(f64::from(value))
    }
}

impl From<i64> for CardSummaryMetaValue {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: i64) -> Self {
        CardSummaryMetaValue::Number(value as f64)
    }
}

/// Grade submission payload accepted by the scheduler `/grade` endpoint.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GradeRequest {
    /// Identifier of the client-maintained study session.
    pub session_id: String,
    /// Identifier of the graded card.
    pub card_id: String,
    /// Grade awarded to the card.
    pub grade: ReviewGrade,
    /// Latency in milliseconds between presenting the card and receiving the grade.
    pub latency_ms: u32,
}

impl GradeRequest {
    /// Construct a new grade request for the provided session and card identifiers.
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        card_id: impl Into<String>,
        grade: ReviewGrade,
        latency_ms: u32,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            card_id: card_id.into(),
            grade,
            latency_ms,
        }
    }
}

/// Response body emitted by the scheduler `/grade` endpoint.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GradeResponse {
    /// Next card to study, if any remain in the queue.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub next_card: Option<CardSummary>,
    /// Updated session statistics for the learner.
    pub stats: SessionStats,
}

impl GradeResponse {
    /// Construct a new grade response with the provided next card and session statistics.
    #[must_use]
    pub fn new(next_card: Option<CardSummary>, stats: SessionStats) -> Self {
        Self { next_card, stats }
    }
}

/// Aggregated statistics returned to clients alongside queue and grade responses.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SessionStats {
    /// Total number of reviews completed today.
    pub reviews_today: u32,
    /// Rolling accuracy for the current session.
    pub accuracy: f64,
    /// Average response latency in milliseconds.
    pub avg_latency_ms: u32,
    /// Number of cards still due for review.
    #[cfg_attr(feature = "serde", serde(default))]
    pub due_count: u32,
    /// Number of cards completed in the session.
    #[cfg_attr(feature = "serde", serde(default))]
    pub completed_count: u32,
}

impl SessionStats {
    /// Create session statistics with zeroed due/completed counts.
    #[must_use]
    pub fn new(reviews_today: u32, accuracy: f64, avg_latency_ms: u32) -> Self {
        Self {
            reviews_today,
            accuracy,
            avg_latency_ms,
            due_count: 0,
            completed_count: 0,
        }
    }

    /// Update the number of cards still due for review.
    #[must_use]
    pub fn with_due_count(mut self, due_count: u32) -> Self {
        self.due_count = due_count;
        self
    }

    /// Update the number of cards completed in the session.
    #[must_use]
    pub fn with_completed_count(mut self, completed_count: u32) -> Self {
        self.completed_count = completed_count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_meta_initializes_map() {
        let summary = CardSummary::new("card", CardSummaryKind::Opening, "fen", "prompt")
            .with_meta("note", "Remember the plan");
        let meta = summary.meta.expect("meta should be populated");
        assert_eq!(
            meta.get("note"),
            Some(&CardSummaryMetaValue::Text("Remember the plan".into()))
        );
    }
}
