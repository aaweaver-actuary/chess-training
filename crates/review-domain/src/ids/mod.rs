//! Type-safe identifier wrappers shared across review domain modules.
pub mod id_conversion_error;
pub mod id_kind;

pub mod card_id;
pub mod edge_id;
pub mod learner_id;
pub mod move_id;
pub mod tactic_id;
pub mod unlock_id;

pub use id_conversion_error::IdConversionError;
pub use id_kind::IdKind;

pub use card_id::CardId;
pub use edge_id::EdgeId;
pub use learner_id::LearnerId;
pub use move_id::MoveId;
pub use tactic_id::TacticId;
pub use unlock_id::UnlockId;
