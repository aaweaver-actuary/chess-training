use review_domain::opening::OpeningGraph;
use review_domain::repertoire::{Repertoire, RepertoireMove};

fn sample_moves() -> Vec<RepertoireMove> {
    vec![
        RepertoireMove::new(1, 10, 20, "e2e4", "e4"),
        RepertoireMove::new(2, 10, 30, "d2d4", "d4"),
        RepertoireMove::new(3, 20, 40, "g1f3", "Nf3"),
    ]
}

#[test]
fn opening_graph_from_repertoire_matches_moves() {
    let repertoire = Repertoire::from_iter(sample_moves().clone());
    let graph = OpeningGraph::from_repertoire(&repertoire);

    let children = graph.children(10);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].id, 1);
    assert_eq!(children[1].id, 2);
}

#[test]
fn opening_graph_unknowns_and_roots() {
    let graph = OpeningGraph::from_moves(sample_moves());
    assert!(graph.path_to(999).is_none());
    assert!(graph.children(999).is_empty());
    assert!(graph.parents(999).is_empty());

    let root_path = graph.path_to(10).expect("root exists");
    assert!(root_path.is_empty());
}

#[test]
fn opening_graph_cycle_protection() {
    let mut moves = sample_moves();
    moves.push(RepertoireMove::new(1, 10, 20, "e2e4", "e4"));
    let deduped = OpeningGraph::from_moves(moves.clone());
    assert_eq!(deduped.children(10).len(), 2);

    moves.push(RepertoireMove::new(4, 40, 10, "a2a3", "a3"));
    let cyclic_graph = OpeningGraph::from_moves(moves);
    assert!(cyclic_graph.path_to(10).is_none());
}
