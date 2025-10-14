use fnv::FnvHasher;
use review_domain::{EdgeId, PositionId, RepertoireMove};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Schema version applied to hashed identifiers.
pub const SCHEMA_VERSION: u32 = 1;
/// Namespace seed used when hashing identifiers for reproducibility.
pub const HASH_NAMESPACE: &str = "chess-training:pgn-import";

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpeningEdgeRecord {
    /// Canonical opening edge generated from the PGN game.
    #[serde(flatten)]
    pub move_entry: RepertoireMove,
    /// Optional origin metadata for analytics or debugging.
    pub source_hint: Option<String>,
}

impl OpeningEdgeRecord {
    #[allow(clippy::too_many_arguments)]
    /// Construct a canonical opening edge record from PGN move data.
    #[must_use]
    pub fn new(
        parent_id: PositionId,
        move_uci: &str,
        child_id: PositionId,
        source_hint: Option<String>,
    ) -> Self {
        let id = hash_with_seed(HASH_NAMESPACE, SCHEMA_VERSION, &(parent_id, move_uci));
        let move_entry = RepertoireMove::new(EdgeId::new(id), parent_id, child_id, move_uci);
        Self {
            move_entry,
            source_hint,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RepertoireEdge {
    /// Owner identifier for the repertoire.
    pub owner: String,
    /// Logical grouping key for the repertoire.
    pub repertoire_key: String,
    /// Identifier of the edge stored in the repertoire.
    pub edge_id: EdgeId,
}

impl RepertoireEdge {
    /// Construct a repertoire edge linking an owner, repertoire key, and opening edge.
    #[must_use]
    pub fn new(owner: &str, repertoire_key: &str, edge_id: EdgeId) -> Self {
        Self {
            owner: owner.to_string(),
            repertoire_key: repertoire_key.to_string(),
            edge_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tactic {
    /// Stable identifier derived from the FEN and principal variation.
    pub id: u64,
    /// FEN string describing the tactic's starting position.
    pub fen: String,
    /// Principal variation encoded as UCI moves.
    pub pv_uci: Vec<String>,
    /// Optional tags applied to the tactic.
    pub tags: Vec<String>,
    /// Optional hint describing the tactic's provenance.
    pub source_hint: Option<String>,
}

impl Tactic {
    /// Construct a tactic entry with deterministic identifier based on the FEN and PV.
    #[must_use]
    pub fn new(
        fen: &str,
        pv_uci: Vec<String>,
        tags: Vec<String>,
        source_hint: Option<String>,
    ) -> Self {
        let id = hash_with_seed(HASH_NAMESPACE, SCHEMA_VERSION, &(fen, pv_uci.as_slice()));
        Self {
            id,
            fen: fen.to_string(),
            pv_uci,
            tags,
            source_hint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn position_ids_differ_for_different_fens() {
        let a = Position::new("fen one", 'w', 0);
        let b = Position::new("fen two", 'w', 0);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn display_returns_fen_string() {
        let position = Position::new("fen one", 'w', 0);
        assert_eq!(position.to_string(), "fen one");
    }

    #[test]
    fn edge_ids_differ_for_unique_moves() {
        let parent = Position::new("fen parent", 'w', 0);
        let child = Position::new("fen child", 'b', 1);
        let first = OpeningEdgeRecord::new(parent.id, "e2e4", child.id, None);
        let second = OpeningEdgeRecord::new(parent.id, "g1f3", child.id, None);
        assert_ne!(first.move_entry.edge_id, second.move_entry.edge_id);
    }

    #[test]
    fn repertoire_edge_preserves_owner_and_key() {
        let record = RepertoireEdge::new("user", "rep", EdgeId::new(42));
        assert_eq!(record.owner, "user");
        assert_eq!(record.repertoire_key, "rep");
        assert_eq!(record.edge_id.get(), 42);
    }

    #[test]
    fn tactic_ids_depend_on_pv() {
        let base = Tactic::new("fen", vec!["e2e4".into(), "e7e5".into()], vec![], None);
        let alt = Tactic::new("fen", vec!["d2d4".into()], vec![], None);
        assert_ne!(base.id, alt.id);
    }

    #[test]
    fn position_serializes_with_expected_fields() {
        let position = Position::new("fen serial", 'w', 0);
        let payload: Value = serde_json::to_value(&position).unwrap();
        assert_eq!(payload["fen"], "fen serial");
        assert_eq!(payload["id"].as_u64(), Some(position.id.get()));
    }
}
