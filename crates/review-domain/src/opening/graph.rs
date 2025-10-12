//! Opening repertoire adjacency graph structures.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::repertoire::RepertoireMove;

use super::OpeningEdge;

/// Errors that can occur while constructing an [`OpeningGraph`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OpeningGraphError {
    /// Duplicate edge identifiers are not permitted because they make hashes ambiguous.
    #[error("duplicate edge identifier {edge_id}")]
    DuplicateEdgeId { edge_id: u64 },
    /// A parent cannot have multiple edges leading to the same child position.
    #[error(
        "duplicate child {child_id} for parent {parent_id} (existing edge {existing_edge_id}, duplicate {duplicate_edge_id})"
    )]
    DuplicateChildEdge {
        parent_id: u64,
        child_id: u64,
        existing_edge_id: u64,
        duplicate_edge_id: u64,
    },
    /// Self loops violate DAG invariants required by repertoire traversal logic.
    #[error("self loop detected for position {position_id} on edge {edge_id}")]
    SelfLoop { edge_id: u64, position_id: u64 },
}

/// Node within an [`OpeningGraph`], tracking adjacency information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionNode {
    position_id: u64,
    children: Vec<OpeningEdge>,
    parents: BTreeSet<u64>,
}

impl PositionNode {
    fn new(position_id: u64) -> Self {
        Self {
            position_id,
            children: Vec::new(),
            parents: BTreeSet::new(),
        }
    }

    fn insert_child(&mut self, edge: OpeningEdge) -> Result<(), OpeningGraphError> {
        if let Some(existing) = self
            .children
            .iter()
            .find(|existing| existing.child_id == edge.child_id)
        {
            return Err(OpeningGraphError::DuplicateChildEdge {
                parent_id: self.position_id,
                child_id: edge.child_id,
                existing_edge_id: existing.id,
                duplicate_edge_id: edge.id,
            });
        }
        self.children.push(edge);
        Ok(())
    }

    fn add_parent(&mut self, parent_id: u64) {
        self.parents.insert(parent_id);
    }

    /// Outgoing edges from this position.
    #[must_use]
    pub fn children(&self) -> &[OpeningEdge] {
        &self.children
    }

    /// Incoming parent position identifiers.
    #[must_use]
    pub fn parents(&self) -> &BTreeSet<u64> {
        &self.parents
    }

    /// Identifier of the represented position.
    #[must_use]
    pub fn position_id(&self) -> u64 {
        self.position_id
    }
}

/// Adjacency representation of a repertoire's opening moves.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OpeningGraph {
    positions: HashMap<u64, PositionNode>,
}

impl OpeningGraph {
    /// Creates a new builder for assembling an [`OpeningGraph`].
    #[must_use]
    pub fn builder() -> OpeningGraphBuilder {
        OpeningGraphBuilder::new()
    }

    /// Convenience helper to build a graph directly from repertoire moves.
    ///
    /// # Errors
    ///
    /// Returns an [`OpeningGraphError`] when the provided moves contain
    /// duplicate identifiers, duplicate child edges, or self loops.
    pub fn try_from_moves<I>(moves: I) -> Result<Self, OpeningGraphError>
    where
        I: IntoIterator<Item = RepertoireMove>,
    {
        Ok(OpeningGraphBuilder::new().ingest_moves(moves)?.build())
    }

    /// Retrieves the node for a given position, if present.
    #[must_use]
    pub fn node(&self, position_id: u64) -> Option<&PositionNode> {
        self.positions.get(&position_id)
    }

    /// Outgoing edges from a position.
    #[must_use]
    pub fn children(&self, position_id: u64) -> Option<&[OpeningEdge]> {
        self.positions.get(&position_id).map(PositionNode::children)
    }

    /// Parent positions that lead into the specified node.
    #[must_use]
    pub fn parents(&self, position_id: u64) -> Option<&BTreeSet<u64>> {
        self.positions.get(&position_id).map(PositionNode::parents)
    }

    /// Iterator over all stored position identifiers.
    pub fn position_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.positions.keys().copied()
    }
}

/// Builder for [`OpeningGraph`], enforcing invariants during construction.
#[derive(Default)]
pub struct OpeningGraphBuilder {
    positions: HashMap<u64, PositionNode>,
    seen_edges: HashSet<u64>,
}

impl OpeningGraphBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            seen_edges: HashSet::new(),
        }
    }

    /// Ingests a single repertoire move, returning an updated builder on success.
    ///
    /// # Errors
    ///
    /// Returns an [`OpeningGraphError`] when the move would violate graph
    /// invariants (duplicate edge IDs, duplicate child edges, or self loops).
    pub fn ingest_move(mut self, move_entry: RepertoireMove) -> Result<Self, OpeningGraphError> {
        self.push_move(move_entry)?;
        Ok(self)
    }

    /// Ingests a collection of repertoire moves.
    ///
    /// # Errors
    ///
    /// Returns an [`OpeningGraphError`] when any move violates the graph
    /// invariants enforced by [`OpeningGraphBuilder::ingest_move`].
    pub fn ingest_moves<I>(mut self, moves: I) -> Result<Self, OpeningGraphError>
    where
        I: IntoIterator<Item = RepertoireMove>,
    {
        for move_entry in moves {
            self.push_move(move_entry)?;
        }
        Ok(self)
    }

    /// Finalises the builder into an [`OpeningGraph`].
    #[must_use]
    pub fn build(self) -> OpeningGraph {
        OpeningGraph {
            positions: self.positions,
        }
    }

    fn push_move(&mut self, move_entry: RepertoireMove) -> Result<(), OpeningGraphError> {
        let RepertoireMove {
            parent_id,
            child_id,
            edge_id,
            move_uci,
            move_san,
        } = move_entry;

        if parent_id == child_id {
            return Err(OpeningGraphError::SelfLoop {
                edge_id,
                position_id: parent_id,
            });
        }

        if !self.seen_edges.insert(edge_id) {
            return Err(OpeningGraphError::DuplicateEdgeId { edge_id });
        }

        let edge = OpeningEdge::new(edge_id, parent_id, child_id, move_uci, move_san);

        {
            let parent_node = self
                .positions
                .entry(parent_id)
                .or_insert_with(|| PositionNode::new(parent_id));
            parent_node.insert_child(edge)?;
        }

        let child_node = self
            .positions
            .entry(child_id)
            .or_insert_with(|| PositionNode::new(child_id));
        child_node.add_parent(parent_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn move_entry(edge: u64, parent: u64, child: u64) -> RepertoireMove {
        RepertoireMove::new(edge, parent, child, "e2e4", "e4")
    }

    #[test]
    fn builder_detects_duplicate_edges() {
        let mut builder = OpeningGraphBuilder::new();
        assert!(builder.push_move(move_entry(1, 10, 11)).is_ok());
        let err = builder.push_move(move_entry(1, 10, 12)).unwrap_err();
        assert_eq!(err, OpeningGraphError::DuplicateEdgeId { edge_id: 1 });
    }

    #[test]
    fn builder_detects_duplicate_children() {
        let mut builder = OpeningGraphBuilder::new();
        assert!(builder.push_move(move_entry(1, 10, 11)).is_ok());
        let err = builder.push_move(move_entry(2, 10, 11)).unwrap_err();
        assert_eq!(
            err,
            OpeningGraphError::DuplicateChildEdge {
                parent_id: 10,
                child_id: 11,
                existing_edge_id: 1,
                duplicate_edge_id: 2,
            }
        );
    }

    #[test]
    fn builder_detects_self_loops() {
        let mut builder = OpeningGraphBuilder::new();
        let err = builder.push_move(move_entry(3, 10, 10)).unwrap_err();
        assert_eq!(
            err,
            OpeningGraphError::SelfLoop {
                edge_id: 3,
                position_id: 10,
            }
        );
    }

    #[test]
    fn graph_exposes_adjacency_helpers() {
        let graph = OpeningGraph::try_from_moves(vec![
            move_entry(1, 1, 2),
            move_entry(2, 1, 3),
            move_entry(3, 2, 4),
        ])
        .expect("graph builds");

        let children = graph.children(1).unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.iter().any(|edge| edge.child_id == 2));
        let parents = graph.parents(4).unwrap();
        assert_eq!(parents.iter().copied().collect::<Vec<_>>(), vec![2]);
    }
}
