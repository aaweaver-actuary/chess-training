use crate::store::StoreError;

/// Helper used in integration tests to detect invalid position errors.
pub fn is_invalid_position(err: &StoreError) -> bool {
    matches!(err, StoreError::InvalidPosition(_))
}

/// Asserts that the provided error represents an invalid chess position.
pub fn assert_invalid_position(err: &StoreError) {
    assert!(
        is_invalid_position(err),
        "expected invalid position error, got {err:?}"
    );
}
