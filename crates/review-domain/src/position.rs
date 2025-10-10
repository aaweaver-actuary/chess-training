//! Shared representation of chess positions used across review services.

use crate::hash::hash64;

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

/// Chess position represented by a FEN string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChessPosition {
    /// Stable identifier derived from the [`fen`](Self::fen).
    pub id: u64,
    /// Full FEN string.
    pub fen: String,
    /// Side to move extracted from the FEN (`'w'` or `'b'`).
    pub side_to_move: char,
    /// Distance in plies from the start position.
    pub ply: u32,
}

impl ChessPosition {
    /// Creates a new [`ChessPosition`] using a deterministic hash of the FEN as the identifier.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::MalformedFen`] when the FEN does not contain exactly 6
    /// space-delimited fields, or when any field is empty.
    ///
    /// Returns [`PositionError::InvalidSideToMove`] when the FEN does not contain a
    /// valid side-to-move segment.
    ///
    /// Returns [`PositionError::InvalidPiecePlacement`] when the FEN contains invalid
    /// characters in the piece placement field.
    #[must_use = "inspect the result to detect invalid chess positions"]
    pub fn new(fen: impl Into<String>, ply: u32) -> Result<Self, PositionError> {
        let fen = fen.into();
        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.len() != 6 || parts.iter().any(|segment| segment.is_empty()) {
            return Err(PositionError::MalformedFen);
        }

        let side_to_move = parts[1]
            .chars()
            .next()
            .filter(|c| matches!(c, 'w' | 'b'))
            .ok_or(PositionError::InvalidSideToMove)?;

        if !parts[0].chars().all(|c| {
            matches!(
                c,
                '/' | '1'
                    ..='8' | 'K' | 'Q' | 'R' | 'B' | 'N' | 'P' | 'k' | 'q' | 'r' | 'b' | 'n' | 'p'
            )
        }) {
            return Err(PositionError::InvalidPiecePlacement);
        }
        let id = hash64(&[fen.as_bytes()]);
        Ok(Self {
            id,
            fen,
            side_to_move,
            ply,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_side_to_move_debug_output_is_informative() {
        let err = PositionError::InvalidSideToMove;
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidSideToMove"));
    }

    #[test]
    fn malformed_fen_rejected() {
        let result = ChessPosition::new("invalid", 0);
        assert!(matches!(result, Err(PositionError::MalformedFen)));
    }

    #[test]
    fn invalid_piece_placement_rejected() {
        let fen = "8/8/8/8/8/8/8/8x w - - 0 1";
        let result = ChessPosition::new(fen, 0);
        assert!(matches!(result, Err(PositionError::InvalidPiecePlacement)));
    }

    #[test]
    fn valid_position_is_constructed() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let position = ChessPosition::new(fen, 0).expect("valid position");
        assert_eq!(position.side_to_move, 'w');
        assert_eq!(position.ply, 0);
        assert_eq!(position.fen, "8/8/8/8/8/8/8/8 w - - 0 1");
    }
}
