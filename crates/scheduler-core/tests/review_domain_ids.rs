use review_domain::ids::{CardId, EdgeId, IdentifierError, MoveId, PositionId};

#[test]
fn scheduler_core_uses_review_domain_identifier_wrappers() {
    let position = PositionId::from(1024_u64);
    assert_eq!(u64::from(position), 1024);

    let edge = EdgeId::try_from(2048_u128).expect("edge id converts");
    assert_eq!(edge.to_string(), "EdgeId(2048)");

    let move_id = MoveId::try_from(2048_i128).expect("move id from signed");
    assert_eq!(move_id.get(), 2048);

    let card = CardId::from(4096_u64);
    assert_eq!(card.to_string(), "CardId(4096)");

    let overflow = CardId::try_from(u128::from(u64::MAX) + 4096);
    assert!(matches!(
        overflow,
        Err(IdentifierError::Overflow { type_name, .. }) if type_name == "CardId"
    ));

    let negative = MoveId::try_from(-4096_i128);
    assert!(matches!(
        negative,
        Err(IdentifierError::Negative { type_name }) if type_name == "MoveId"
    ));
}
