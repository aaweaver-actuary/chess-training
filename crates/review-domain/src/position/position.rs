use super::PositionId;
use crate::hash_with_seed;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    /// Stable identifier derived from hashing the FEN.
    pub id: PositionId,
    /// Full FEN string describing the position.
    pub fen: String,
}

impl Position {
    /// Construct a position with a deterministic identifier derived from the FEN.
    #[must_use]
    pub fn new(fen: &str) -> Self {
        let id = hash_with_seed(fen);
        Self {
            id: PositionId::new(id),
            fen: fen.to_string(),
        }
    }

    /// Return the side to move, or `None` if the FEN is malformed.
    #[must_use]
    pub fn side_to_move(&self) -> Option<char> {
        self.fen
            .split(' ')
            .nth(1)?
            .chars()
            .next()
            .filter(|c| matches!(c, 'w' | 'b'))
    }
}
