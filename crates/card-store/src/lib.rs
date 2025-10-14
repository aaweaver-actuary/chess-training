//! card-store — unified persistence traits and in-memory implementation.
#![allow(unexpected_cfgs)]

/// Chess position re-exports shared with review-domain.
pub mod chess_position;
/// Storage configuration helpers.
pub mod config;
/// Error compatibility types for persistence operations.
pub mod errors;
/// In-memory store implementation and helpers.
pub mod memory;
/// Domain model types tailored to storage needs.
pub mod model;
/// Persistence trait definitions used by services.
pub mod store;

/// Error returned when chess positions fail validation.
pub use crate::errors::PositionError;
/// Core store trait and error surface for persistence implementations.
pub use crate::store::{ReviewCardStore, StoreError};

/// Deterministic hashing helper shared with review-domain.
pub use review_domain::hash64;

#[cfg(test)]
pub(crate) mod tests;
