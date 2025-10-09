//! Domain model structs shared by card-store implementations.

use std::num::NonZeroU8;

use chrono::NaiveDate;

use review_domain::{CardKind as GenericCardKind, UnlockRecord as GenericUnlockRecord};

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

/// Opening edge describing a transition between two positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    /// Deterministic edge identifier computed from the parent and move.
    pub id: u64,
    /// Parent position identifier.
    pub parent_id: u64,
    /// Child position identifier.
    pub child_id: u64,
    /// Move in UCI notation.
    pub move_uci: String,
    /// Move in SAN notation.
    pub move_san: String,
}

impl Edge {
    /// Builds a deterministic [`Edge`] from an [`EdgeInput`].
    pub fn from_input(input: EdgeInput) -> Self {
        let EdgeInput {
            parent_id,
            child_id,
            move_uci,
            move_san,
        } = input;
        let id = hash64(&[&parent_id.to_be_bytes(), move_uci.as_bytes()]);
        Self {
            id,
            parent_id,
            child_id,
            move_uci,
            move_san,
        }
    }
}

/// Payload carried by opening cards in the store layer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OpeningCard {
    /// Identifier of the reviewed edge.
    pub edge_id: u64,
}

/// Payload carried by tactic cards in the store layer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TacticCard {
    /// Identifier of the reviewed tactic.
    pub tactic_id: u64,
}

/// Classification of a card target.
pub type CardKind = GenericCardKind<OpeningCard, TacticCard>;

/// Mutable scheduling state of a card.
#[derive(Clone, Debug, PartialEq)]
pub struct CardState {
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

impl CardState {
    /// Creates a new [`CardState`] with sensible defaults.
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
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    /// Stable card identifier (owner + target).
    pub id: u64,
    /// Owner/user identifier.
    pub owner_id: String,
    /// Card classification.
    pub kind: CardKind,
    /// Scheduling state.
    pub state: CardState,
}

impl Card {
    /// Convenience accessor for the due date.
    pub fn due_on(&self) -> NaiveDate {
        self.state.due_on
    }

    /// Updates the mutable scheduling state.
    pub fn update_state(&mut self, updater: impl FnOnce(&mut CardState)) {
        updater(&mut self.state);
    }
}

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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnlockDetail {
    /// Edge unlocked for the user.
    pub edge_id: u64,
}

/// Unlock ledger entry representing newly released opening moves.
pub type UnlockRecord = GenericUnlockRecord<String, UnlockDetail>;

/// Deterministically compute a card identifier for an opening edge.
pub fn card_id_for_opening(owner_id: &str, edge_id: u64) -> u64 {
    hash64(&[owner_id.as_bytes(), &edge_id.to_be_bytes()])
}

/// Deterministically compute a card identifier for a tactic.
pub fn card_id_for_tactic(owner_id: &str, tactic_id: u64) -> u64 {
    hash64(&[owner_id.as_bytes(), &tactic_id.to_be_bytes()])
}
