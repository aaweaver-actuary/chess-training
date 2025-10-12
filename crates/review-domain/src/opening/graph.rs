//! Opening graph representation with helpers for legacy serialization.

use std::collections::HashMap;

use crate::{opening::OpeningEdge, repertoire::RepertoireMove};

/// Node representing a position within an [`OpeningGraph`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PositionNode {
    /// Unique identifier for the position.
    pub position_id: u64,
    /// Outgoing edges leading to child positions.
    pub edges: Vec<OpeningEdge>,
}

impl PositionNode {
    /// Creates a new [`PositionNode`] with no outgoing edges.
    #[must_use]
    pub fn new(position_id: u64) -> Self {
        Self {
            position_id,
            edges: Vec::new(),
        }
    }

    /// Registers an outgoing [`OpeningEdge`] from this position.
    pub fn add_edge(&mut self, edge: OpeningEdge) {
        self.edges.push(edge);
    }
}

/// Directed graph of opening positions and edges.
#[derive(Clone, Debug, Default)]
pub struct OpeningGraph {
    positions: HashMap<u64, PositionNode>,
    edge_sequence: Vec<OpeningEdge>,
}

impl OpeningGraph {
    /// Builds an [`OpeningGraph`] from an iterator of edges.
    #[must_use]
    pub fn from_edges<I>(edges: I) -> Self
    where
        I: IntoIterator<Item = OpeningEdge>,
    {
        let mut positions = HashMap::new();
        let mut edge_sequence = Vec::new();

        for edge in edges {
            positions
                .entry(edge.parent_id)
                .or_insert_with(|| PositionNode::new(edge.parent_id))
                .add_edge(edge.clone());

            positions
                .entry(edge.child_id)
                .or_insert_with(|| PositionNode::new(edge.child_id));

            edge_sequence.push(edge);
        }

        Self {
            positions,
            edge_sequence,
        }
    }

    /// Flattens the graph back into the legacy edge list ordering.
    #[must_use]
    pub fn legacy_moves(&self) -> Vec<RepertoireMove> {
        self.edge_sequence
            .iter()
            .map(|edge| {
                RepertoireMove::new(
                    edge.id,
                    edge.parent_id,
                    edge.child_id,
                    edge.move_uci.clone(),
                    edge.move_san.clone(),
                )
            })
            .collect()
    }

    /// Retrieves a position node by its identifier.
    #[must_use]
    pub fn position(&self, position_id: u64) -> Option<&PositionNode> {
        self.positions.get(&position_id)
    }

    /// Returns an iterator over all tracked positions.
    pub fn positions(&self) -> impl Iterator<Item = &PositionNode> {
        self.positions.values()
    }
}
