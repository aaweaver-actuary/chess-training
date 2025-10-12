//! Canonical representation of stored opening repertoire moves.

pub mod move_;
pub mod opening_graph;
pub mod repertoire_;
pub mod repertoire_error;

pub use move_::RepertoireMove;
pub use opening_graph::{OpeningGraph, OpeningGraphBuildError, OpeningGraphBuilder};
pub use repertoire_::Repertoire;
pub use repertoire_error::RepertoireError;

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
