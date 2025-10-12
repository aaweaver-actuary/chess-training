use review_domain::opening::graph::{OpeningGraph, OpeningGraphError};
use review_domain::repertoire::RepertoireMove;

fn sample_moves() -> Vec<RepertoireMove> {
    vec![
        RepertoireMove::new(11, 1, 2, "e2e4", "e4"),
        RepertoireMove::new(22, 2, 3, "e7e5", "...e5"),
        RepertoireMove::new(33, 1, 4, "g1f3", "Nf3"),
    ]
}

#[test]
fn opening_graph_builds_and_exposes_adjacency() {
    let graph = OpeningGraph::builder()
        .ingest_moves(sample_moves())
        .expect("valid move set ingests")
        .build();

    let mut positions: Vec<_> = graph.position_ids().collect();
    positions.sort_unstable();
    assert_eq!(positions, vec![1, 2, 3, 4]);

    let node = graph.node(1).expect("root position exists");
    assert_eq!(node.position_id(), 1);
    assert_eq!(node.children().len(), 2);

    let parents_of_three: Vec<_> = graph.parents(3).unwrap().iter().copied().collect();
    assert_eq!(parents_of_three, vec![2]);

    let children_of_one = graph.children(1).unwrap();
    assert!(children_of_one.iter().any(|edge| edge.child_id == 2));
    assert!(children_of_one.iter().any(|edge| edge.child_id == 4));

    let rebuilt = OpeningGraph::try_from_moves(vec![
        RepertoireMove::new(44, 5, 6, "d2d4", "d4"),
        RepertoireMove::new(55, 6, 7, "d7d5", "...d5"),
    ])
    .expect("try_from_moves succeeds");

    assert_eq!(
        rebuilt
            .parents(7)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![6]
    );
    assert_eq!(rebuilt.children(5).unwrap().len(), 1);
}

#[test]
fn opening_graph_detects_invariant_violations() {
    let duplicate_edge_err = OpeningGraph::builder()
        .ingest_moves(vec![
            RepertoireMove::new(99, 1, 2, "e2e4", "e4"),
            RepertoireMove::new(99, 1, 3, "d2d4", "d4"),
        ])
        .err()
        .expect("duplicate edge ids rejected");
    assert!(matches!(
        duplicate_edge_err,
        OpeningGraphError::DuplicateEdgeId { edge_id } if edge_id == 99
    ));

    let duplicate_child_err = OpeningGraph::builder()
        .ingest_move(RepertoireMove::new(10, 1, 2, "e2e4", "e4"))
        .expect("first move accepted")
        .ingest_move(RepertoireMove::new(20, 1, 2, "e2e4", "e4"))
        .err()
        .expect("duplicate child rejected");
    assert!(matches!(
        duplicate_child_err,
        OpeningGraphError::DuplicateChildEdge {
            parent_id,
            child_id,
            existing_edge_id,
            duplicate_edge_id,
        } if parent_id == 1 && child_id == 2 && existing_edge_id == 10 && duplicate_edge_id == 20
    ));

    let self_loop_err = OpeningGraph::builder()
        .ingest_moves(vec![RepertoireMove::new(7, 4, 4, "e2e4", "e4")])
        .err()
        .expect("self loop rejected");
    assert!(matches!(
        self_loop_err,
        OpeningGraphError::SelfLoop { edge_id, position_id }
            if edge_id == 7 && position_id == 4
    ));
}
