use review_domain::RepertoireMove;
use review_domain::opening::{EdgeInput, OpeningGraph};
use serde_json::json;

#[test]
fn opening_graph_flattens_to_legacy_edges_snapshot() {
    let edges: Vec<_> = vec![
        EdgeInput {
            parent_id: 1,
            move_uci: "e2e4".to_string(),
            move_san: "e4".to_string(),
            child_id: 2,
        }
        .into_edge(),
        EdgeInput {
            parent_id: 2,
            move_uci: "g1f3".to_string(),
            move_san: "Nf3".to_string(),
            child_id: 3,
        }
        .into_edge(),
        EdgeInput {
            parent_id: 2,
            move_uci: "f1c4".to_string(),
            move_san: "Bc4".to_string(),
            child_id: 4,
        }
        .into_edge(),
    ];

    let graph = OpeningGraph::from_edges(edges.clone());
    let legacy_moves = graph.legacy_moves();

    let expected: Vec<RepertoireMove> = edges
        .into_iter()
        .map(|edge| {
            RepertoireMove::new(
                edge.id,
                edge.parent_id,
                edge.child_id,
                edge.move_uci,
                edge.move_san,
            )
        })
        .collect();

    let expected_json: Vec<_> = expected
        .iter()
        .map(|mv| {
            json!({
                "edge_id": mv.edge_id,
                "parent_id": mv.parent_id,
                "child_id": mv.child_id,
                "move_uci": mv.move_uci,
                "move_san": mv.move_san,
            })
        })
        .collect();

    insta::assert_json_snapshot!(
        &expected_json,
        @r###"[
  {
    "child_id": 2,
    "edge_id": 14975286395967137125,
    "move_san": "e4",
    "move_uci": "e2e4",
    "parent_id": 1
  },
  {
    "child_id": 3,
    "edge_id": 804340966091102782,
    "move_san": "Nf3",
    "move_uci": "g1f3",
    "parent_id": 2
  },
  {
    "child_id": 4,
    "edge_id": 4389009945857421815,
    "move_san": "Bc4",
    "move_uci": "f1c4",
    "parent_id": 2
  }
]"###
    );

    assert_eq!(legacy_moves, expected);
}
