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
    #[must_use]
    pub fn map_opening<O2>(self, mapper: impl FnOnce(Opening) -> O2) -> CardKind<O2, Tactic> {
        match self {
            CardKind::Opening(opening) => CardKind::Opening(mapper(opening)),
            CardKind::Tactic(tactic) => CardKind::Tactic(tactic),
        }
    }

    /// Maps the tactic payload to a different type while leaving the opening payload untouched.
    #[must_use]
    pub fn map_tactic<T2>(self, mapper: impl FnOnce(Tactic) -> T2) -> CardKind<Opening, T2> {
        match self {
            CardKind::Opening(opening) => CardKind::Opening(opening),
            CardKind::Tactic(tactic) => CardKind::Tactic(mapper(tactic)),
        }
    }

    /// Returns references to the payload without moving the value.
    #[must_use]
    pub fn as_ref(&self) -> CardKind<&Opening, &Tactic> {
        match self {
            CardKind::Opening(opening) => CardKind::Opening(opening),
            CardKind::Tactic(tactic) => CardKind::Tactic(tactic),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn map_opening_transforms_opening_variant() {
        let card: CardKind<String, ()> = CardKind::Opening("line".to_string());
        let mapped: CardKind<usize, _> = card.map_opening(|opening| opening.len());
        assert!(matches!(mapped, CardKind::Opening(4)));
    }

    #[test]
    fn map_opening_leaves_tactic_variant_untouched() {
        let card: CardKind<&str, _> = CardKind::Tactic("fork");
        let mapped = card.map_opening(|opening| opening.len());
        assert!(matches!(mapped, CardKind::Tactic("fork")));
    }

    #[test]
    fn map_tactic_transforms_tactic_variant() {
        let card: CardKind<(), String> = CardKind::Tactic("pin".to_string());
        let mapped: CardKind<(), usize> = card.map_tactic(|tactic| tactic.len());
        assert!(matches!(mapped, CardKind::Tactic(3)));
    }

    #[test]
    fn map_tactic_leaves_opening_variant_untouched() {
        let card: CardKind<_, &str> = CardKind::Opening("Najdorf");
        let mapped = card.map_tactic(|tactic| tactic.len());
        assert!(matches!(mapped, CardKind::Opening("Najdorf")));
    }

    #[test]
    fn as_ref_preserves_payload_references() {
        let tactic = String::from("skewer");
        let card: CardKind<(), String> = CardKind::Tactic(tactic.clone());
        match card.as_ref() {
            CardKind::Tactic(reference) => assert_eq!(*reference, "skewer"),
            CardKind::Opening(_) => panic!("expected tactic variant"),
        }
        let opening: CardKind<String, ()> = CardKind::Opening(String::from("Ruy Lopez"));
        match opening.as_ref() {
            CardKind::Opening(reference) => assert_eq!(*reference, "Ruy Lopez"),
            CardKind::Tactic(_) => panic!("expected opening variant"),
        }
    }
}
