//! Storage trait and error types shared across card-store backends.

use std::fmt;

use chrono::NaiveDate;
use thiserror::Error;

use crate::chess_position::ChessPosition;
use crate::errors::PositionError;
use crate::model::{Card, Edge, EdgeId, EdgeInput, ReviewRequest, StoredCardState, UnlockRecord};

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
    DuplicateUnlock { edge: EdgeId, day: NaiveDate },
    /// Underlying in-memory synchronization primitive was poisoned.
    #[error("lock on {resource} store data has been poisoned")]
    PoisonedLock { resource: &'static str },
    /// Collision detected when generating deterministic identifiers.
    #[error("hash collision detected for {entity}")]
    HashCollision { entity: &'static str },
    /// Invalid position provided during an upsert operation.
    #[error(transparent)]
    InvalidPosition(#[from] PositionError),
    /// Scheduler state could not be persisted because the interval was invalid.
    #[error("scheduler state cannot be persisted: {reason}")]
    InvalidSchedulerState { reason: String },
}

/// Persistence abstraction used across services.
pub trait CardStore: Send + Sync + fmt::Debug {
    /// Insert or update a [`Position`]. Returns the stored record.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the underlying persistence layer fails to
    /// store the position or when the provided position is invalid.
    fn upsert_position(&self, position: ChessPosition) -> Result<ChessPosition, StoreError>;
    /// Insert or update an [`Edge`]. Returns the stored record.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the persistence layer cannot upsert the edge.
    fn upsert_edge(&self, edge: EdgeInput) -> Result<Edge, StoreError>;
    /// Create or fetch an opening card for the given owner and edge.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the store cannot create or fetch the card.
    fn create_opening_card(
        &self,
        owner_id: &str,
        edge: &Edge,
        state: StoredCardState,
    ) -> Result<Card, StoreError>;
    /// Fetch all due cards for an owner on or before `as_of`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the store cannot query the due cards.
    fn fetch_due_cards(&self, owner_id: &str, as_of: NaiveDate) -> Result<Vec<Card>, StoreError>;
    /// Record a review and return the updated card state.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the review cannot be recorded or the grade is
    /// invalid.
    fn record_review(&self, review: ReviewRequest) -> Result<Card, StoreError>;
    /// Record a newly unlocked opening edge.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the unlock cannot be recorded or conflicts
    /// with an existing record.
    fn record_unlock(&self, unlock: UnlockRecord) -> Result<(), StoreError>;
}
