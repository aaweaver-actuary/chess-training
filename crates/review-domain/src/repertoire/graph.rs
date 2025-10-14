use std::collections::BTreeMap;

use crate::ids::{EdgeId, PositionId};

use super::RepertoireMove;

/// Adjacency structure representing an opening repertoire as a directed graph.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OpeningGraph {
    moves: Vec<RepertoireMove>,
    by_edge: BTreeMap<EdgeId, usize>,
    outgoing: BTreeMap<PositionId, Vec<usize>>,
    incoming: BTreeMap<PositionId, Vec<usize>>,
}

impl OpeningGraph {
    /// Creates an empty graph with no positions or edges.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a graph from an owned collection of repertoire moves.
    #[must_use]
    pub fn from_moves(moves: Vec<RepertoireMove>) -> Self {
        let mut graph = Self::default();
        for mv in moves {
            graph.insert_move(mv);
        }
        graph
    }

    fn insert_move(&mut self, mv: RepertoireMove) {
        let index = self.moves.len();
        self.by_edge.insert(mv.edge_id, index);
        self.outgoing.entry(mv.parent_id).or_default().push(index);
        self.incoming.entry(mv.child_id).or_default().push(index);
        self.moves.push(mv);
    }

    /// Returns the number of edges contained in the graph.
    #[must_use]
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Indicates whether the graph contains any edges.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Provides immutable access to all repertoire moves backing the graph.
    #[must_use]
    pub fn moves(&self) -> &[RepertoireMove] {
        &self.moves
    }

    /// Returns an iterator over the moves that depart from the provided parent position.
    #[must_use]
    pub fn children(&self, parent_id: PositionId) -> impl Iterator<Item = &RepertoireMove> + '_ {
        self.outgoing
            .get(&parent_id)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|&idx| &self.moves[idx]))
    }

    /// Returns an iterator over the moves that lead into the provided child position.
    #[must_use]
    pub fn parents(&self, child_id: PositionId) -> impl Iterator<Item = &RepertoireMove> + '_ {
        self.incoming
            .get(&child_id)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|&idx| &self.moves[idx]))
    }

    /// Finds a move by its edge identifier.
    #[must_use]
    pub fn edge(&self, edge_id: EdgeId) -> Option<&RepertoireMove> {
        self.by_edge.get(&edge_id).map(|&idx| &self.moves[idx])
    }

    /// Iterates over all moves contained in the graph in insertion order.
    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = &RepertoireMove> + '_ {
        self.moves.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{EdgeId, PositionId};

    fn sample_move(edge: u64, parent: u64, child: u64) -> RepertoireMove {
        RepertoireMove::new(
            EdgeId::new(edge),
            PositionId::new(parent),
            PositionId::new(child),
            format!("m{edge}"),
            format!("M{edge}"),
        )
    }

    #[test]
    fn graph_tracks_edges_by_parent_and_child() {
        let moves = vec![sample_move(1, 10, 11), sample_move(2, 10, 12)];
        let graph = OpeningGraph::from_moves(moves);
        let children: Vec<_> = graph
            .children(PositionId::new(10))
            .map(|mv| mv.child_id)
            .collect();
        assert_eq!(children, vec![PositionId::new(11), PositionId::new(12)]);
        let parents: Vec<_> = graph
            .parents(PositionId::new(12))
            .map(|mv| mv.parent_id)
            .collect();
        assert_eq!(parents, vec![PositionId::new(10)]);
    }

    #[test]
    fn graph_edge_lookup_returns_original_move() {
        let mv = sample_move(5, 20, 21);
        let graph = OpeningGraph::from_moves(vec![mv.clone()]);
        let fetched = graph.edge(mv.edge_id).expect("edge present");
        assert_eq!(fetched.move_uci, mv.move_uci);
    }
}
