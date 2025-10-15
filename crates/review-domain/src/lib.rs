//! Core domain types shared across the chess training back-end services.

pub mod card;
pub mod grade;
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
pub mod utils;

use chrono::NaiveDate;

/// Generic flashcard definition used across services.
pub use card::{Card, CardKind, StoredCardState};
/// Validated review grades and related errors.
pub use grade::{Grade, GradeError};
/// Strongly typed identifier wrappers used across the crate.
pub use ids::{CardId, EdgeId, IdConversionError, IdKind, LearnerId, MoveId, TacticId};
/// Opening-focused request and payload types.
pub use opening::{EdgeInput, OpeningCard, OpeningEdge, OpeningEdgeHandle};
/// Normalized chess position representation and related errors.
pub use position::{Position, PositionError, PositionId};
/// Opening repertoire store, graph representation, and associated move model.
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

pub use utils::hash_with_seed;

pub const TEST_EPSILON: f32 = 1e-6;

/// Helper function to create `NaiveDate` instances in tests.
///
/// # Panics
/// Panics if the provided year, month, and day do not form a valid date.
/// # Examples
/// ```rust
/// use chrono::NaiveDate;
/// use review_domain::naive_date;
/// let date = naive_date(2024, 5, 15);
/// assert_eq!(date, NaiveDate::from_ymd_opt(2024, 5, 15).unwrap());
/// ```
#[must_use]
pub fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}
