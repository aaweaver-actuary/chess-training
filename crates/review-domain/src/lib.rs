//! Core domain types shared across the chess training back-end services.

mod card;
mod card_kind;
mod card_state;
mod hash;
mod macros;
mod opening;
mod position;
mod review;
mod review_grade;
pub mod study_stage;
mod tactic;
mod unlock;
pub mod valid_grade;

/// Generic flashcard definition used across services.
pub use card::Card;
/// High-level classification of review cards.
pub use card_kind::CardKind;
/// Scheduling metadata tracked for each stored card.
pub use card_state::StoredCardState;
/// Deterministic hashing helper backed by BLAKE3.
pub use hash::hash64;
/// Opening-focused request and payload types.
pub use opening::{EdgeInput, OpeningCard, OpeningEdge};
/// Normalized chess position representation and related errors.
pub use position::{ChessPosition, PositionError};
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
/// Validated review grades and related errors.
pub use valid_grade::{GradeError, ValidGrade};
