use std::collections::BTreeMap;

use crate::ids::{EdgeId, PositionId};

use super::RepertoireMove;

/// Adjacency structure representing an opening repertoire as a directed graph.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpeningGraph {
    moves: Vec<RepertoireMove>,
    by_edge: BTreeMap<EdgeId, usize>,
    outgoing: BTreeMap<PositionId, Vec<usize>>,
    incoming: BTreeMap<PositionId, Vec<usize>>,
}

impl OpeningGraph {
    /// Creates an empty graph with no positions or edges.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::repertoire::OpeningGraph;
    /// let graph = OpeningGraph::new();
    /// assert!(graph.is_empty());
    /// assert_eq!(graph.len(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a graph from an owned collection of repertoire moves.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::ids::{EdgeId, PositionId};
    /// use review_domain::repertoire::{OpeningGraph, RepertoireMove};
    ///
    /// let (e1, e2) = (EdgeId::new(1), EdgeId::new(2));
    /// let (p1, p2, p3) = (PositionId::new(1), PositionId::new(2), PositionId::new(3));
    /// let moves = vec![
    ///     RepertoireMove::new(e1, p1, p2, "e2e4"),
    ///     RepertoireMove::new(e2, p1, p3, "d2d4"),
    /// ];
    /// let graph = OpeningGraph::from_moves(moves);
    /// assert_eq!(graph.len(), 2);
    /// assert_eq!(graph.children(p1).count(), 2);
    /// assert_eq!(graph.parents(p2).count(), 1);
    /// assert_eq!(graph.parents(p3).count(), 1);
    /// assert_eq!(graph.edge(e1).unwrap().move_uci, "e2e4");
    /// assert_eq!(graph.edge(e2).unwrap().move_uci, "d2d4");
    /// ```
    #[must_use]
    pub fn from_moves(moves: Vec<RepertoireMove>) -> Self {
        let mut graph = Self::default();
        for mv in moves {
            graph.add_move(mv);
        }
        graph
    }

    /// Inserts a new move into the graph, updating all adjacency indices.
    ////
    /// This does not enforce any business rules around duplicates or merging. It simply
    /// updates the internal state to reflect the new edge.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::ids::{EdgeId, PositionId};
    /// use review_domain::repertoire::{OpeningGraph, RepertoireMove};
    ///
    /// let mut graph = OpeningGraph::new();
    /// let edge = EdgeId::new(1);
    /// let old_psn = PositionId::new(10);
    /// let new_psn = PositionId::new(11);
    /// let mv = RepertoireMove::new(
    ///     edge,
    ///     old_psn,
    ///     new_psn,
    ///     "e2e4"
    /// );
    /// graph.add_move(mv.clone());
    /// assert_eq!(graph.len(), 1);
    /// assert_eq!(graph.children(PositionId::new(10)).next(), Some(&mv));
    /// assert_eq!(graph.parents(PositionId::new(11)).next(), Some(&mv));
    /// assert_eq!(graph.edge(EdgeId::new(1)), Some(&mv));
    /// ```
    pub fn add_move(&mut self, mv: RepertoireMove) {
        let index = self.moves.len();
        self.by_edge.insert(mv.edge_id, index);
        self.outgoing.entry(mv.parent_id).or_default().push(index);
        self.incoming.entry(mv.child_id).or_default().push(index);
        self.moves.push(mv);
    }

    /// Extends the graph by adding multiple moves from an iterator.
    /// This is equivalent to calling `add_move` for each item in the iterator.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::ids::{EdgeId, PositionId};
    /// use review_domain::repertoire::{OpeningGraph, RepertoireMove};
    ///
    /// let mut graph = OpeningGraph::new();
    /// let moves = vec![
    ///     RepertoireMove::new(EdgeId::new(1), PositionId::new(10), PositionId::new(11), "e2e4"),
    ///     RepertoireMove::new(EdgeId::new(2), PositionId::new(10), PositionId::new(12), "d2d4"),
    ///     RepertoireMove::new(EdgeId::new(3), PositionId::new(12), PositionId::new(13), "d7d5"),
    ///     RepertoireMove::new(EdgeId::new(4), PositionId::new(13), PositionId::new(14), "g1f3"),
    ///     RepertoireMove::new(EdgeId::new(5), PositionId::new(12), PositionId::new(15), "d7d5"),
    /// ];
    /// graph.extend(moves);
    ///
    /// assert_eq!(graph.len(), 5);
    /// assert_eq!(graph.children(PositionId::new(10)).count(), 2);
    /// assert_eq!(graph.children(PositionId::new(12)).count(), 2);
    /// assert_eq!(graph.parents(PositionId::new(11)).count(), 1);
    /// ```
    pub fn extend<I: IntoIterator<Item = RepertoireMove>>(&mut self, iter: I) {
        for mv in iter {
            self.add_move(mv);
        }
    }

    /// Returns the number of edges contained in the graph.
    ///
    /// # Examples
    /// ```
    /// use review_domain::{OpeningGraph, RepertoireMove, EdgeId, PositionId};
    ///
    /// let mut graph = OpeningGraph::new();
    /// assert_eq!(graph.len(), 0);
    ///
    /// let mv = RepertoireMove::new(
    ///     EdgeId::new(1),
    ///     PositionId::new(10),
    ///     PositionId::new(11),
    ///     "e2e4",
    /// );
    /// graph.add_move(mv);
    /// assert_eq!(graph.len(), 1);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Indicates whether the graph contains any edges.
    ///
    /// # Examples
    /// ```
    /// use review_domain::{OpeningGraph, RepertoireMove, EdgeId, PositionId};
    /// let mut graph = OpeningGraph::new();
    /// assert!(graph.is_empty());
    ///
    /// let mv = RepertoireMove::new(
    ///     EdgeId::new(1),
    ///     PositionId::new(10),
    ///     PositionId::new(11),
    ///     "e2e4",
    /// );
    /// graph.add_move(mv);
    /// assert!(!graph.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Provides immutable access to all repertoire moves backing the graph.
    ///
    /// # Examples
    /// ```
    /// use review_domain::{OpeningGraph, RepertoireMove, EdgeId, PositionId};
    /// let mut graph = OpeningGraph::new();
    /// assert!(graph.moves().is_empty());
    /// let mv1 = RepertoireMove::new(
    ///     EdgeId::new(1),
    ///     PositionId::new(10),
    ///     PositionId::new(11),
    ///     "e2e4",
    /// );
    /// let mv2 = RepertoireMove::new(
    ///     EdgeId::new(2),
    ///     PositionId::new(10),
    ///     PositionId::new(12),
    ///     "d2d4",
    /// );
    /// graph.add_move(mv1.clone());
    /// graph.add_move(mv2.clone());
    /// assert_eq!(graph.moves(), &[mv1, mv2]);
    /// ```
    #[must_use]
    pub fn moves(&self) -> &[RepertoireMove] {
        &self.moves
    }

    /// Returns an iterator over the moves that depart from the provided parent position.
    ///
    /// # Examples
    /// ```
    /// use review_domain::{OpeningGraph, RepertoireMove, EdgeId, PositionId};
    /// let mut graph = OpeningGraph::new();
    /// let mv1 = RepertoireMove::new(
    ///     EdgeId::new(1),
    ///     PositionId::new(10),
    ///     PositionId::new(11),
    ///     "e2e4",
    /// );
    /// let mv2 = RepertoireMove::new(
    ///     EdgeId::new(2),
    ///     PositionId::new(10),
    ///     PositionId::new(12),
    ///     "d2d4",
    /// );
    /// let mv2_a = RepertoireMove::new(
    ///     EdgeId::new(3),
    ///     PositionId::new(12),
    ///     PositionId::new(13),
    ///     "d7d5",
    /// );
    /// graph.add_move(mv1.clone());
    /// graph.add_move(mv2.clone());
    /// graph.add_move(mv2_a.clone());
    ///
    /// // This should only return the two moves that start from position 10, even though
    /// // there is a third move in the graph -- these are the DIRECT children of 10.
    /// assert_eq!(graph.children(PositionId::new(10)).count(), 2);
    /// ```
    pub fn children(&self, parent_id: PositionId) -> impl Iterator<Item = &RepertoireMove> + '_ {
        self.outgoing
            .get(&parent_id)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|&idx| &self.moves[idx]))
    }

    /// Returns an iterator over the moves that lead into the provided child position.
    pub fn parents(&self, child_id: PositionId) -> impl Iterator<Item = &RepertoireMove> + '_ {
        self.incoming
            .get(&child_id)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|&idx| &self.moves[idx]))
    }

    /// Finds a move by its edge identifier.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::{OpeningGraph, RepertoireMove, EdgeId, PositionId};
    /// let mut graph = OpeningGraph::new();
    /// let mv = RepertoireMove::new(
    ///     EdgeId::new(1),
    ///     PositionId::new(10),
    ///     PositionId::new(11),
    ///     "e2e4",
    /// );
    /// graph.add_move(mv.clone());
    /// assert_eq!(graph.edge(EdgeId::new(1)), Some(&mv));
    /// assert_eq!(graph.edge(EdgeId::new(2)), None);
    /// ```
    #[must_use]
    pub fn edge(&self, edge_id: EdgeId) -> Option<&RepertoireMove> {
        self.by_edge.get(&edge_id).map(|&idx| &self.moves[idx])
    }

    /// Iterates over all moves contained in the graph in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &RepertoireMove> + '_ {
        self.moves.iter()
    }

    /// Extracts a subgraph beginning from the specified position and including all
    /// descendant moves.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::{OpeningGraph, RepertoireMove, EdgeId, PositionId};
    /// let mut graph = OpeningGraph::new();
    /// let moves = vec![
    ///   RepertoireMove::new(EdgeId(1), PositionId(10), PositionId(11), "e2e4"),
    ///   RepertoireMove::new(EdgeId(2), PositionId(10), PositionId(12), "d2d4"),
    ///   RepertoireMove::new(EdgeId(3), PositionId(11), PositionId(13), "e7e5"),
    ///   RepertoireMove::new(EdgeId(4), PositionId(13), PositionId(14), "g1f3"),
    ///   RepertoireMove::new(EdgeId(5), PositionId(12), PositionId(15), "d7d5"),
    /// ];
    ///
    /// graph.extend(moves);
    /// let subgraph = graph.subgraph_from(PositionId(11));
    ///
    /// // Starting from position 11, we should only have two moves:
    /// // 11 -> 13 (e7e5) and 13 -> 14 (g1f3)
    /// assert_eq!(subgraph.len(), 2);
    ///
    /// // Check that the moves are indeed the expected ones
    /// let expected_moves = vec![
    ///     RepertoireMove::new(EdgeId(3), PositionId(11), PositionId(13), "e7e5"),
    ///     RepertoireMove::new(EdgeId(4), PositionId(13), PositionId(14), "g1f3"),
    /// ];
    /// assert_eq!(subgraph.moves(), &expected_moves);
    /// ```
    pub fn subgraph_from(&self, start: PositionId) -> Self {
        let mut visited = BTreeMap::new();
        let mut to_visit = vec![start];
        let mut subgraph = Self::new();

        while let Some(current) = to_visit.pop() {
            for mv in self.children(current) {
                if visited.insert(mv.edge_id, true).is_none() {
                    to_visit.push(mv.child_id);
                    subgraph.add_move(mv.clone());
                }
            }
        }

        subgraph
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
