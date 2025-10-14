use derive_builder::Builder;
use std::iter::FromIterator;

use crate::ids::EdgeId;
use crate::{OpeningGraph, RepertoireError, RepertoireMove};

/// Aggregated store for the opening moves a student has committed to memory.
#[derive(Clone, Debug, PartialEq, Eq, Default, Builder)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Repertoire {
    /// Friendly label describing the scope of the repertoire (e.g. "King's Indian Defence").
    pub name: String,
    /// Directed graph describing the repertoire's opening moves.
    pub graph: OpeningGraph,
}

impl Repertoire {
    /// Creates an empty repertoire with the provided descriptive name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            graph: OpeningGraph::new(),
        }
    }

    /// Creates a new builder for constructing a [`Repertoire`].
    #[must_use]
    pub fn builder(name: &str) -> RepertoireBuilder {
        RepertoireBuilder {
            name: Some(name.to_string()),
            graph: Some(OpeningGraph::new()),
        }
    }

    /// Human readable label associated with the repertoire.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Immutable view of all moves currently tracked by the repertoire.
    #[must_use]
    pub fn moves(&self) -> &[RepertoireMove] {
        self.graph.moves()
    }

    /// Graph representation of the repertoire for adjacency queries.
    #[must_use]
    pub fn graph(&self) -> &OpeningGraph {
        &self.graph
    }

    /// Placeholder stub for inserting a move into the repertoire.
    ///
    /// The implementation will later enforce business rules around duplicates and merge
    /// policies. For now it communicates intent through the returned error value.
    ///
    /// # Errors
    ///
    /// Always returns [`RepertoireError::NotImplemented`] until the insertion logic is
    /// implemented.
    pub fn add_move(&mut self, _move_entry: RepertoireMove) -> Result<(), RepertoireError> {
        Err(RepertoireError::not_implemented("add_move"))
    }

    /// Placeholder stub for removing a move from the repertoire by its edge identifier.
    ///
    /// Future implementations will prune the internal store and return success if the move is
    /// found. The current stub advertises the missing functionality to consumers.
    ///
    /// # Errors
    ///
    /// Always returns [`RepertoireError::NotImplemented`] until the removal logic is
    /// implemented.
    pub fn remove_move(&mut self, _edge_id: EdgeId) -> Result<(), RepertoireError> {
        Err(RepertoireError::not_implemented("remove_move"))
    }
}

impl FromIterator<RepertoireMove> for Repertoire {
    /// Creates a `Repertoire` from an iterator of moves, using an empty string as the name.
    fn from_iter<I: IntoIterator<Item = RepertoireMove>>(iter: I) -> Self {
        Self {
            name: String::new(),
            graph: OpeningGraph::from_moves(iter.into_iter().collect()),
        }
    }
}

impl RepertoireBuilder {
    fn add_move_without_checking(&mut self, mv: RepertoireMove) -> &mut Self {
        self.graph
            .as_mut()
            .expect("graph should be initialized either in the constructor or directly above this")
            .add_move(mv);
        self
    }

    fn initialize_graph_if_needed(&mut self) {
        if self.graph.is_none() {
            self.graph = Some(OpeningGraph::new());
        }
    }

    /// Add a move to the repertoire.
    /// If the graph is not initialized, it will be created.
    ///
    /// # Examples
    /// ```rust
    /// use review_domain::ids::{EdgeId, PositionId};
    /// use review_domain::repertoire::{RepertoireBuilder, RepertoireMove};
    pub fn add(&mut self, mv: RepertoireMove) -> &mut Self {
        self.initialize_graph_if_needed();
        self.add_move_without_checking(mv);
        self
    }

    /// Add multiple moves from an iterator.
    pub fn extend<I: IntoIterator<Item = RepertoireMove>>(&mut self, iter: I) -> &mut Self {
        self.initialize_graph_if_needed();
        iter.into_iter().for_each(|mv| {
            self.add_move_without_checking(mv);
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::PositionId;

    use super::*;

    #[test]
    fn covers_new_constructor() {
        let rep = Repertoire::new("Coverage");
        assert_eq!(rep.name(), "Coverage");
        assert!(rep.moves().is_empty());
    }

    fn sample_move(n: u64) -> RepertoireMove {
        RepertoireMove {
            parent_id: PositionId::new(n),
            child_id: PositionId::new(n + 1),
            edge_id: EdgeId::new(n * 10),
            move_uci: format!("e2e{n}"),
        }
    }

    #[test]
    fn test_builder_single_move() {
        let rep = Repertoire::builder("BuilderTest")
            .add(sample_move(1))
            .build()
            .unwrap();
        assert_eq!(rep.name(), "BuilderTest");
        assert_eq!(rep.moves().len(), 1);
        assert_eq!(rep.moves()[0].parent_id.get(), 1);
    }

    #[test]
    fn test_builder_multiple_moves() {
        let rep = Repertoire::builder("Multi")
            .add(sample_move(1))
            .add(sample_move(2))
            .build()
            .unwrap();
        assert_eq!(rep.moves().len(), 2);
        assert_eq!(rep.moves()[1].parent_id.get(), 2);
    }

    #[test]
    fn test_builder_extend() {
        let moves = (1..=3).map(sample_move);
        let rep = Repertoire::builder("Extend").extend(moves).build().unwrap();
        assert_eq!(rep.moves().len(), 3);
        assert_eq!(rep.moves()[2].parent_id.get(), 3);
    }

    #[test]
    fn test_from_iterator() {
        let moves: Vec<_> = (10..13).map(sample_move).collect();
        let rep: Repertoire = moves.clone().into_iter().collect();
        assert_eq!(rep.moves().len(), 3);
        assert_eq!(rep.moves()[0].parent_id.get(), 10);
        assert_eq!(rep.name(), "");
    }

    #[test]
    fn test_graph_children_and_parents() {
        let mut rep = Repertoire::builder("Graph");
        let rep_result = rep.add(sample_move(1)).add(sample_move(2));

        let rr2 = rep_result.clone();

        let children: Vec<_> = rep_result
            .clone()
            .build()
            .unwrap()
            .graph()
            .children(PositionId::new(1))
            .map(|mv| mv.child_id)
            .collect();
        assert_eq!(children, vec![PositionId::new(2)]);

        let parents: Vec<_> = rr2
            .build()
            .unwrap()
            .graph()
            .parents(PositionId::new(3))
            .map(|mv| mv.parent_id)
            .collect();
        assert_eq!(parents, vec![PositionId::new(2)]);
    }
}
