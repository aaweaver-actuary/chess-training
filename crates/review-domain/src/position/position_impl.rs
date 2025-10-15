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
#[cfg(test)]
mod tests {
    use super::*;

    fn make_position(fen: &str) -> Position {
        Position::new(fen)
    }

    #[test]
    fn test_new_assigns_id_and_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = make_position(fen);
        assert_eq!(pos.fen, fen);
        // The id should be deterministic for the same FEN
        let pos2 = make_position(fen);
        assert_eq!(pos.id, pos2.id);
    }

    #[test]
    fn test_new_different_fens_have_different_ids() {
        let fen1 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen2 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let pos1 = make_position(fen1);
        let pos2 = make_position(fen2);
        assert_ne!(pos1.id, pos2.id);
    }

    #[test]
    fn test_side_to_move_white() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), Some('w'));
    }

    #[test]
    fn test_side_to_move_black() {
        let fen = "8/8/8/8/8/8/8/8 b - - 0 1";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), Some('b'));
    }

    #[test]
    fn test_side_to_move_invalid_side() {
        let fen = "8/8/8/8/8/8/8/8 x - - 0 1";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), None);
    }

    #[test]
    fn test_side_to_move_missing_side() {
        // FEN missing side to move
        let fen = "8/8/8/8/8/8/8/8";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), None);
    }

    #[test]
    fn test_side_to_move_empty_fen() {
        let fen = "";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), None);
    }

    #[test]
    fn test_side_to_move_short_fen() {
        // Only one field
        let fen = "8/8/8/8/8/8/8/8";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), None);
    }

    #[test]
    fn test_side_to_move_extra_spaces() {
        // Extra spaces between fields
        let fen = "8/8/8/8/8/8/8/8    w   - - 0 1";
        let pos = make_position(fen);
        assert_eq!(pos.side_to_move(), Some('w'));
    }

    #[test]
    fn test_clone_and_eq() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let pos1 = make_position(fen);
        let pos2 = pos1.clone();
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn test_debug_format() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let pos = make_position(fen);
        let debug_str = format!("{pos:?}");
        assert!(debug_str.contains("Position"));
        assert!(debug_str.contains("fen"));
    }
}
