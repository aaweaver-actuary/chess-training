//! scheduler-core — SM-2 scheduling, unlock policy, and supporting types.

pub mod config;
pub mod domain;
pub mod errors;
pub mod queue;
pub mod scheduler;
pub mod sm2;
pub mod store;

pub use config::SchedulerConfig;
pub use domain::{
    Card, CardKind, CardState, ReviewOutcome, SchedulerOpeningCard, SchedulerTacticCard,
    SchedulerUnlockDetail, UnlockRecord, new_card,
};
pub use errors::SchedulerError;
pub use queue::build_queue_for_day;
pub use review_domain::ReviewGrade;
pub use scheduler::Scheduler;
pub use store::{CardStore, InMemoryStore};
