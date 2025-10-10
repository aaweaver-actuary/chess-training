//! Shared data structures for representing chess study artifacts.

mod card;
mod card_kind;
mod card_state;
mod hash;
mod opening;
mod position;
mod review;
mod study_stage;
mod tactic;
mod unlock;

pub use card::Card;
pub use card_kind::CardKind;
pub use card_state::StoredCardState;
pub use hash::hash64;
pub use opening::{EdgeInput, OpeningCard, OpeningEdge};
pub use position::{ChessPosition, PositionError};
pub use review::ReviewRequest;
pub use study_stage::StudyStage;
pub use tactic::TacticCard;
pub use unlock::{UnlockDetail, UnlockRecord};
