//! chess-training-pgn-import — ingest PGN repertoires into review-domain structures.

/// Import configuration surface, including CLI defaults.
pub mod config;
/// Error types surfaced during configuration and parsing.
pub mod errors;
/// PGN importer implementation.
pub mod importer;
/// Intermediate data structures produced during import.
pub mod model;
/// Storage abstractions used by the importer.
pub mod storage;

/// Configuration parameters used to drive PGN ingestion.
pub use crate::config::IngestConfig;
/// Importer façade and error type exposed to binary crates.
pub use crate::importer::{ImportError, Importer};
/// In-memory storage implementation useful for tests and tooling.
pub use crate::storage::InMemoryImportStore;
