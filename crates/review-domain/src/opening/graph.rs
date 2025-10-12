//! Opening graph adjacency structure for repertoire traversal.

use std::collections::{HashMap, HashSet};

use crate::{
    opening::OpeningEdge,
    repertoire::{Repertoire, RepertoireMove},
};

/// Directed acyclic graph representation of opening positions and edges.
#[derive(Default)]
pub struct OpeningGraph {
    edges: HashMap<u64, OpeningEdge>,
    outgoing: HashMap<u64, Vec<u64>>, // parent position -> edge ids
    incoming: HashMap<u64, Vec<u64>>, // child position -> edge ids
}

impl OpeningGraph {
    /// Builds a graph from the moves stored in a [`Repertoire`].
    #[must_use]
    pub fn from_repertoire(repertoire: &Repertoire) -> Self {
        Self::from_moves(repertoire.moves().iter().cloned())
    }

    /// Builds a graph from an iterator of repertoire moves.
    #[must_use]
    pub fn from_moves<I>(moves: I) -> Self
    where
        I: IntoIterator<Item = RepertoireMove>,
    {
        let mut graph = Self::default();
        for mv in moves {
            let edge = OpeningEdge::new(
                mv.edge_id,
                mv.parent_id,
                mv.child_id,
                mv.move_uci,
                mv.move_san,
            );
            graph.insert_edge(edge);
        }
        graph
    }

    /// Returns the edges that originate from the provided parent position.
    #[must_use]
    pub fn children(&self, position_id: u64) -> Vec<&OpeningEdge> {
        self.outgoing
            .get(&position_id)
            .into_iter()
            .flat_map(|edge_ids| edge_ids.iter())
            .filter_map(|edge_id| self.edges.get(edge_id))
            .collect()
    }

    /// Returns the edges that lead into the provided child position.
    #[must_use]
    pub fn parents(&self, position_id: u64) -> Vec<&OpeningEdge> {
        self.incoming
            .get(&position_id)
            .into_iter()
            .flat_map(|edge_ids| edge_ids.iter())
            .filter_map(|edge_id| self.edges.get(edge_id))
            .collect()
    }

    /// Returns the ordered path of edges from a root to the requested position, if it exists.
    #[must_use]
    pub fn path_to(&self, position_id: u64) -> Option<Vec<&OpeningEdge>> {
        if !self.contains_position(position_id) {
            return None;
        }

        let mut path = Vec::new();
        let mut current = position_id;
        let mut visited = HashSet::new();

        while let Some(parents) = self.incoming.get(&current) {
            if parents.is_empty() {
                break;
            }

            // Follow the first recorded parent edge to maintain deterministic traversal.
            let edge_id = parents[0];
            let edge = self.edges.get(&edge_id)?;

            if !visited.insert(current) {
                // Cycle detected; abort rather than looping forever.
                return None;
            }

            path.push(edge);
            current = edge.parent_id;
        }

        path.reverse();
        Some(path)
    }

    fn contains_position(&self, position_id: u64) -> bool {
        self.outgoing.contains_key(&position_id)
            || self.incoming.contains_key(&position_id)
            || self
                .edges
                .values()
                .any(|edge| edge.parent_id == position_id || edge.child_id == position_id)
    }

    fn insert_edge(&mut self, edge: OpeningEdge) {
        if self.edges.contains_key(&edge.id) {
            return;
        }

        self.outgoing
            .entry(edge.parent_id)
            .or_default()
            .push(edge.id);
        self.incoming
            .entry(edge.child_id)
            .or_default()
            .push(edge.id);
        self.edges.insert(edge.id, edge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_moves() -> Vec<RepertoireMove> {
        vec![
            RepertoireMove::new(1, 10, 20, "e2e4", "e4"),
            RepertoireMove::new(2, 10, 30, "d2d4", "d4"),
            RepertoireMove::new(3, 20, 40, "g1f3", "Nf3"),
        ]
    }

    #[test]
    fn children_returns_edges_for_parent() {
        let repertoire = Repertoire::from_iter(sample_moves().clone());
        let graph = OpeningGraph::from_repertoire(&repertoire);

        let children = graph.children(10);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].id, 1);
        assert_eq!(children[1].id, 2);
    }

    #[test]
    fn parents_returns_edges_for_child() {
        let graph = OpeningGraph::from_moves(sample_moves());

        let parents = graph.parents(40);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].id, 3);
    }

    #[test]
    fn path_to_collects_route_from_root() {
        let graph = OpeningGraph::from_moves(sample_moves());

        let path = graph.path_to(40).expect("path exists");
        let ids: Vec<u64> = path.iter().map(|edge| edge.id).collect();
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn unknown_position_has_no_path() {
        let graph = OpeningGraph::from_moves(sample_moves());
        assert!(graph.path_to(999).is_none());
        assert!(graph.children(999).is_empty());
        assert!(graph.parents(999).is_empty());
    }

    #[test]
    fn root_position_has_empty_path() {
        let graph = OpeningGraph::from_moves(sample_moves());
        let path = graph.path_to(10).expect("root exists");
        assert!(path.is_empty());
    }
}
