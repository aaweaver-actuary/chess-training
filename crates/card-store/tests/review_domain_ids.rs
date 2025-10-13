use review_domain::ids::{
    CardId, EdgeId, IdConversionError, IdKind, LearnerId, MoveId, PositionId, UnlockId,
};

#[test]
fn review_domain_identifiers_round_trip_from_card_store() {
    let position = PositionId::from(5_u64);
    assert_eq!(position.get(), 5);
    assert_eq!(u64::from(position), 5);

    let learner = LearnerId::from(11_u64);
    assert_eq!(learner.to_string(), "LearnerId(11)");

    let edge = EdgeId::try_from(8_u128).expect("edge id converts");
    assert_eq!(edge.get(), 8);
    assert_eq!(edge.to_string(), "EdgeId(8)");

    let move_id = MoveId::try_from(8_i128).expect("move id from signed");
    assert_eq!(move_id.get(), 8);

    let parsed: CardId = "42".parse().expect("parse card id");
    assert_eq!(parsed, CardId::from(42_u64));

    let unlock = UnlockId::from(99_u64);
    assert_eq!(u64::from(unlock), 99);

    let overflow = CardId::try_from(u128::from(u64::MAX) + 1);
    assert!(matches!(
        overflow,
        Err(IdConversionError::Overflow { kind, value, max })
            if kind == IdKind::Card
                && value == u128::from(u64::MAX) + 1
                && max == u64::MAX
    ));

    let negative = MoveId::try_from(-7_i128);
    assert!(matches!(
        negative,
        Err(IdConversionError::Negative { kind, value })
            if kind == IdKind::Move && value == -7
    ));
}
