#![allow(dead_code)]

use thiserror::Error;

/// Error enumeration for quiz engine failures.
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
