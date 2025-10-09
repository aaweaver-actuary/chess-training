use review_domain::CardKind as GenericCardKind;

/// Payload describing an opening-based card within the scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchedulerOpeningCard {
    pub parent_prefix: String,
}

impl SchedulerOpeningCard {
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
    pub fn new() -> Self {
        Self
    }
}

/// Represents the type of a card in the scheduler.
pub type CardKind = GenericCardKind<SchedulerOpeningCard, SchedulerTacticCard>;
