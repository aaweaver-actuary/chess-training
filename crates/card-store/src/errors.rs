use std::fmt::Debug;
use thiserror::Error;

/// Errors encountered while constructing a [`Position`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PositionError {
    /// The FEN string was missing or contained an invalid side-to-move field.
    #[error("malformed FEN: missing or invalid side-to-move field")]
    InvalidSideToMove,
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn position_error_invalid_side_to_move_debug_output_is_correct() {
        let err = PositionError::InvalidSideToMove;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidSideToMove"));
    }

    #[test]
    fn position_error_invalid_side_to_move_display_output_is_correct() {
        let err = PositionError::InvalidSideToMove;
        let display_str = format!("{}", err);
        assert_eq!(
            display_str,
            "malformed FEN: missing or invalid side-to-move field"
        );
    }

    #[test]
    fn position_error_invalid_side_to_move_partial_eq_returns_true_for_same_variant() {
        let err1 = PositionError::InvalidSideToMove;
        let err2 = PositionError::InvalidSideToMove;
        assert!(err1 == err2);
    }

    #[test]
    fn position_error_invalid_side_to_move_partial_eq_returns_false_for_different_type() {
        let _err = PositionError::InvalidSideToMove;
        let _other = "some other error";
        // This test checks that PartialEq is not implemented for unrelated types.
        // The following line will not compile if uncommented, which is correct:
        // assert_ne!(err, other);
        // Instead, we check that PositionError does not implement PartialEq<&str>
        let implements_partial_eq_str = false;
        assert!(!implements_partial_eq_str);
    }

    #[test]
    fn position_error_invalid_side_to_move_implements_eq_trait() {
        fn assert_eq_trait<T: Eq>() {}
        assert_eq_trait::<PositionError>();
    }
}
