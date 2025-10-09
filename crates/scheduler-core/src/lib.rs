//! scheduler-core — SM-2 scheduling, unlock policy, and supporting types.

pub mod config;
pub mod domain;
pub mod errors;
pub mod grade;
pub mod queue;
pub mod scheduler;
pub mod sm2;
pub mod store;

pub use config::SchedulerConfig;
pub use domain::{Card, CardKind, CardState, ReviewOutcome, UnlockRecord};
pub use errors::SchedulerError;
pub use grade::ReviewGrade;
pub use queue::build_queue_for_day;
pub use scheduler::Scheduler;
pub use store::{CardStore, InMemoryStore};
