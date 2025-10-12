use review_domain::{OpeningGraph, OpeningGraphBuildError, RepertoireMove};

fn sample_move(edge: u64, parent: u64, child: u64, uci: &str, san: &str) -> RepertoireMove {
    RepertoireMove::new(edge, parent, child, uci, san)
}

#[test]
fn builder_constructs_adjacency_metadata() {
    let moves = vec![
        sample_move(1, 10, 20, "e2e4", "e4"),
        sample_move(2, 20, 30, "e7e5", "...e5"),
        sample_move(3, 10, 40, "g1f3", "Nf3"),
    ];

    let graph = OpeningGraph::builder()
        .extend(moves.clone())
        .build()
        .expect("graph builds");

    assert_eq!(graph.edge_count(), 3);
    assert_eq!(graph.position_count(), 4);

    let mut root_positions: Vec<_> = graph.root_positions().collect();
    root_positions.sort_unstable();
    assert_eq!(root_positions, vec![10]);

    let children = graph.children_of(10).expect("root has children");
    assert_eq!(children.len(), 2);
    let child_ids: Vec<_> = children.iter().map(|edge| edge.child_id).collect();
    assert_eq!(child_ids, vec![20, 40]);

    let parents = graph.parents_of(30).expect("child has parents");
    assert_eq!(parents, &[20]);
}

#[test]
fn builder_rejects_duplicate_edges() {
    let result = OpeningGraph::builder()
        .add_move(sample_move(1, 10, 20, "e2e4", "e4"))
        .add_move(sample_move(1, 10, 21, "e2e4", "e4"))
        .build();

    let err = result.expect_err("duplicate edge should fail");
    assert!(matches!(
        err,
        OpeningGraphBuildError::DuplicateEdge { edge_id: 1 }
    ));
}

#[test]
fn builder_rejects_duplicate_transitions() {
    let result = OpeningGraph::builder()
        .add_move(sample_move(1, 10, 20, "e2e4", "e4"))
        .add_move(sample_move(2, 10, 20, "e2e4", "e4"))
        .build();

    let err = result.expect_err("duplicate transition should fail");
    assert!(matches!(
        err,
        OpeningGraphBuildError::DuplicateTransition {
            parent_id: 10,
            child_id: 20
        }
    ));
}
