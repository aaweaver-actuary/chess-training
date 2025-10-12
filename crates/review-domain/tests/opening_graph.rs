use review_domain::opening::graph::{OpeningGraph, OpeningGraphError};
use review_domain::repertoire::RepertoireMove;

fn sample_moves() -> Vec<RepertoireMove> {
    vec![
        RepertoireMove::new(10, 1, 2, "e2e4", "e4"),
        RepertoireMove::new(20, 1, 3, "d2d4", "d4"),
        RepertoireMove::new(30, 2, 4, "g1f3", "Nf3"),
    ]
}

#[test]
fn builder_produces_adjacency_graph() {
    let graph = OpeningGraph::builder()
        .ingest_moves(sample_moves())
        .expect("ingestion succeeds")
        .build();

    let root_children = graph.children(1).expect("root position present in graph");
    assert_eq!(root_children.len(), 2);
    assert!(root_children.iter().any(|edge| edge.child_id == 2));
    assert!(root_children.iter().any(|edge| edge.child_id == 3));

    let node_two_parents: Vec<_> = graph
        .parents(2)
        .expect("child node tracked")
        .iter()
        .copied()
        .collect();
    assert_eq!(node_two_parents, vec![1]);

    let node_four_parents: Vec<_> = graph
        .parents(4)
        .expect("grandchild node tracked")
        .iter()
        .copied()
        .collect();
    assert_eq!(node_four_parents, vec![2]);
}

#[test]
fn duplicate_edge_ids_are_rejected() {
    let err = OpeningGraph::builder()
        .ingest_moves(vec![
            RepertoireMove::new(42, 1, 2, "e2e4", "e4"),
            RepertoireMove::new(42, 1, 3, "d2d4", "d4"),
        ])
        .err()
        .expect("duplicate edge identifiers should fail");

    assert!(matches!(
        err,
        OpeningGraphError::DuplicateEdgeId { edge_id } if edge_id == 42
    ));
}

#[test]
fn duplicate_children_from_parent_are_rejected() {
    let err = OpeningGraph::builder()
        .ingest_moves(vec![
            RepertoireMove::new(100, 5, 6, "e2e4", "e4"),
            RepertoireMove::new(200, 5, 6, "e2e4", "e4"),
        ])
        .err()
        .expect("duplicate child edges must be rejected");

    assert!(matches!(
        err,
        OpeningGraphError::DuplicateChildEdge {
            parent_id,
            child_id,
            existing_edge_id,
            duplicate_edge_id,
        } if parent_id == 5 && child_id == 6 && existing_edge_id == 100 && duplicate_edge_id == 200
    ));
}

#[test]
fn self_loops_are_rejected() {
    let err = OpeningGraph::builder()
        .ingest_moves(vec![RepertoireMove::new(7, 9, 9, "e2e4", "e4")])
        .err()
        .expect("self loops should not be allowed");

    assert!(matches!(
        err,
        OpeningGraphError::SelfLoop { edge_id, position_id }
            if edge_id == 7 && position_id == 9
    ));
}
