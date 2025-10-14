
/// Errors encountered while constructing a [`ChessPosition`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PositionError {
    /// The FEN string did not provide all required fields.
    #[error("malformed FEN: expected 6 space-delimited fields")]
    MalformedFen,
    /// The FEN string was missing or contained an invalid side-to-move field.
    #[error("malformed FEN: missing or invalid side-to-move field")]
    InvalidSideToMove,
    /// The FEN string contained an invalid piece placement field.
    #[error("malformed FEN: invalid piece placement field")]
    InvalidPiecePlacement,
}
