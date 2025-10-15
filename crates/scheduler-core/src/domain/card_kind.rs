use review_domain::CardKind as GenericCardKind;
use std::hash::Hash;

/// Payload describing an opening-based card within the scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchedulerOpeningCard {
    /// Identifier prefix tying the card back to its parent opening line.
    pub parent_prefix: String,
}

impl SchedulerOpeningCard {
    /// Constructs an opening card payload for the provided parent prefix.
    ///
    /// # Examples
    /// ```rust
    /// use scheduler_core::SchedulerOpeningCard;
    /// let card = SchedulerOpeningCard::new("e4-e5-Nf3");
    /// assert_eq!(card.parent_prefix, "e4-e5-Nf3");
    /// ```
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

    #[test]
    fn scheduler_opening_card_new_sets_parent_prefix() {
        let prefix = "e4-e5-Nf3".to_string();
        let card = SchedulerOpeningCard::new(prefix.clone());
        assert_eq!(card.parent_prefix, prefix);

        // Test with &str input
        let card2 = SchedulerOpeningCard::new("d4-d5-c4");
        assert_eq!(card2.parent_prefix, "d4-d5-c4");
    }

    #[test]
    fn scheduler_opening_card_equality_and_hash() {
        let card1 = SchedulerOpeningCard::new("foo");
        let card2 = SchedulerOpeningCard::new("foo");
        let card3 = SchedulerOpeningCard::new("bar");

        assert_eq!(card1, card2);
        assert_ne!(card1, card3);

        let mut hasher1 = DefaultHasher::new();
        card1.hash(&mut hasher1);
        let mut hasher2 = DefaultHasher::new();
        card2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());

        let mut hasher3 = DefaultHasher::new();
        card3.hash(&mut hasher3);
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn scheduler_opening_card_debug_format() {
        let card = SchedulerOpeningCard::new("test");
        let debug_str = format!("{card:?}");
        assert!(debug_str.contains("SchedulerOpeningCard"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn scheduler_tactic_card_new_is_default_and_equality() {
        let card1 = SchedulerTacticCard::new();
        #[allow(clippy::default_constructed_unit_structs)]
        let card2 = SchedulerTacticCard::default();
        assert_eq!(card1, card2);

        // Copy and clone
        let card3 = card1;
        #[allow(clippy::clone_on_copy)]
        let card4 = card3.clone();
        assert_eq!(card3, card4);

        // Hash
        let mut hasher1 = DefaultHasher::new();
        card1.hash(&mut hasher1);
        let mut hasher2 = DefaultHasher::new();
        card2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn scheduler_tactic_card_debug_format() {
        let card = SchedulerTacticCard::new();
        let debug_str = format!("{card:?}");
        assert!(debug_str.contains("SchedulerTacticCard"));
    }

    // CardKind is a type alias; test that it can be constructed and compared
    #[test]
    fn card_kind_type_alias_usage() {
        // Assuming CardKind is an enum with variants Opening and Tactic
        // and that it derives PartialEq, Debug, etc.

        let opening = SchedulerOpeningCard::new("foo");
        let tactic = SchedulerTacticCard::new();

        let card_opening: CardKind = GenericCardKind::Opening(opening.clone());
        let card_tactic: CardKind = GenericCardKind::Tactic(tactic);

        // Test Debug
        let dbg_opening = format!("{card_opening:?}");
        let dbg_tactic = format!("{card_tactic:?}");
        assert!(dbg_opening.contains("Opening"));
        assert!(dbg_tactic.contains("Tactic"));

        // Test PartialEq
        let card_opening2: CardKind = GenericCardKind::Opening(opening);
        assert_eq!(card_opening, card_opening2);
        assert_ne!(card_opening, card_tactic);
    }
}
