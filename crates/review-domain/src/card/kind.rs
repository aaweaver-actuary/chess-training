//! Generic flashcard classification helpers shared across services.

use std::fmt;

/// Describes the high-level type of a study card.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl fmt::Display for CardKind<&str, &str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardKind::Opening(name) => write!(f, "Opening: {name}"),
            CardKind::Tactic(name) => write!(f, "Tactic: {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_opening_transforms_opening_variant() {
        let card: CardKind<&str, &str> = CardKind::Opening("line");
        let mapped: CardKind<usize, _> = card.map_opening(str::len);
        assert_eq!(mapped, CardKind::Opening(4));
    }

    #[test]
    fn map_opening_leaves_tactic_variant_untouched() {
        let card: CardKind<&str, &str> = CardKind::Tactic("fork");
        let mapped = card.map_opening(str::len);
        assert_eq!(mapped, CardKind::Tactic("fork"));
    }

    #[test]
    fn map_tactic_transforms_tactic_variant() {
        let card: CardKind<&str, &str> = CardKind::Tactic("pin");
        let mapped: CardKind<&str, usize> = card.map_tactic(str::len);
        assert_eq!(mapped, CardKind::Tactic(3));
    }

    #[test]
    fn map_tactic_leaves_opening_variant_untouched() {
        let card: CardKind<&str, &str> = CardKind::Opening("Najdorf");
        let mapped = card.map_tactic(str::len);
        assert_eq!(mapped, CardKind::Opening("Najdorf"));
    }

    #[test]
    fn as_ref_preserves_payload_references() {
        let tactic_payload = String::from("skewer");
        let tactic_card: CardKind<String, String> = CardKind::Tactic(tactic_payload.clone());
        assert_eq!(tactic_card.as_ref(), CardKind::Tactic(&tactic_payload));

        let opening_payload = String::from("Ruy Lopez");
        let opening_card: CardKind<String, String> = CardKind::Opening(opening_payload.clone());
        assert_eq!(opening_card.as_ref(), CardKind::Opening(&opening_payload));
    }

    #[test]
    fn to_string_formats_opening_variant() {
        let card: CardKind<&str, &str> = CardKind::Opening("Sicilian Defense");
        assert_eq!(card.to_string(), "Opening: Sicilian Defense");
    }

    #[test]
    fn to_string_formats_tactic_variant() {
        let card: CardKind<&str, &str> = CardKind::Tactic("Fork");
        assert_eq!(card.to_string(), "Tactic: Fork");
    }
}
