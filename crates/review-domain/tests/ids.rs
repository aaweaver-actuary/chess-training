use review_domain::ids::{CardId, EdgeId, IdentifierError, MoveId, PositionId};

#[test]
fn position_id_round_trips_through_from_and_into() {
    let raw = 42_u64;
    let id = PositionId::from(raw);

    assert_eq!(id.get(), raw);
    assert_eq!(u64::from(id), raw);
}

#[test]
fn identifiers_support_display_output() {
    let id = EdgeId::from(17_u64);
    assert_eq!(id.to_string(), "EdgeId(17)");
}

#[test]
fn card_id_rejects_values_that_do_not_fit_into_u64() {
    let raw = u128::from(u64::MAX) + 1;
    let result = CardId::try_from(raw);

    assert!(matches!(
        result,
        Err(IdentifierError::Overflow {
            type_name,
            attempted_value
        }) if type_name == "CardId" && attempted_value == raw
    ));
}

#[test]
fn move_id_from_signed_value_validates_non_negative() {
    let result = MoveId::try_from(-1_i128);

    assert!(
        matches!(result, Err(IdentifierError::Negative { type_name }) if type_name == "MoveId")
    );
}

#[cfg(feature = "serde")]
#[test]
fn identifiers_serialize_as_plain_numbers() {
    let id = EdgeId::from(99_u64);
    let serialized = serde_json::to_string(&id).expect("serialize id");

    assert_eq!(serialized, "99");
    let roundtrip: EdgeId = serde_json::from_str(&serialized).expect("deserialize id");
    assert_eq!(roundtrip, id);
}

#[test]
fn parsing_negative_string_reports_negative_error() {
    let result = "-5".parse::<MoveId>();

    assert!(matches!(
        result,
        Err(IdentifierError::Negative { type_name }) if type_name == "MoveId"
    ));
}
