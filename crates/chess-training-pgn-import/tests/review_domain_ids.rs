use std::convert::TryFrom;

use review_domain::ids::{CardId, EdgeId, MoveId, PositionId};

#[test]
fn review_domain_ids_support_runtime_conversions() {
    let position = PositionId::new(41);
    assert_eq!(position.into_inner(), 41);
    assert_eq!(position.as_u64(), 41);
    assert_eq!(PositionId::try_from(41_i64).unwrap(), position);
    assert!(PositionId::try_from(-1_i64).is_err());
    assert_eq!(PositionId::try_from(41_i128).unwrap(), position);
    assert_eq!(PositionId::try_from(41_u128).unwrap(), position);

    let edge = EdgeId::from(43_u64);
    assert_eq!(u64::from(&edge), 43);
    assert_eq!(edge.into_inner(), 43);
    assert_eq!(EdgeId::try_from(43_i64).unwrap(), edge);
    assert!(EdgeId::try_from(-1_i64).is_err());
    assert_eq!(EdgeId::try_from(43_i128).unwrap(), edge);
    assert_eq!(EdgeId::try_from(43_u128).unwrap(), edge);

    let mv = MoveId::from(47_u64);
    assert_eq!(format!("{mv}"), "47");
    assert_eq!(format!("{mv:?}"), "MoveId(47)");
    assert_eq!(MoveId::try_from(47_i64).unwrap(), mv);
    assert!(MoveId::try_from(-1_i64).is_err());
    assert_eq!(MoveId::try_from(47_i128).unwrap(), mv);
    assert_eq!(MoveId::try_from(47_u128).unwrap(), mv);

    let card = CardId::new(53_u64);
    assert_eq!(card.to_string(), "53");
    assert_eq!(u64::from(card), 53);
    assert_eq!(CardId::try_from(53_i64).unwrap(), card);
    assert!(CardId::try_from(-1_i64).is_err());
    assert_eq!(CardId::try_from(53_i128).unwrap(), card);
    assert_eq!(CardId::try_from(53_u128).unwrap(), card);
}
