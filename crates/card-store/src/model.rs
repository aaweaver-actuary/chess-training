//! Domain model structs shared by card-store implementations.

use std::num::NonZeroU8;

use chrono::NaiveDate;

pub use review_domain::{OpeningCard, TacticCard};

use review_domain::{
    Card as GenericCard, CardKind as GenericCardKind, OpeningEdge,
    UnlockDetail as GenericUnlockDetail, UnlockRecord as GenericUnlockRecord,
};

use crate::hash64;

/// Input payload for inserting or updating an edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeInput {
    /// Parent position identifier.
    pub parent_id: u64,
    /// Move in UCI format.
    pub move_uci: String,
    /// Move in SAN format.
    pub move_san: String,
    /// Child position identifier.
    pub child_id: u64,
}

impl EdgeInput {
    /// Converts the input payload into a canonical [`OpeningEdge`].
    ///
    /// The canonical form computes a deterministic edge ID from the parent position and move,
    /// and returns an [`OpeningEdge`] with normalized fields.
    #[must_use]
    pub fn into_edge(self) -> Edge {
        let id = hash64(&[&self.parent_id.to_be_bytes(), self.move_uci.as_bytes()]);
        Edge {
            id,
            parent_id: self.parent_id,
            child_id: self.child_id,
            move_uci: self.move_uci,
            move_san: self.move_san,
        }
    }
}

/// Opening edge describing a transition between two positions.
pub type Edge = OpeningEdge;

/// Classification of a card target.
pub type CardKind = GenericCardKind<OpeningCard, TacticCard>;

/// Mutable scheduling state of a card stored in the card-store service.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredCardState {
    /// Date on which the card becomes due.
    pub due_on: NaiveDate,
    /// Current interval in days.
    pub interval: NonZeroU8,
    /// Ease factor controlling growth of the interval.
    pub ease_factor: f32,
    /// Consecutive correct reviews streak.
    pub consecutive_correct: u32,
    /// Date of the last successful review.
    pub last_reviewed_on: Option<NaiveDate>,
}

impl StoredCardState {
    /// Creates a new [`StoredCardState`] with sensible defaults.
    #[must_use]
    pub fn new(due_on: NaiveDate, interval: NonZeroU8, ease_factor: f32) -> Self {
        Self {
            due_on,
            interval,
            ease_factor,
            consecutive_correct: 0,
            last_reviewed_on: None,
        }
    }
}

/// Flashcard representing either an opening move or a tactic.
pub type Card = GenericCard<u64, String, CardKind, StoredCardState>;

/// Request payload for recording a review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRequest {
    /// Target card identifier.
    pub card_id: u64,
    /// Date of the review.
    pub reviewed_on: NaiveDate,
    /// Grade (0-4) awarded by the learner.
    pub grade: u8,
}

/// Domain payload stored for each unlock record.
pub type UnlockDetail = GenericUnlockDetail;

/// Unlock ledger entry representing newly released opening moves.
pub type UnlockRecord = GenericUnlockRecord<String, UnlockDetail>;

/// Unlock ledger entry representing newly released opening moves.
pub type UnlockRecord = GenericUnlockRecord<String, UnlockDetail>;

/// Deterministically compute a card identifier for an opening edge.
#[must_use]
pub fn card_id_for_opening(owner_id: &str, edge_id: u64) -> u64 {
    hash64(&[owner_id.as_bytes(), &edge_id.to_be_bytes()])
}

/// Deterministically compute a card identifier for a tactic.
#[must_use]
pub fn card_id_for_tactic(owner_id: &str, tactic_id: u64) -> u64 {
    hash64(&[owner_id.as_bytes(), &tactic_id.to_be_bytes()])
}
