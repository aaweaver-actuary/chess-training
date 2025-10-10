use crate::store::StoreError;

pub fn is_invalid_position(err: &StoreError) -> bool {
    matches!(err, StoreError::InvalidPosition(_))
}

pub fn assert_invalid_position(err: StoreError) {
    if !is_invalid_position(&err) {
        panic!("expected invalid position error, got {err:?}");
    }
}
