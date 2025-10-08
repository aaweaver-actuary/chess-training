//! Storage trait and error types shared across card-store backends.

use std::fmt;

use chrono::NaiveDate;
use thiserror::Error;

use crate::model::{
    Card, CardState, Edge, EdgeInput, Position, PositionError, ReviewRequest, UnlockRecord,
};

/// Unified error type returned by [`CardStore`] implementations.
#[derive(Debug, Error, PartialEq)]
pub enum StoreError {
    /// A required position was not found.
    #[error("missing position with id {id}")]
    MissingPosition { id: u64 },
    /// An edge lookup failed.
    #[error("missing edge with id {id}")]
    MissingEdge { id: u64 },
    /// Attempted to update a card that does not exist.
    #[error("missing card with id {id}")]
    MissingCard { id: u64 },
    /// The provided grade was outside the supported range.
    #[error("invalid grade {grade}; expected 0-4")]
    InvalidGrade { grade: u8 },
    /// Unlock record already exists for the day.
    #[error("duplicate unlock for edge {edge} on {day}")]
    DuplicateUnlock { edge: u64, day: NaiveDate },
    /// Underlying in-memory synchronization primitive was poisoned.
    #[error("lock on {resource} store data has been poisoned")]
    PoisonedLock { resource: &'static str },
    /// Collision detected when generating deterministic identifiers.
    #[error("hash collision detected for {entity}")]
    HashCollision { entity: &'static str },
    /// Invalid position provided during an upsert operation.
    #[error(transparent)]
    InvalidPosition(#[from] PositionError),
}

/// Persistence abstraction used across services.
pub trait CardStore: Send + Sync + fmt::Debug {
    /// Insert or update a [`Position`]. Returns the stored record.
    fn upsert_position(&self, position: Position) -> Result<Position, StoreError>;
    /// Insert or update an [`Edge`]. Returns the stored record.
    fn upsert_edge(&self, edge: EdgeInput) -> Result<Edge, StoreError>;
    /// Create or fetch an opening card for the given owner and edge.
    fn create_opening_card(
        &self,
        owner_id: &str,
        edge: &Edge,
        state: CardState,
    ) -> Result<Card, StoreError>;
    /// Fetch all due cards for an owner on or before `as_of`.
    fn fetch_due_cards(&self, owner_id: &str, as_of: NaiveDate) -> Result<Vec<Card>, StoreError>;
    /// Record a review and return the updated card state.
    fn record_review(&self, review: ReviewRequest) -> Result<Card, StoreError>;
    /// Record a newly unlocked opening edge.
    fn record_unlock(&self, unlock: UnlockRecord) -> Result<(), StoreError>;
}
