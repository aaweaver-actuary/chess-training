use review_domain::CardKind as GenericCardKind;

/// Payload describing an opening-based card within the scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchedulerOpeningCard {
    /// Identifier prefix tying the card back to its parent opening line.
    pub parent_prefix: String,
}

impl SchedulerOpeningCard {
    /// Constructs an opening card payload for the provided parent prefix.
    #[must_use]
    pub fn new(parent_prefix: impl Into<String>) -> Self {
        Self {
            parent_prefix: parent_prefix.into(),
        }
    }
}

/// Marker struct representing tactic cards. Kept as a struct to allow future metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SchedulerTacticCard;

impl SchedulerTacticCard {
    /// Constructs a tactic card payload.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Represents the type of a card in the scheduler.
pub type CardKind = GenericCardKind<SchedulerOpeningCard, SchedulerTacticCard>;
