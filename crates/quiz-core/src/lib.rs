//! Core quiz engine crate scaffolding.
//!
//! Modules and adapters are placeholders that will be implemented in later tasks.

pub mod engine;
pub mod errors;
pub mod ports;
pub mod source;
pub mod state;

pub use engine::QuizEngine;
pub use errors::{AdapterResult, QuizError, QuizResult};
pub use ports::{FeedbackMessage, PromptContext, QuizPort};
pub use source::QuizSource;
pub use state::{AttemptResult, AttemptState, QuizSession, QuizStep, QuizSummary};

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "wasm")]
pub mod wasm;
