use std::fmt::Write;

use review_domain::ids::{CardId, EdgeId, IdConversionError, IdKind, MoveId, PositionId};

#[test]
fn id_conversion_errors_surface_kind_labels() {
    let overflow = CardId::try_from(u128::from(u64::MAX) + 1).expect_err("overflow should error");
    let negative = EdgeId::try_from(-1_i64).expect_err("negative should error");

    match overflow {
        IdConversionError::Overflow { kind, value, max } => {
            assert_eq!(kind, IdKind::Card);
            assert_eq!(value, u128::from(u64::MAX) + 1);
            assert_eq!(max, u64::MAX);
            assert_eq!(kind.to_string(), "card");
        }
        IdConversionError::Negative { .. } | IdConversionError::InvalidFormat { .. } => {
            panic!("expected overflow")
        }
    }

    match negative {
        IdConversionError::Negative { kind, value } => {
            assert_eq!(kind, IdKind::Edge);
            assert_eq!(value, -1);
            assert_eq!(kind.to_string(), "edge");
        }
        IdConversionError::Overflow { .. } | IdConversionError::InvalidFormat { .. } => {
            panic!("expected negative")
        }
    }
}

#[test]
fn ids_integrate_with_card_store_helpers() {
    let mut buffer = String::new();

    let position = PositionId::from(42_u64);
    let edge = EdgeId::from(72_u64);
    let mov = MoveId::from(99_u64);
    let card = CardId::from(7_u64);

    write!(
        &mut buffer,
        "{}:{}:{}:{}",
        position.get(),
        edge.get(),
        mov.get(),
        card.get()
    )
    .unwrap();

    assert_eq!(buffer, "42:72:99:7");
    assert_eq!(u64::from(position), 42);
    assert_eq!(u64::from(edge), 72);
    assert_eq!(u64::from(mov), 99);
    assert_eq!(u64::from(card), 7);
}
