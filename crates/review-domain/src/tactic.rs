//! Shared tactic-specific data structures.

/// Payload carried by tactic review cards.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TacticCard {
    /// Identifier of the reviewed tactic.
    pub tactic_id: u64,
}

impl TacticCard {
    /// Creates a new `TacticCard` payload.
    #[must_use]
    pub fn new(tactic_id: u64) -> Self {
        Self { tactic_id }
    }
}
