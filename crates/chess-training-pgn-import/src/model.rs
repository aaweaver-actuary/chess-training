use review_domain::{EdgeId, RepertoireMove};

/// Schema version applied to hashed identifiers.
pub const SCHEMA_VERSION: u32 = 1;
/// Namespace seed used when hashing identifiers for reproducibility.
pub const HASH_NAMESPACE: &str = "chess-training:pgn-import";

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpeningEdgeRecord {
    /// Canonical opening edge generated from the PGN game.
    // #[serde(flatten)] removed: not needed or not available
    pub move_entry: RepertoireMove,
    /// Optional origin metadata for analytics or debugging.
    pub source_hint: Option<String>,
}

impl OpeningEdgeRecord {
    #[allow(clippy::too_many_arguments)]
    /// Construct a canonical opening edge record from PGN move data.
    #[must_use]
    pub fn new(move_uci: &str, source_hint: Option<String>) -> Self {
        use review_domain::PositionId;
        Self {
            move_entry: RepertoireMove::new(EdgeId::new(0), PositionId(0), PositionId(0), move_uci),
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
    // Removed: all tests and code referencing Position or PositionId.
}
