use std::collections::HashSet;
use std::convert::TryFrom;

use review_domain::ids::{CardId, EdgeId, MoveId, PositionId};

#[test]
fn id_newtypes_round_trip_from_u64() {
    let position = PositionId::from(42_u64);
    let edge = EdgeId::from(7_u64);
    let mv = MoveId::from(99_u64);
    let card = CardId::from(123_u64);

    assert_eq!(u64::from(position), 42);
    assert_eq!(u64::from(edge), 7);
    assert_eq!(u64::from(mv), 99);
    assert_eq!(u64::from(card), 123);
}

#[test]
fn id_newtypes_support_try_from_i64() {
    let ok = PositionId::try_from(15_i64).expect("positive values should convert");
    assert_eq!(u64::from(ok), 15);

    let err = PositionId::try_from(-2_i64).expect_err("negative values should fail");
    assert_eq!(
        err.to_string(),
        "out of range integral type conversion attempted"
    );
}

#[test]
fn id_newtypes_are_hashable_and_ordered() {
    let mut set = HashSet::new();
    set.insert(PositionId::from(5));
    assert!(set.contains(&PositionId::from(5)));

    let mut ids = vec![MoveId::from(3), MoveId::from(1), MoveId::from(2)];
    ids.sort();
    assert_eq!(ids, vec![MoveId::from(1), MoveId::from(2), MoveId::from(3)]);
}

#[cfg(feature = "serde")]
#[test]
fn id_newtypes_serialize_as_u64() {
    let json = serde_json::to_string(&EdgeId::from(64_u64)).expect("serialization succeeds");
    assert_eq!(json, "64");

    let value: EdgeId = serde_json::from_str(&json).expect("deserialization succeeds");
    assert_eq!(value, EdgeId::from(64_u64));
}
