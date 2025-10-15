#![allow(dead_code)]

use thiserror::Error;

/// Error enumeration for quiz engine failures.
/// This includes errors related to PGN parsing and format validation.
///
/// # Variants
/// - `UnreadablePgn(String)`: Wraps failures encountered while parsing SAN tokens from PGN input.
/// - `MultipleGames`: Raised when the provided PGN text contains more than one game entry.
/// - `VariationsUnsupported`: Raised when the PGN contains nested variations or alternate main lines.
/// - `WrongFormat`: Raised when the PGN includes unsupported annotations or lacks a single main line.
/// - `NoMoves`: Raised when a PGN entry parses but does not provide any playable moves.
/// - `Io`: Adapter-facing error for underlying I/O failures.
///
/// # Examples
/// ```rust
/// use quiz_core::QuizError;
/// let error = QuizError::UnreadablePgn("invalid token".to_string());
/// assert_eq!(format!("{error}"), "failed to parse PGN: invalid token");
/// ```
#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuizError {
    /// Wraps failures encountered while parsing SAN tokens from PGN input.
    #[error("failed to parse PGN: {0}")]
    UnreadablePgn(String),
    /// Raised when the provided PGN text contains more than one game entry.
    #[error("PGN must contain exactly one game")]
    MultipleGames,
    /// Raised when the PGN contains nested variations or alternate main lines.
    #[error("variations are not supported in quiz mode")]
    VariationsUnsupported,
    /// Raised when the PGN includes unsupported annotations or lacks a single main line.
    #[error("expected a single main line of moves")]
    WrongFormat,
    /// Raised when a PGN entry parses but does not provide any playable moves.
    #[error("PGN did not contain any moves")]
    NoMoves,
    /// Adapter-facing error for underlying I/O failures.
    #[error("I/O error")]
    Io,
}
