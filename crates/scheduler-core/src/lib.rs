//! scheduler-core — SM-2 scheduling, unlock policy, and supporting types.

/// Scheduler configuration options governing SM-2 behavior.
pub mod config;
/// Domain-specific data structures exposed by the scheduler.
pub mod domain;
/// Error type returned by scheduler operations.
pub mod errors;
/// Review queue construction helpers.
pub mod queue;
/// Review planning helpers exposed to front-end consumers.
pub mod review_planner;
/// High-level scheduler façade orchestrating reviews.
pub mod scheduler;
/// SM-2 calculation utilities.
pub mod sm2;
/// Storage abstractions consumed by the scheduler.
pub mod store;

/// Configuration values used to tune the scheduler.
pub use config::SchedulerConfig;
/// Domain exports for cards, unlocks, and helper constructors.
pub use domain::{
    Card, CardKind, CardState, ReviewOutcome, SchedulerOpeningCard, SchedulerTacticCard,
    SchedulerUnlockDetail, UnlockRecord, new_card,
};
/// Error returned when scheduling operations fail.
pub use errors::SchedulerError;
/// Build the review queue for a given study day.
pub use queue::build_queue_for_day;
/// Review grade shared with review-domain consumers.
pub use review_domain::ReviewGrade;
/// Review planner exports.
pub use review_planner::{
    AccuracyRisk, BacklogPressure, Recommendation, ReviewOverview, ReviewPlanner,
    ReviewPlannerError, ReviewSnapshot, UpcomingUnlock,
};
/// Scheduler façade orchestrating queue building and review processing.
pub use scheduler::Scheduler;
/// Storage trait and in-memory implementation used by the scheduler.
pub use store::{CardStore, InMemoryStore};
