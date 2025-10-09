//! Generic representation of a review card shared across services.

/// A study card belonging to an owner and tracking custom state.
#[derive(Clone, Debug, PartialEq)]
pub struct Card<Id, Owner, Kind, State> {
    /// Stable identifier of the card.
    pub id: Id,
    /// Identifier of the owner/learner for the card.
    pub owner_id: Owner,
    /// Domain-specific classification of the card.
    pub kind: Kind,
    /// Mutable state associated with the card.
    pub state: State,
}
