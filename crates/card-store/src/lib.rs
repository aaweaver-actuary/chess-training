//! card-store — unified persistence traits and in-memory implementation.
#![allow(unexpected_cfgs)]

pub mod chess_position;
pub mod config;
pub mod errors;
pub mod memory;
pub mod model;
pub mod store;

pub use crate::errors::PositionError;
pub use crate::store::{CardStore, StoreError};

pub use review_domain::hash64;

#[cfg(test)]
pub(crate) mod tests;
