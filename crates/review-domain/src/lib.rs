//! Shared data structures for representing chess study artifacts.

mod card;
mod card_kind;
mod opening;
mod study_stage;
mod tactic;
mod unlock;

pub use card::Card;
pub use card_kind::CardKind;
pub use opening::{OpeningCard, OpeningEdge};
pub use study_stage::StudyStage;
pub use tactic::TacticCard;
pub use unlock::{UnlockDetail, UnlockRecord};
