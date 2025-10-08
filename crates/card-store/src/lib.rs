//! card-store — unified persistence traits and in-memory implementation.

pub mod config;
pub mod memory;
pub mod model;
pub mod store;

pub use crate::store::{CardStore, StoreError};
