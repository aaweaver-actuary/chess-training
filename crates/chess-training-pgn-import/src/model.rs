use fnv::FnvHasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

pub const SCHEMA_VERSION: u32 = 1;
pub const HASH_NAMESPACE: &str = "chess-training:pgn-import";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub id: u64,
    pub fen: String,
    pub side_to_move: char,
    pub ply: u32,
}

impl Position {
    pub fn new(fen: &str, side_to_move: char, ply: u32) -> Self {
        let id = hash_with_seed(HASH_NAMESPACE, SCHEMA_VERSION, &fen);
        Self {
            id,
            fen: fen.to_string(),
            side_to_move,
            ply,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub id: u64,
    pub parent_id: u64,
    pub move_uci: String,
    pub move_san: String,
    pub child_id: u64,
    pub source_hint: Option<String>,
}

impl Edge {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent_id: u64,
        move_uci: &str,
        move_san: &str,
        child_id: u64,
        source_hint: Option<String>,
    ) -> Self {
        let id = hash_with_seed(HASH_NAMESPACE, SCHEMA_VERSION, &(parent_id, move_uci));
        Self {
            id,
            parent_id,
            move_uci: move_uci.to_string(),
            move_san: move_san.to_string(),
            child_id,
            source_hint,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepertoireEdge {
    pub owner: String,
    pub repertoire_key: String,
    pub edge_id: u64,
}

impl RepertoireEdge {
    pub fn new(owner: &str, repertoire_key: &str, edge_id: u64) -> Self {
        Self {
            owner: owner.to_string(),
            repertoire_key: repertoire_key.to_string(),
            edge_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tactic {
    pub id: u64,
    pub fen: String,
    pub pv_uci: Vec<String>,
    pub tags: Vec<String>,
    pub source_hint: Option<String>,
}

impl Tactic {
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

fn hash_with_seed<T: Hash>(namespace: &str, schema_version: u32, value: &T) -> u64 {
    let mut hasher = FnvHasher::default();
    namespace.hash(&mut hasher);
    schema_version.hash(&mut hasher);
    value.hash(&mut hasher);
    hasher.finish()
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fen)
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
        let first = Edge::new(parent.id, "e2e4", "e4", child.id, None);
        let second = Edge::new(parent.id, "g1f3", "Nf3", child.id, None);
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn repertoire_edge_preserves_owner_and_key() {
        let record = RepertoireEdge::new("user", "rep", 42);
        assert_eq!(record.owner, "user");
        assert_eq!(record.repertoire_key, "rep");
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
        assert_eq!(payload["id"].as_u64(), Some(position.id));
    }
}
