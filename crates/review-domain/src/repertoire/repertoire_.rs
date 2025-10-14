use std::iter::FromIterator;

use crate::ids::EdgeId;
use crate::{OpeningGraph, RepertoireError, RepertoireMove};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Aggregated store for the opening moves a student has committed to memory.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Repertoire {
    /// Friendly label describing the scope of the repertoire (e.g. "King's Indian Defence").
    name: String,
    /// Directed graph describing the repertoire's opening moves.
    graph: OpeningGraph,
}

impl Repertoire {
    /// Creates an empty repertoire with the provided descriptive name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph: OpeningGraph::new(),
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

    /// Provides the Avro schema for [`Repertoire`] when the `avro` feature is enabled.
    #[cfg(feature = "avro")]
    #[must_use]
    ///
    /// # Panics
    ///
    /// Panics if the hard-coded Avro schema definition fails to parse, indicating the
    /// embedded JSON is invalid.
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
                    self.graph
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

impl FromIterator<RepertoireMove> for Repertoire {
    /// Creates a `Repertoire` from an iterator of moves, using an empty string as the name.
    fn from_iter<I: IntoIterator<Item = RepertoireMove>>(iter: I) -> Self {
        Self {
            name: String::new(),
            graph: OpeningGraph::from_moves(iter.into_iter().collect()),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Repertoire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a> {
            name: &'a str,
            moves: &'a [RepertoireMove],
        }

        let helper = Helper {
            name: &self.name,
            moves: self.graph.moves(),
        };
        helper.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Repertoire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            moves: Vec<RepertoireMove>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            name: helper.name,
            graph: OpeningGraph::from_moves(helper.moves),
        })
    }
}

/// Builder for `Repertoire` to allow ergonomic construction with a custom name and moves.
#[derive(Default)]
pub struct RepertoireBuilder {
    name: String,
    moves: Vec<RepertoireMove>,
}

impl RepertoireBuilder {
    /// Create a new builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            moves: Vec::new(),
        }
    }

    /// Add a move to the repertoire.
    #[must_use]
    pub fn add_move(mut self, mv: RepertoireMove) -> Self {
        self.moves.push(mv);
        self
    }

    /// Add multiple moves from an iterator.
    #[must_use]
    pub fn extend<I: IntoIterator<Item = RepertoireMove>>(mut self, iter: I) -> Self {
        self.moves.extend(iter);
        self
    }

    /// Build the `Repertoire`.
    #[must_use]
    pub fn build(self) -> Repertoire {
        Repertoire {
            name: self.name,
            graph: OpeningGraph::from_moves(self.moves),
        }
    }
}
#[cfg(test)]
mod builder_and_iter_tests {
    use super::*;

    fn sample_move(n: u64) -> RepertoireMove {
        use crate::ids::{EdgeId, PositionId};

        RepertoireMove {
            parent_id: PositionId::new(n),
            child_id: PositionId::new(n + 1),
            edge_id: EdgeId::new(n * 10),
            move_uci: format!("e2e{n}"),
            move_san: format!("e{n}"),
        }
    }

    #[test]
    fn test_builder_single_move() {
        let rep = RepertoireBuilder::new("BuilderTest")
            .add_move(sample_move(1))
            .build();
        assert_eq!(rep.name(), "BuilderTest");
        assert_eq!(rep.moves().len(), 1);
        assert_eq!(rep.moves()[0].parent_id.get(), 1);
    }

    #[test]
    fn test_builder_multiple_moves() {
        let rep = RepertoireBuilder::new("Multi")
            .add_move(sample_move(1))
            .add_move(sample_move(2))
            .build();
        assert_eq!(rep.moves().len(), 2);
        assert_eq!(rep.moves()[1].parent_id.get(), 2);
    }

    #[test]
    fn test_builder_extend() {
        let moves = (1..=3).map(sample_move);
        let rep = RepertoireBuilder::new("Extend").extend(moves).build();
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
        let rep = RepertoireBuilder::new("Graph")
            .add_move(sample_move(1))
            .add_move(sample_move(2))
            .build();
        let children: Vec<_> = rep
            .graph()
            .children(crate::ids::PositionId::new(1))
            .map(|mv| mv.child_id)
            .collect();
        assert_eq!(children, vec![crate::ids::PositionId::new(2)]);

        let parents: Vec<_> = rep
            .graph()
            .parents(crate::ids::PositionId::new(3))
            .map(|mv| mv.parent_id)
            .collect();
        assert_eq!(parents, vec![crate::ids::PositionId::new(2)]);
    }
}

#[cfg(test)]
mod coverage_minimal {
    use super::*;

    #[test]
    fn covers_new_constructor() {
        let rep = Repertoire::new("Coverage");
        assert_eq!(rep.name(), "Coverage");
        assert!(rep.moves().is_empty());
    }
}
#[cfg(all(test, feature = "avro"))]
mod avro_tests {
    use super::*;
    use apache_avro::schema::Schema;
    use apache_avro::types::Value;

    #[test]
    fn test_avro_schema_json_is_valid() {
        let schema = Repertoire::avro_schema();
        assert!(matches!(schema, Schema::Record { .. }));
    }

    #[test]
    fn test_to_avro_value_matches_schema() {
        let rep = RepertoireBuilder::new("AvroTest")
            .add_move(crate::RepertoireMove {
                parent_id: crate::ids::PositionId::new(1),
                child_id: crate::ids::PositionId::new(2),
                edge_id: crate::ids::EdgeId::new(3),
                move_uci: "e2e4".to_string(),
                move_san: "e4".to_string(),
            })
            .build();
        let value = rep.to_avro_value();
        assert!(matches!(value, Value::Record(_)));
    }

    #[test]
    fn test_avro_schema_json_constant() {
        // Just check the constant is valid JSON
        let json: serde_json::Value = serde_json::from_str(Repertoire::AVRO_SCHEMA_JSON).unwrap();
        assert!(json.is_object());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ids::*, RepertoireError, RepertoireMove};

    fn sample_move() -> RepertoireMove {
        RepertoireMove {
            parent_id: PositionId::new(100),
            child_id: PositionId::new(101),
            edge_id: EdgeId::new(1),
            move_uci: "e2e4".to_string(),
            move_san: "e4".to_string(),
        }
    }

    #[test]
    fn test_new_and_name() {
        let rep = Repertoire::new("French Defence");
        assert_eq!(rep.name(), "French Defence");
        assert!(rep.moves().is_empty());
    }

    #[test]
    fn test_moves_accessor() {
        let empty = Repertoire::new("Test");
        assert_eq!(empty.moves().len(), 0);

        let mv = sample_move();
        let rep = RepertoireBuilder::new("Test").add_move(mv.clone()).build();
        assert_eq!(rep.moves().len(), 1);
        assert_eq!(rep.moves()[0], mv);
    }

    #[test]
    fn test_add_move_stub() {
        let mut rep = Repertoire::new("Test");
        let mv = sample_move();
        let err = rep.add_move(mv).unwrap_err();
        let RepertoireError::NotImplemented { operation } = err;
        assert_eq!(operation, "add_move");
    }

    #[test]
    fn test_remove_move_stub() {
        let mut rep = Repertoire::new("Test");
        let err = rep.remove_move(EdgeId::new(42)).unwrap_err();
        let RepertoireError::NotImplemented { operation } = err;
        assert_eq!(operation, "remove_move");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let rep = RepertoireBuilder::new("SerDe")
            .add_move(sample_move())
            .build();
        let json = serde_json::to_string(&rep).expect("serialize");
        let de: Repertoire = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(rep, de);
    }

    #[cfg(feature = "avro")]
    #[test]
    fn test_avro_schema_and_value() {
        let rep = Repertoire::new("Avro");
        let schema = Repertoire::avro_schema();
        let value = rep.to_avro_value();
        // Just check types and that schema parses
        assert!(matches!(schema, apache_avro::schema::Schema::Record { .. }));
        assert!(matches!(value, apache_avro::types::Value::Record(_)));
    }
}
