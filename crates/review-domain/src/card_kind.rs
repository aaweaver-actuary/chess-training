//! Generic flashcard classification helpers shared across services.

/// Describes the high-level type of a study card.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CardKind<Opening, Tactic> {
    /// Card reviewing an opening concept.
    Opening(Opening),
    /// Card reviewing a tactic.
    Tactic(Tactic),
}

impl<Opening, Tactic> CardKind<Opening, Tactic> {
    /// Maps the opening payload to a different type while leaving the tactic payload untouched.
    pub fn map_opening<O2>(self, mapper: impl FnOnce(Opening) -> O2) -> CardKind<O2, Tactic> {
        match self {
            CardKind::Opening(opening) => CardKind::Opening(mapper(opening)),
            CardKind::Tactic(tactic) => CardKind::Tactic(tactic),
        }
    }

    /// Maps the tactic payload to a different type while leaving the opening payload untouched.
    pub fn map_tactic<T2>(self, mapper: impl FnOnce(Tactic) -> T2) -> CardKind<Opening, T2> {
        match self {
            CardKind::Opening(opening) => CardKind::Opening(opening),
            CardKind::Tactic(tactic) => CardKind::Tactic(mapper(tactic)),
        }
    }

    /// Returns references to the payload without moving the value.
    pub fn as_ref(&self) -> CardKind<&Opening, &Tactic> {
        match self {
            CardKind::Opening(opening) => CardKind::Opening(opening),
            CardKind::Tactic(tactic) => CardKind::Tactic(tactic),
        }
    }
}
