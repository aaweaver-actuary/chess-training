use review_domain::ids::{CardId, EdgeId, IdConversionError, LearnerId, MoveId, UnlockId};

#[test]
fn review_domain_identifiers_round_trip_from_card_store() {
    let learner = LearnerId::from(11_u64);
    assert_eq!(learner.to_string(), "LearnerId(11)");

    let edge = EdgeId::try_from(8_u128).expect("edge id converts");
    assert_eq!(edge.get(), 8);
    assert_eq!(edge.to_string(), "EdgeId(8)");

    let move_id = MoveId::try_from(8_u128).expect("move id from unsigned");
    assert_eq!(move_id.get(), 8);

    let parsed = CardId::from(42_u64);
    assert_eq!(parsed, CardId::new(42));

    let unlock = UnlockId::from(99_u64);
    assert_eq!(u64::from(unlock), 99);

    let overflow = CardId::try_from(u128::from(u64::MAX) + 1);
    assert!(matches!(
        overflow,
        Err(IdConversionError::Overflow { value, .. }) if value == u128::from(u64::MAX) + 1
    ));
}
