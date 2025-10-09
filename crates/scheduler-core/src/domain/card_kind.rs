#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents the type of a card in the scheduler.
///
/// - `Opening`: A card representing an opening position or concept, typically associated with a parent entity.
/// - `Tactic`: A card representing a tactical motif or problem, not associated with a parent.
pub enum CardKind {
    /// An opening card, associated with a parent entity.
    ///
    /// `parent_prefix` identifies the parent (e.g., a specific opening line or group) to which this card belongs.
    Opening { parent_prefix: String },
    /// A tactic card, representing a standalone tactical motif or problem.
    Tactic,
}
