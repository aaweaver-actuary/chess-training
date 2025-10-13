use review_domain::ids::{CardId, EdgeId, IdConversionError, MoveId, PositionId};

#[test]
fn u64_roundtrip_for_all_identifiers() {
    let position_id = PositionId::from(1_u64);
    let edge_id = EdgeId::from(2_u64);
    let move_id = MoveId::from(3_u64);
    let card_id = CardId::from(4_u64);

    assert_eq!(u64::from(position_id), 1);
    assert_eq!(u64::from(edge_id), 2);
    assert_eq!(u64::from(move_id), 3);
    assert_eq!(u64::from(card_id), 4);
}

#[test]
fn try_from_rejects_out_of_range_values() {
    let too_large = u128::from(u64::MAX) + 1;

    assert_eq!(
        PositionId::try_from(too_large),
        Err(IdConversionError::Overflow { value: too_large })
    );
    assert_eq!(
        EdgeId::try_from(too_large),
        Err(IdConversionError::Overflow { value: too_large })
    );
    assert_eq!(
        MoveId::try_from(too_large),
        Err(IdConversionError::Overflow { value: too_large })
    );
    assert_eq!(
        CardId::try_from(too_large),
        Err(IdConversionError::Overflow { value: too_large })
    );
}

#[test]
fn identifier_wrappers_are_copy_and_ord() {
    fn assert_copy<T: Copy>() {}
    fn assert_ord<T: Ord>() {}

    assert_copy::<PositionId>();
    assert_copy::<EdgeId>();
    assert_copy::<MoveId>();
    assert_copy::<CardId>();

    assert_ord::<PositionId>();
    assert_ord::<EdgeId>();
    assert_ord::<MoveId>();
    assert_ord::<CardId>();
}
