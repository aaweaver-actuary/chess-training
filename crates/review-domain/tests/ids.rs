use review_domain::ids::{CardId, EdgeId, IdConversionError, MoveId, PositionId};

#[test]
fn id_round_trips_through_u64_conversions() {
    let position = PositionId::from(42_u64);
    let edge = EdgeId::from(87_u64);
    let mov = MoveId::from(99_u64);
    let card = CardId::from(123_u64);

    assert_eq!(u64::from(position), 42);
    assert_eq!(u64::from(edge), 87);
    assert_eq!(u64::from(mov), 99);
    assert_eq!(u64::from(card), 123);
}

#[test]
fn try_from_i64_rejects_negative_values() {
    let negative = -27_i64;

    assert!(matches!(
        PositionId::try_from(negative),
        Err(IdConversionError::Negative { .. })
    ));
    assert!(matches!(
        EdgeId::try_from(negative),
        Err(IdConversionError::Negative { .. })
    ));
    assert!(matches!(
        MoveId::try_from(negative),
        Err(IdConversionError::Negative { .. })
    ));
    assert!(matches!(
        CardId::try_from(negative),
        Err(IdConversionError::Negative { .. })
    ));
}

#[test]
fn ids_order_by_their_numeric_value() {
    let low = CardId::from(1_u64);
    let high = CardId::from(2_u64);

    assert!(low < high);
    assert!(high > low);
}

#[cfg(feature = "serde")]
mod serde_support {
    use super::*;

    #[test]
    fn ids_serialize_and_deserialize_as_numbers() {
        let original = EdgeId::from(512_u64);
        let serialized = serde_json::to_string(&original).expect("serialize id");

        assert_eq!(serialized, "512");

        let round_trip: EdgeId = serde_json::from_str(&serialized).expect("deserialize id");

        assert_eq!(round_trip, original);
    }
}
