//! Core domain types shared across the chess training back-end services.

pub mod card;
pub mod card_aggregate;
pub mod card_kind;
pub mod card_state;
pub mod grade;
pub mod hash;
pub mod ids;
pub mod macros;
pub mod opening;
pub mod position;
pub mod repertoire;
pub mod review;
pub mod review_grade;
pub mod study_stage;
pub mod tactic;
pub mod unlock;

/// Generic flashcard definition used across services.
pub use card::Card;
/// Aggregate wrapper around the default card representation.
pub use card_aggregate::CardAggregate;
/// High-level classification of review cards.
pub use card_kind::CardKind;
/// Scheduling metadata tracked for each stored card.
pub use card_state::StoredCardState;
/// Validated review grades and related errors.
pub use grade::{GradeError, ValidGrade};
/// Deterministic hashing helper backed by BLAKE3.
pub use hash::hash64;
/// Strongly typed identifier wrappers used across the crate.
pub use ids::{CardId, EdgeId, IdConversionError, MoveId, PositionId};
/// Opening-focused request and payload types.
pub use opening::{EdgeInput, OpeningCard, OpeningEdge};
/// Normalized chess position representation and related errors.
pub use position::{ChessPosition, PositionError};
/// Opening repertoire store, adjacency graph, and associated move representation.
pub use repertoire::{OpeningGraph, Repertoire, RepertoireError, RepertoireMove};
/// Review submission payload capturing user input.
pub use review::ReviewRequest;
/// Grading scale for spaced repetition reviews.
pub use review_grade::ReviewGrade;
/// Learning stage classification for cards.
pub use study_stage::StudyStage;
/// Tactic-focused card payloads.
pub use tactic::TacticCard;
/// Unlock record details for progressive content releases.
pub use unlock::{UnlockDetail, UnlockRecord};
