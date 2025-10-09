use thiserror::Error;

/// Errors encountered while constructing a [`Position`].
#[derive(Debug, Error, PartialEq, Eq)]
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
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn position_error_invalid_side_to_move_debug_output_is_correct() {
        let err = PositionError::InvalidSideToMove;
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidSideToMove"));
    }

    #[test]
    fn position_error_malformed_fen_debug_output_is_correct() {
        let err = PositionError::MalformedFen;
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("MalformedFen"));
    }

    #[test]
    fn position_error_invalid_side_to_move_display_output_is_correct() {
        let err = PositionError::InvalidSideToMove;
        let display_str = format!("{err}");
        assert_eq!(
            display_str,
            "malformed FEN: missing or invalid side-to-move field"
        );
    }

    #[test]
    fn position_error_malformed_fen_display_output_is_correct() {
        let err = PositionError::MalformedFen;
        let display_str = format!("{err}");
        assert_eq!(
            display_str,
            "malformed FEN: expected 6 space-delimited fields"
        );
    }

    #[test]
    fn position_error_invalid_piece_placement_display_output_is_correct() {
        let err = PositionError::InvalidPiecePlacement;
        let display_str = format!("{err}");
        assert_eq!(display_str, "malformed FEN: invalid piece placement field");
    }

    #[test]
    fn position_error_invalid_side_to_move_partial_eq_returns_true_for_same_variant() {
        let err1 = PositionError::InvalidSideToMove;
        let err2 = PositionError::InvalidSideToMove;
        assert_eq!(err1, err2);
    }

    #[test]
    fn position_error_malformed_fen_partial_eq_returns_true_for_same_variant() {
        let err1 = PositionError::MalformedFen;
        let err2 = PositionError::MalformedFen;
        assert_eq!(err1, err2);
    }

    #[test]
    fn position_error_invalid_piece_placement_partial_eq_returns_true_for_same_variant() {
        let err1 = PositionError::InvalidPiecePlacement;
        let err2 = PositionError::InvalidPiecePlacement;
        assert_eq!(err1, err2);
    }

    #[test]
    fn position_error_invalid_side_to_move_implements_eq_trait() {
        fn assert_eq_trait<T: Eq>() {}
        assert_eq_trait::<PositionError>();
    }
}
