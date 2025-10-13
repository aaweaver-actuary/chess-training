use review_domain::{
    ids::{EdgeId, PositionId},
    repertoire::{Repertoire, RepertoireError, RepertoireMove},
};

#[test]
fn repertoire_collects_moves() {
    let mut repertoire = Repertoire::new("e4 starts");
    assert_eq!(repertoire.name(), "e4 starts");
    assert!(repertoire.moves().is_empty());

    let move_entry = RepertoireMove::new(
        EdgeId::new(1),
        PositionId::new(2),
        PositionId::new(3),
        "e2e4",
        "e4",
    );
    let result = repertoire.add_move(move_entry.clone());
    assert_eq!(result, Err(RepertoireError::not_implemented("add_move")));
    assert!(
        repertoire.moves().is_empty(),
        "stub should not mutate the repertoire yet"
    );
}

#[test]
fn remove_move_stub_returns_expected_error() {
    let mut repertoire = Repertoire::new("queen's gambit");
    let result = repertoire.remove_move(EdgeId::new(42));
    assert_eq!(result, Err(RepertoireError::not_implemented("remove_move")));
}

#[cfg(feature = "serde")]
#[test]
fn repertoire_serializes_to_json() {
    let repertoire = Repertoire::new("catalan");
    let json = serde_json::to_string(&repertoire).expect("serialization succeeds");
    assert!(json.contains("catalan"));
}

#[cfg(feature = "avro")]
#[test]
fn repertoire_exposes_avro_schema() {
    let schema = Repertoire::avro_schema();
    let schema_json = schema.canonical_form();
    assert!(schema_json.contains("Repertoire"));
}
