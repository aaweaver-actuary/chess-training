use review_domain::ids::{CardId, EdgeId, IdConversionError, IdKind, MoveId, PositionId};
use std::convert::TryFrom;

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
        Err(IdConversionError::Overflow { kind, .. }) if kind == IdKind::Card
    ));

    let negative = MoveId::try_from(-4096_i128);
    assert!(matches!(
        negative,
        Err(IdConversionError::Negative { kind, .. }) if kind == IdKind::Move
    ));
}

#[test]
fn review_domain_ids_are_available_in_scheduler_core() {
    let position = PositionId::new(61);
    assert_eq!(position.into_inner(), 61);
    assert_eq!(position.as_u64(), 61);
    assert_eq!(PositionId::try_from(61_i64).unwrap(), position);
    assert!(PositionId::try_from(-1_i64).is_err());
    assert_eq!(PositionId::try_from(61_i128).unwrap(), position);
    assert_eq!(PositionId::try_from(61_u128).unwrap(), position);

    let edge = EdgeId::from(67_u64);
    assert_eq!(u64::from(&edge), 67);
    assert_eq!(edge.into_inner(), 67);
    assert_eq!(EdgeId::try_from(67_i64).unwrap(), edge);
    assert!(EdgeId::try_from(-1_i64).is_err());
    assert_eq!(EdgeId::try_from(67_i128).unwrap(), edge);
    assert_eq!(EdgeId::try_from(67_u128).unwrap(), edge);

    let mv = MoveId::from(71_u64);
    assert_eq!(format!("{mv}"), "MoveId(71)");
    assert_eq!(format!("{mv:?}"), "MoveId(71)");
    assert_eq!(MoveId::try_from(71_i64).unwrap(), mv);
    assert!(MoveId::try_from(-1_i64).is_err());
    assert_eq!(MoveId::try_from(71_i128).unwrap(), mv);
    assert_eq!(MoveId::try_from(71_u128).unwrap(), mv);

    let card = CardId::new(73_u64);
    assert_eq!(card.to_string(), "CardId(73)");
    assert_eq!(u64::from(card), 73);
    assert_eq!(CardId::try_from(73_i64).unwrap(), card);
    assert!(CardId::try_from(-1_i64).is_err());
    assert_eq!(CardId::try_from(73_i128).unwrap(), card);
    assert_eq!(CardId::try_from(73_u128).unwrap(), card);
}
