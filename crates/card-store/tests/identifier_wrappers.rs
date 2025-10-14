use std::fmt::Write;

use review_domain::ids::{CardId, EdgeId, IdConversionError, MoveId, PositionId};

#[test]
fn id_conversion_errors_surface_kind_labels() {
    let overflow = CardId::try_from(u128::from(u64::MAX) + 1).expect_err("overflow should error");

    assert!(matches!(
        overflow,
        IdConversionError::Overflow { value, .. } if value == u128::from(u64::MAX) + 1
    ));
}

#[test]
fn ids_integrate_with_card_store_helpers() {
    let mut buffer = String::new();

    let position = PositionId::from(42_u64);
    let edge = EdgeId::from(72_u64);
    let mov = MoveId::from(99_u64);
    let card = CardId::from(7_u64);

    write!(&mut buffer, "{position}:{edge}:{mov}:{card}").unwrap();

    assert_eq!(buffer, "PositionId(42):EdgeId(72):MoveId(99):CardId(7)");
    assert_eq!(u64::from(position), 42);
    assert_eq!(u64::from(edge), 72);
    assert_eq!(u64::from(mov), 99);
    assert_eq!(u64::from(card), 7);
}
