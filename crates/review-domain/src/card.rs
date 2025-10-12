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

#[cfg(test)]
mod tests {
    use super::Card;

    #[derive(Clone, Debug, PartialEq)]
    struct Owner(u64);

    #[derive(Clone, Debug, PartialEq)]
    enum CardKind {
        Tactics,
        Strategy,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct CardState {
        ease: f32,
        interval_days: u32,
        lapses: u32,
    }

    impl CardState {
        fn new(ease: f32, interval_days: u32, lapses: u32) -> Self {
            Self {
                ease,
                interval_days,
                lapses,
            }
        }
    }

    #[test]
    fn card_fields_are_stored() {
        let card = Card {
            id: "card-123".to_string(),
            owner_id: Owner(42),
            kind: CardKind::Tactics,
            state: CardState::new(2.3, 7, 0),
        };

        assert_eq!(card.id, "card-123");
        assert_eq!(card.owner_id, Owner(42));
        assert_eq!(card.kind, CardKind::Tactics);
        assert_eq!(card.state, CardState::new(2.3, 7, 0));
    }

    #[test]
    fn cards_with_identical_fields_are_equal() {
        let card_a = Card {
            id: 1_u32,
            owner_id: Owner(3),
            kind: CardKind::Strategy,
            state: CardState::new(1.7, 3, 1),
        };
        let card_b = Card {
            id: 1_u32,
            owner_id: Owner(3),
            kind: CardKind::Strategy,
            state: CardState::new(1.7, 3, 1),
        };

        assert_eq!(card_a, card_b);
    }

    #[test]
    fn card_clone_produces_distinct_but_equal_instance() {
        let original = Card {
            id: [0_u8; 16],
            owner_id: Owner(7),
            kind: CardKind::Tactics,
            state: CardState::new(2.1, 14, 2),
        };

        let clone = original.clone();

        assert_eq!(original, clone);
        let original_ptr = std::ptr::from_ref(&original);
        let clone_ptr = std::ptr::from_ref(&clone);

        assert!(std::ptr::eq(original_ptr, original_ptr));
        assert!(!std::ptr::eq(original_ptr, clone_ptr));
    }

    #[test]
    fn card_allows_mutating_state_through_public_field() {
        let mut card = Card {
            id: 9_i64,
            owner_id: Owner(100),
            kind: CardKind::Strategy,
            state: CardState::new(2.5, 10, 0),
        };

        card.state.ease = 2.8;
        card.state.interval_days += 5;
        card.state.lapses += 1;

        assert!((card.state.ease - 2.8).abs() < f32::EPSILON);
        assert_eq!(card.state.interval_days, 15);
        assert_eq!(card.state.lapses, 1);
    }
}
