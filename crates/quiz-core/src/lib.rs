//! Core quiz engine crate scaffolding.
//!
//! Modules and adapters are placeholders that will be implemented in later tasks.

pub mod engine;
pub mod errors;
pub mod ports;
pub mod state;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "wasm")]
pub mod wasm;
