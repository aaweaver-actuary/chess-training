use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{RepertoireMove, opening::OpeningEdge};

/// Graph representation of opening repertoire moves.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OpeningGraph {
    adjacency: HashMap<u64, Vec<OpeningEdge>>,
    parents: HashMap<u64, Vec<u64>>,
    root_positions: Vec<u64>,
    edge_count: usize,
    position_count: usize,
}

/// Builder that constructs [`OpeningGraph`] instances from repertoire moves.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OpeningGraphBuilder {
    moves: Vec<RepertoireMove>,
}

/// Error produced when building an [`OpeningGraph`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OpeningGraphBuildError {
    /// The provided edge identifier already exists within the graph.
    #[error("duplicate edge identifier {edge_id}")]
    DuplicateEdge { edge_id: u64 },
    /// A parent/child pair was specified multiple times in the input.
    #[error("duplicate transition from {parent_id} to {child_id}")]
    DuplicateTransition { parent_id: u64, child_id: u64 },
}

impl OpeningGraph {
    /// Creates a new builder for constructing an [`OpeningGraph`].
    #[must_use]
    pub fn builder() -> OpeningGraphBuilder {
        OpeningGraphBuilder::new()
    }

    /// Constructs an [`OpeningGraph`] directly from the provided iterator of moves.
    ///
    /// # Errors
    ///
    /// Returns [`OpeningGraphBuildError`] if duplicate edges or transitions are encountered.
    pub fn from_moves<I>(moves: I) -> Result<Self, OpeningGraphBuildError>
    where
        I: IntoIterator<Item = RepertoireMove>,
    {
        OpeningGraphBuilder::new().extend(moves).build()
    }

    /// Total number of edges contained in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    /// Total number of unique positions represented in the graph.
    #[must_use]
    pub fn position_count(&self) -> usize {
        self.position_count
    }

    /// Iterator over root position identifiers.
    #[must_use = "iterate over the returned positions to access the graph roots"]
    pub fn root_positions(&self) -> impl Iterator<Item = u64> + '_ {
        self.root_positions.iter().copied()
    }

    /// Outgoing edges for the provided parent position.
    #[must_use]
    pub fn children_of(&self, position_id: u64) -> Option<&[OpeningEdge]> {
        self.adjacency.get(&position_id).map(Vec::as_slice)
    }

    /// Parent position identifiers for the provided child position.
    #[must_use]
    pub fn parents_of(&self, position_id: u64) -> Option<&[u64]> {
        self.parents.get(&position_id).map(Vec::as_slice)
    }
}

impl OpeningGraphBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self { moves: Vec::new() }
    }

    /// Adds a move to the builder.
    #[must_use]
    pub fn add_move(mut self, mv: RepertoireMove) -> Self {
        self.moves.push(mv);
        self
    }

    /// Extends the builder with an iterator of moves.
    #[must_use]
    pub fn extend<I: IntoIterator<Item = RepertoireMove>>(mut self, moves: I) -> Self {
        self.moves.extend(moves);
        self
    }

    /// Builds the graph from the collected moves.
    ///
    /// # Errors
    ///
    /// Returns [`OpeningGraphBuildError`] if duplicate edges or transitions are encountered.
    pub fn build(self) -> Result<OpeningGraph, OpeningGraphBuildError> {
        build_graph(self.moves)
    }
}

fn build_graph(moves: Vec<RepertoireMove>) -> Result<OpeningGraph, OpeningGraphBuildError> {
    let mut adjacency: HashMap<u64, Vec<OpeningEdge>> = HashMap::new();
    let mut parents: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut edge_ids = HashSet::new();
    let mut transitions = HashSet::new();
    let mut positions = BTreeSet::new();

    for mv in moves {
        let RepertoireMove {
            parent_id,
            child_id,
            edge_id,
            move_uci,
            move_san,
        } = mv;

        if !edge_ids.insert(edge_id) {
            return Err(OpeningGraphBuildError::DuplicateEdge { edge_id });
        }

        if !transitions.insert((parent_id, child_id)) {
            return Err(OpeningGraphBuildError::DuplicateTransition {
                parent_id,
                child_id,
            });
        }

        positions.insert(parent_id);
        positions.insert(child_id);

        let edge = OpeningEdge {
            id: edge_id,
            parent_id,
            child_id,
            move_uci,
            move_san,
        };

        adjacency.entry(parent_id).or_default().push(edge);
        adjacency.entry(child_id).or_default();

        parents.entry(parent_id).or_default();
        parents.entry(child_id).or_default().push(parent_id);
    }

    let root_positions = positions
        .iter()
        .copied()
        .filter(|position_id| parents.get(position_id).is_none_or(Vec::is_empty))
        .collect();

    Ok(OpeningGraph {
        edge_count: edge_ids.len(),
        position_count: positions.len(),
        adjacency,
        parents,
        root_positions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_move(edge: u64, parent: u64, child: u64, uci: &str, san: &str) -> RepertoireMove {
        RepertoireMove::new(edge, parent, child, uci, san)
    }

    #[test]
    fn builder_is_equivalent_to_from_moves() {
        let moves = vec![
            sample_move(1, 2, 3, "e2e4", "e4"),
            sample_move(2, 3, 4, "e7e5", "...e5"),
        ];

        let builder_graph = OpeningGraph::builder()
            .extend(moves.clone())
            .build()
            .unwrap();
        let from_moves_graph = OpeningGraph::from_moves(moves).unwrap();

        assert_eq!(builder_graph, from_moves_graph);
    }

    #[test]
    fn empty_builder_produces_empty_graph() {
        let graph = OpeningGraph::builder().build().unwrap();
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.position_count(), 0);
        assert!(graph.root_positions().next().is_none());
    }

    #[test]
    fn detects_duplicate_edge() {
        let result = build_graph(vec![
            sample_move(1, 10, 11, "e2e4", "e4"),
            sample_move(1, 10, 12, "e2e4", "e4"),
        ]);

        assert_eq!(
            result,
            Err(OpeningGraphBuildError::DuplicateEdge { edge_id: 1 })
        );
    }

    #[test]
    fn detects_duplicate_transition() {
        let result = build_graph(vec![
            sample_move(1, 10, 11, "e2e4", "e4"),
            sample_move(2, 10, 11, "g1f3", "Nf3"),
        ]);

        assert_eq!(
            result,
            Err(OpeningGraphBuildError::DuplicateTransition {
                parent_id: 10,
                child_id: 11,
            })
        );
    }

    #[test]
    fn populates_root_positions() {
        let graph = build_graph(vec![
            sample_move(1, 1, 2, "e2e4", "e4"),
            sample_move(2, 2, 3, "e7e5", "...e5"),
            sample_move(3, 4, 5, "d2d4", "d4"),
        ])
        .unwrap();

        let roots: BTreeSet<_> = graph.root_positions().collect();
        assert_eq!(roots, BTreeSet::from([1, 4]));
    }
}
