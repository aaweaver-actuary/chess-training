use review_domain::{
    ids::{EdgeId, PositionId},
    repertoire::{Repertoire, RepertoireBuilder, RepertoireError, RepertoireMove},
};

#[test]
fn repertoire_collects_moves() {
    let mut repertoire = Repertoire::new("e4 starts");
    assert_eq!(repertoire.name(), "e4 starts");
    assert!(repertoire.moves().is_empty());
    assert!(repertoire.graph().is_empty());

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

#[test]
fn builder_supports_composing_repertoire() {
    let repertoire = RepertoireBuilder::new("builder test")
        .add_move(RepertoireMove::new(
            EdgeId::new(10),
            PositionId::new(20),
            PositionId::new(21),
            "e2e4",
            "e4",
        ))
        .extend([RepertoireMove::new(
            EdgeId::new(11),
            PositionId::new(21),
            PositionId::new(22),
            "g1f3",
            "Nf3",
        )])
        .build();

    assert_eq!(repertoire.name(), "builder test");
    assert_eq!(repertoire.moves().len(), 2);
    assert_eq!(repertoire.moves()[0].move_uci, "e2e4");
    assert_eq!(repertoire.moves()[1].move_san, "Nf3");

    let children: Vec<_> = repertoire
        .graph()
        .children(PositionId::new(20))
        .map(|mv| mv.edge_id.get())
        .collect();
    assert_eq!(children, vec![10]);
}

#[test]
fn repertoire_collect_from_iterator_preserves_moves() {
    let moves = vec![
        RepertoireMove::new(
            EdgeId::new(1),
            PositionId::new(1),
            PositionId::new(2),
            "e2e4",
            "e4",
        ),
        RepertoireMove::new(
            EdgeId::new(2),
            PositionId::new(2),
            PositionId::new(3),
            "d2d4",
            "d4",
        ),
    ];

    let repertoire: Repertoire = moves.clone().into_iter().collect();

    assert_eq!(repertoire.name(), "");
    assert_eq!(repertoire.moves(), &moves[..]);
    assert_eq!(repertoire.graph().len(), 2);
}

#[test]
fn repertoire_move_constructor_accepts_string_inputs() {
    let mv = RepertoireMove::new(
        EdgeId::new(7),
        PositionId::new(8),
        PositionId::new(9),
        String::from("e7e5"),
        String::from("...e5"),
    );

    assert_eq!(mv.edge_id, EdgeId::new(7));
    assert_eq!(mv.parent_id, PositionId::new(8));
    assert_eq!(mv.child_id, PositionId::new(9));
    assert_eq!(mv.move_uci, "e7e5");
    assert_eq!(mv.move_san, "...e5");
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
