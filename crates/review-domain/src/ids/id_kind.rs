use std::fmt;

/// Identifies which strongly typed identifier failed to convert.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdKind {
    /// Identifier for stored chess positions.
    Position,
    /// Identifier for directed edges between positions.
    Edge,
    /// Identifier for individual moves inside an opening tree.
    Move,
    /// Identifier for persisted review cards.
    Card,
    /// Identifier for a learner using the training platform.
    Learner,
    /// Identifier for unlock records associated with learners.
    Unlock,
    /// Identifier for tactics in the tactics training system.
    Tactic,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Position => "position",
            Self::Edge => "edge",
            Self::Move => "move",
            Self::Card => "card",
            Self::Learner => "learner",
            Self::Unlock => "unlock",
            Self::Tactic => "tactic",
        };
        f.write_str(label)
    }
}
