use review_domain::{OpeningGraph, RepertoireMove};

fn sample_graph() -> OpeningGraph {
    let moves = vec![
        RepertoireMove::new(1, 1, 2, "e2e4", "e4"),
        RepertoireMove::new(2, 2, 3, "g1f3", "Nf3"),
        RepertoireMove::new(3, 1, 4, "d2d4", "d4"),
        RepertoireMove::new(4, 4, 5, "c2c4", "c4"),
        RepertoireMove::new(5, 3, 6, "f1c4", "Bc4"),
        RepertoireMove::new(6, 10, 11, "a2a3", "a3"),
    ];

    OpeningGraph::from_moves(moves)
}

#[test]
fn children_returns_outgoing_edges_for_position() {
    let graph = sample_graph();

    let child_edges: Vec<_> = graph.children(1).collect();
    assert_eq!(child_edges.len(), 2);
    assert_eq!(child_edges[0].edge_id, 1);
    assert_eq!(child_edges[1].edge_id, 3);

    let empty: Vec<_> = graph.children(42).collect();
    assert!(empty.is_empty());
}

#[test]
fn parents_returns_incoming_edges_for_position() {
    let graph = sample_graph();

    let parent_edges: Vec<_> = graph.parents(3).collect();
    assert_eq!(parent_edges.len(), 1);
    assert_eq!(parent_edges[0].edge_id, 2);

    let root_parents: Vec<_> = graph.parents(1).collect();
    assert!(root_parents.is_empty());
}

#[test]
fn roots_reports_positions_without_parents() {
    let graph = sample_graph();
    assert_eq!(graph.roots(), &[1, 10]);
}

#[test]
fn path_to_returns_sequence_from_root() {
    let graph = sample_graph();

    let path_to_six: Vec<_> = graph
        .path_to(6)
        .expect("position reachable")
        .into_iter()
        .map(|edge| edge.edge_id)
        .collect();
    assert_eq!(path_to_six, vec![1, 2, 5]);

    let root_path = graph.path_to(1).expect("root present");
    assert!(root_path.is_empty());

    assert!(graph.path_to(99).is_none());
}
