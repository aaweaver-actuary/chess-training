use review_domain::opening::graph::{OpeningGraph, OpeningGraphError};
use review_domain::repertoire::RepertoireMove;

#[test]
fn opening_graph_smoke_from_importer_dependency() {
    let graph = OpeningGraph::builder()
        .ingest_moves(vec![
            RepertoireMove::new(1, 10, 11, "e2e4", "e4"),
            RepertoireMove::new(2, 11, 12, "e7e5", "...e5"),
            RepertoireMove::new(3, 10, 13, "g1f3", "Nf3"),
        ])
        .expect("builder accepts unique edges")
        .build();

    assert_eq!(
        graph
            .parents(11)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![10]
    );
    assert_eq!(graph.children(10).unwrap().len(), 2);

    let duplicate_error = OpeningGraph::builder()
        .ingest_moves(vec![
            RepertoireMove::new(9, 1, 2, "e2e4", "e4"),
            RepertoireMove::new(9, 1, 3, "d2d4", "d4"),
        ])
        .err()
        .expect("duplicate edge ids produce error");
    assert!(matches!(
        duplicate_error,
        OpeningGraphError::DuplicateEdgeId { edge_id } if edge_id == 9
    ));

    let self_loop_error = OpeningGraph::builder()
        .ingest_moves(vec![RepertoireMove::new(7, 4, 4, "e2e4", "e4")])
        .err()
        .expect("self loop disallowed");
    assert!(matches!(
        self_loop_error,
        OpeningGraphError::SelfLoop { edge_id, position_id }
            if edge_id == 7 && position_id == 4
    ));
}
