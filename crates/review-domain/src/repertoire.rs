//! Canonical representation of stored opening repertoire moves.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Domain error produced when manipulating a [`Repertoire`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepertoireError {
    /// Placeholder error returned by not-yet-implemented operations.
    #[error("repertoire operation '{operation}' is not implemented yet")]
    NotImplemented { operation: &'static str },
}

impl RepertoireError {
    /// Creates a [`RepertoireError::NotImplemented`] for the provided operation.
    #[must_use]
    pub const fn not_implemented(operation: &'static str) -> Self {
        Self::NotImplemented { operation }
    }
}

/// A single move stored within an opening repertoire.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RepertoireMove {
    /// Identifier of the originating position.
    pub parent_id: u64,
    /// Identifier of the resulting position.
    pub child_id: u64,
    /// Deterministic identifier of the represented opening edge.
    pub edge_id: u64,
    /// Move encoded in UCI notation.
    pub move_uci: String,
    /// Move encoded in SAN notation.
    pub move_san: String,
}

impl RepertoireMove {
    /// Builds a new [`RepertoireMove`] from the constituent identifiers and move notation.
    #[must_use]
    pub fn new(
        edge_id: u64,
        parent_id: u64,
        child_id: u64,
        move_uci: impl Into<String>,
        move_san: impl Into<String>,
    ) -> Self {
        Self {
            edge_id,
            parent_id,
            child_id,
            move_uci: move_uci.into(),
            move_san: move_san.into(),
        }
    }
}

/// Aggregated store for the opening moves a student has committed to memory.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Repertoire {
    /// Friendly label describing the scope of the repertoire (e.g. "King's Indian Defence").
    name: String,
    /// Collection of moves that make up the repertoire.
    moves: Vec<RepertoireMove>,
}

impl Repertoire {
    /// Creates an empty repertoire with the provided descriptive name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            moves: Vec::new(),
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
        &self.moves
    }

    /// Placeholder stub for inserting a move into the repertoire.
    ///
    /// The implementation will later enforce business rules around duplicates and merge
    /// policies. For now it communicates intent through the returned error value.
    pub fn add_move(&mut self, _move_entry: RepertoireMove) -> Result<(), RepertoireError> {
        Err(RepertoireError::not_implemented("add_move"))
    }

    /// Placeholder stub for removing a move from the repertoire by its edge identifier.
    ///
    /// Future implementations will prune the internal store and return success if the move is
    /// found. The current stub advertises the missing functionality to consumers.
    pub fn remove_move(&mut self, _edge_id: u64) -> Result<(), RepertoireError> {
        Err(RepertoireError::not_implemented("remove_move"))
    }

    /// Provides the Avro schema for [`Repertoire`] when the `avro` feature is enabled.
    #[cfg(feature = "avro")]
    #[must_use]
    pub fn avro_schema() -> apache_avro::schema::Schema {
        apache_avro::schema::Schema::parse_str(Self::AVRO_SCHEMA_JSON)
            .expect("repertoire schema is valid")
    }

    /// Converts the repertoire into an Avro [`Value`](apache_avro::types::Value).
    #[cfg(feature = "avro")]
    #[must_use]
    pub fn to_avro_value(&self) -> apache_avro::types::Value {
        use apache_avro::types::Value;

        Value::Record(vec![
            ("name".into(), Value::String(self.name.clone())),
            (
                "moves".into(),
                Value::Array(
                    self.moves
                        .iter()
                        .map(RepertoireMove::to_avro_value)
                        .collect(),
                ),
            ),
        ])
    }

    #[cfg(feature = "avro")]
    const AVRO_SCHEMA_JSON: &'static str = r#"{
        "type": "record",
        "name": "Repertoire",
        "fields": [
            {"name": "name", "type": "string"},
            {"name": "moves", "type": {"type": "array", "items": {
                "type": "record",
                "name": "RepertoireMove",
                "fields": [
                    {"name": "parent_id", "type": "string"},
                    {"name": "child_id", "type": "string"},
                    {"name": "edge_id", "type": "string"},
                    {"name": "move_uci", "type": "string"},
                    {"name": "move_san", "type": "string"}
                ]
            }}}
        ]
    }"#;
}

#[cfg(feature = "avro")]
impl RepertoireMove {
    fn to_avro_value(&self) -> apache_avro::types::Value {
        use apache_avro::types::Value;

        Value::Record(vec![
            (
                "parent_id".into(),
                Value::String(self.parent_id.to_string()),
            ),
            ("child_id".into(), Value::String(self.child_id.to_string())),
            ("edge_id".into(), Value::String(self.edge_id.to_string())),
            ("move_uci".into(), Value::String(self.move_uci.clone())),
            ("move_san".into(), Value::String(self.move_san.clone())),
        ])
    }
}
