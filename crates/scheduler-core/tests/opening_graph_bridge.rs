use review_domain::opening::graph::{OpeningGraph, OpeningGraphError};
use review_domain::repertoire::RepertoireMove;

#[test]
fn opening_graph_is_accessible_to_scheduler() {
    let graph = OpeningGraph::builder()
        .ingest_moves(vec![
            RepertoireMove::new(5, 100, 101, "e2e4", "e4"),
            RepertoireMove::new(6, 101, 102, "e7e5", "...e5"),
        ])
        .expect("scheduler dependencies should ingest moves")
        .build();

    assert_eq!(
        graph
            .parents(101)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![100]
    );
    assert_eq!(graph.children(100).unwrap()[0].child_id, 101);

    let duplicate = OpeningGraph::builder()
        .ingest_move(RepertoireMove::new(8, 200, 201, "d2d4", "d4"))
        .expect("first move accepted")
        .ingest_move(RepertoireMove::new(8, 200, 202, "c2c4", "c4"))
        .err()
        .expect("duplicate edge detected");
    assert!(matches!(
        duplicate,
        OpeningGraphError::DuplicateEdgeId { edge_id } if edge_id == 8
    ));
}
