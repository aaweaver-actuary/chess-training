use std::collections::{BTreeMap, BTreeSet};

use crate::model::{OpeningEdgeRecord, Position, RepertoireEdge, Tactic};

/// Trait for abstracting storage of chess training data, such as positions, edges, repertoire edges, and tactics.
///
/// The `upsert_*` methods insert or update the given item in the storage.
///
/// # Return value semantics
/// Each `upsert_*` method returns an [`UpsertOutcome`] describing whether the item was newly inserted or replaced an existing
/// item.
///
/// # Expected behavior
/// Implementors should ensure that the storage is updated with the provided item, and that the return value accurately reflects
/// whether the item was newly added or replaced an existing entry.
pub trait Storage {
    fn upsert_position(&mut self, position: Position) -> UpsertOutcome;
    fn upsert_edge(&mut self, edge: OpeningEdgeRecord) -> UpsertOutcome;
    fn upsert_repertoire_edge(&mut self, record: RepertoireEdge) -> UpsertOutcome;
    fn upsert_tactic(&mut self, tactic: Tactic) -> UpsertOutcome;
}

#[must_use]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Indicates whether an upsert operation inserted a new record or replaced an existing one.
pub enum UpsertOutcome {
    /// The record did not previously exist in storage and was inserted.
    Inserted,
    /// The record replaced an existing entry in storage.
    Replaced,
}

impl UpsertOutcome {
    #[must_use]
    pub const fn is_inserted(self) -> bool {
        matches!(self, Self::Inserted)
    }

    pub const fn from_bool(inserted: bool) -> Self {
        if inserted {
            Self::Inserted
        } else {
            Self::Replaced
        }
    }
}

#[derive(Default)]
/// An in-memory implementation of the `Storage` trait, primarily used for testing purposes.
pub struct ImportInMemoryStore {
    positions: BTreeMap<u64, Position>,
    edges: BTreeMap<u64, OpeningEdgeRecord>,
    repertoire_edges: BTreeSet<(String, String, u64)>,
    tactics: BTreeMap<u64, Tactic>,
}

impl Storage for ImportInMemoryStore {
    fn upsert_position(&mut self, position: Position) -> UpsertOutcome {
        UpsertOutcome::from_bool(self.positions.insert(position.id, position).is_none())
    }

    fn upsert_edge(&mut self, edge: OpeningEdgeRecord) -> UpsertOutcome {
        UpsertOutcome::from_bool(self.edges.insert(edge.edge.id, edge).is_none())
    }

    fn upsert_repertoire_edge(&mut self, record: RepertoireEdge) -> UpsertOutcome {
        UpsertOutcome::from_bool(
            self.repertoire_edges
                .insert((record.owner, record.repertoire_key, record.edge_id)),
        )
    }

    fn upsert_tactic(&mut self, tactic: Tactic) -> UpsertOutcome {
        UpsertOutcome::from_bool(self.tactics.insert(tactic.id, tactic).is_none())
    }
}

impl ImportInMemoryStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn positions(&self) -> Vec<Position> {
        self.positions.values().cloned().collect()
    }

    #[must_use]
    pub fn edges(&self) -> Vec<OpeningEdgeRecord> {
        self.edges.values().cloned().collect()
    }

    #[must_use]
    pub fn tactics(&self) -> Vec<Tactic> {
        self.tactics.values().cloned().collect()
    }

    #[must_use]
    pub fn repertoire_edges(&self) -> Vec<RepertoireEdge> {
        self.repertoire_edges
            .iter()
            .map(|(owner, repertoire_key, edge_id)| {
                RepertoireEdge::new(owner, repertoire_key, *edge_id)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Position as ModelPosition;

    fn sample_position(index: u32) -> Position {
        ModelPosition::new(&format!("fen {index}"), 'w', index)
    }

    #[test]
    fn upsert_methods_report_insert_status() {
        let mut store = ImportInMemoryStore::default();
        let parent = sample_position(0);
        let child = sample_position(1);
        let edge = OpeningEdgeRecord::new(parent.id, "e2e4", "e4", child.id, None);
        let record = RepertoireEdge::new("owner", "rep", edge.edge.id);
        let tactic = Tactic::new("fen", vec!["e2e4".into()], vec![], None);

        assert!(store.upsert_position(parent.clone()).is_inserted());
        assert!(!store.upsert_position(parent).is_inserted());
        assert!(store.upsert_edge(edge.clone()).is_inserted());
        assert!(!store.upsert_edge(edge.clone()).is_inserted());
        assert!(store.upsert_repertoire_edge(record.clone()).is_inserted());
        assert!(!store.upsert_repertoire_edge(record).is_inserted());
        assert!(store.upsert_tactic(tactic.clone()).is_inserted());
        assert!(!store.upsert_tactic(tactic).is_inserted());
    }

    #[test]
    fn repertoire_edges_accessor_round_trips_entries() {
        let mut store = ImportInMemoryStore::default();
        let parent = sample_position(0);
        let child = sample_position(1);
        let edge = OpeningEdgeRecord::new(parent.id, "e2e4", "e4", child.id, None);
        assert!(store.upsert_edge(edge.clone()).is_inserted());
        assert!(store
            .upsert_repertoire_edge(RepertoireEdge::new("owner", "rep", edge.edge.id))
            .is_inserted());

        let records = store.repertoire_edges();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].owner, "owner");
        assert_eq!(records[0].repertoire_key, "rep");
        assert_eq!(records[0].edge_id, edge.edge.id);
    }

    #[test]
    fn in_memory_store_default_is_the_same_as_new() {
        let default_store = ImportInMemoryStore::default();
        let new_store = ImportInMemoryStore::new();
        assert_eq!(default_store.positions.len(), new_store.positions.len());
        assert_eq!(default_store.edges.len(), new_store.edges.len());
        assert_eq!(
            default_store.repertoire_edges.len(),
            new_store.repertoire_edges.len()
        );
        assert_eq!(default_store.tactics.len(), new_store.tactics.len());
    }
}
