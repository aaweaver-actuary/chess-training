#![allow(dead_code)]

use shakmaty::san::{ParseSanError, SanError};
use std::io;
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

/// Convenience result alias used across the quiz engine and adapters.
pub type QuizResult<T> = Result<T, QuizError>;

/// Specialised alias emphasising adapter-facing interactions.
pub type AdapterResult<T> = QuizResult<T>;

impl From<io::Error> for QuizError {
    fn from(_: io::Error) -> Self {
        QuizError::Io
    }
}

impl From<ParseSanError> for QuizError {
    fn from(err: ParseSanError) -> Self {
        QuizError::UnreadablePgn(err.to_string())
    }
}

impl From<SanError> for QuizError {
    fn from(err: SanError) -> Self {
        QuizError::UnreadablePgn(err.to_string())
    }
}

impl QuizError {
    pub(crate) fn unreadable_from_parse(token: impl Into<String>, err: &ParseSanError) -> Self {
        let token = token.into();
        let detail = err.to_string();

        if token.is_empty() {
            QuizError::UnreadablePgn(detail)
        } else {
            QuizError::UnreadablePgn(format!("{token}: {detail}"))
        }
    }

    pub(crate) fn unreadable_from_san(token: impl Into<String>, err: &SanError) -> Self {
        let token = token.into();
        let detail = err.to_string();

        if token.is_empty() {
            QuizError::UnreadablePgn(detail)
        } else {
            QuizError::UnreadablePgn(format!("{token}: {detail}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::QuizError;
    use shakmaty::san::{ParseSanError, SanError};
    use std::io;

    #[test]
    fn converts_io_error_into_quiz_error() {
        let io_error = io::Error::other("boom");
        let quiz_error: QuizError = io_error.into();

        assert_eq!(quiz_error, QuizError::Io);
    }

    #[test]
    fn converts_parse_san_error_into_unreadable_pgn() {
        let parse_error = ParseSanError;
        let quiz_error: QuizError = parse_error.into();

        assert_eq!(quiz_error, QuizError::UnreadablePgn("invalid san".into()));
    }

    #[test]
    fn converts_san_error_into_unreadable_pgn() {
        let san_error = SanError::AmbiguousSan;
        let quiz_error: QuizError = san_error.into();

        assert_eq!(quiz_error, QuizError::UnreadablePgn("ambiguous san".into()));
    }
}
