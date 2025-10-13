use review_domain::ids::{CardId, EdgeId, IdConversionError, IdKind, MoveId, PositionId};

#[test]
fn review_domain_ids_are_available_to_importer_consumers() {
    let position = PositionId::from(64_u64);
    assert_eq!(position.to_string(), "PositionId(64)");

    let edge = EdgeId::try_from(64_u128).expect("edge conversion");
    assert_eq!(edge.get(), 64);

    let move_id = MoveId::try_from(64_i128).expect("move conversion");
    assert_eq!(u64::from(move_id), 64);

    let card = CardId::from(512_u64);
    assert_eq!(u64::from(card), 512);

    let parsed: CardId = "512".parse().expect("parse card id");
    assert_eq!(parsed, card);

    let overflow = PositionId::try_from(u128::from(u64::MAX) + 5);
    assert!(matches!(
        overflow,
        Err(IdConversionError::Overflow { kind, .. }) if kind == IdKind::Position
    ));

    let negative = MoveId::try_from(-32_i128);
    assert!(matches!(
        negative,
        Err(IdConversionError::Negative { kind, .. }) if kind == IdKind::Move
    ));
}
