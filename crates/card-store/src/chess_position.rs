use crate::{PositionError, hash64};

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
    /// Creates a new [`Position`] using a deterministic hash of the FEN as the identifier.
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
