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
    /// Returns [`Err`] when the FEN omits or provides an invalid side-to-move field.
    pub fn new(fen: impl Into<String>, ply: u32) -> Result<Self, PositionError> {
        let fen = fen.into();
        let side_to_move = fen
            .split_whitespace()
            .nth(1)
            .and_then(|s| {
                let c = s.chars().next()?;
                matches!(c, 'w' | 'b').then_some(c)
            })
            .ok_or(PositionError::InvalidSideToMove)?;
        let id = hash64(&[fen.as_bytes()]);
        Ok(Self {
            id,
            fen,
            side_to_move,
            ply,
        })
    }
}
