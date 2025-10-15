use crate::{Card, CardKind};

/// Ordering function for candidate unlock cards.
/// Opening cards are prioritized first, ordered by their parent prefix (alphabetically)
/// and then by their UUID to ensure a stable order.
/// Tactic cards are ordered last, sorted by their UUID.
///
/// # Examples
/// ```
/// use uuid::Uuid;
/// use scheduler_core::store::candidate_ordering;
/// use scheduler_core::domain::{Card, CardKind, SchedulerOpeningCard, SchedulerTacticCard};
/// use std::cmp::Ordering;
///
/// let card_a = Card {
///     id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
///     owner_id: Uuid::new_v4(),
///     kind: CardKind::Opening(SchedulerOpeningCard::new("e4")),
///     state: Default::default(),
/// };
/// let card_b = Card {
///     id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
///     owner_id: Uuid::new_v4(),
///     kind: CardKind::Opening(SchedulerOpeningCard::new("d4")),
///     state: Default::default(),
/// };
/// let card_c = Card {
///     id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
///     owner_id: Uuid::new_v4(),
///     kind: CardKind::Tactic(SchedulerTacticCard::new()),
///     state: Default::default(),
/// };
///
/// assert_eq!(candidate_ordering(&card_a, &card_b), Ordering::Greater); // "e4" > "d4"
/// assert_eq!(candidate_ordering(&card_b, &card_a), Ordering::Less);
/// assert_eq!(candidate_ordering(&card_a, &card_c), Ordering::Less);
/// assert_eq!(candidate_ordering(&card_c, &card_a), Ordering::Greater);
/// assert_eq!(candidate_ordering(&card_b, &card_c), Ordering::Less);
/// assert_eq!(candidate_ordering(&card_c, &card_b), Ordering::Greater);
/// ```
#[must_use]
pub fn candidate_ordering(a: &Card, b: &Card) -> std::cmp::Ordering {
    match (&a.kind, &b.kind) {
        (CardKind::Opening(a_opening), CardKind::Opening(b_opening)) => {
            (&a_opening.parent_prefix, &a.id).cmp(&(&b_opening.parent_prefix, &b.id))
        }
        (CardKind::Opening(_), _) => std::cmp::Ordering::Less,
        (_, CardKind::Opening(_)) => std::cmp::Ordering::Greater,
        (CardKind::Tactic(_), CardKind::Tactic(_)) => a.id.cmp(&b.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::{Card, CardKind, SchedulerOpeningCard, SchedulerTacticCard, Sm2State};
    use std::cmp::Ordering;
    use uuid::Uuid;

    fn opening_card_with_prefix(prefix: &str, id: &str) -> Card {
        Card {
            id: Uuid::parse_str(id).unwrap(),
            owner_id: Uuid::new_v4(),
            kind: CardKind::Opening(SchedulerOpeningCard::new(prefix)),
            state: Sm2State::default(),
        }
    }

    fn tactic_card(id: &str) -> Card {
        Card {
            id: Uuid::parse_str(id).unwrap(),
            owner_id: Uuid::new_v4(),
            kind: CardKind::Tactic(SchedulerTacticCard::new()),
            state: Sm2State::default(),
        }
    }

    #[test]
    fn opening_vs_opening_prefix_ordering() {
        let card_a = opening_card_with_prefix("e4", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let card_b = opening_card_with_prefix("d4", "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb");
        assert_eq!(candidate_ordering(&card_a, &card_b), Ordering::Greater);
        assert_eq!(candidate_ordering(&card_b, &card_a), Ordering::Less);
    }

    #[test]
    fn opening_vs_opening_same_prefix_uuid_ordering() {
        let card_a = opening_card_with_prefix("e4", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let card_b = opening_card_with_prefix("e4", "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb");
        assert_eq!(candidate_ordering(&card_a, &card_b), Ordering::Less);
        assert_eq!(candidate_ordering(&card_b, &card_a), Ordering::Greater);
    }

    #[test]
    fn opening_vs_tactic() {
        let card_opening = opening_card_with_prefix("e4", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let card_tactic = tactic_card("cccccccc-cccc-cccc-cccc-cccccccccccc");
        assert_eq!(
            candidate_ordering(&card_opening, &card_tactic),
            Ordering::Less
        );
        assert_eq!(
            candidate_ordering(&card_tactic, &card_opening),
            Ordering::Greater
        );
    }

    #[test]
    fn tactic_vs_tactic_uuid_ordering() {
        let card_a = tactic_card("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let card_b = tactic_card("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb");
        assert_eq!(candidate_ordering(&card_a, &card_b), Ordering::Less);
        assert_eq!(candidate_ordering(&card_b, &card_a), Ordering::Greater);
    }

    #[test]
    fn opening_vs_opening_identical() {
        let card_a = opening_card_with_prefix("e4", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let card_b = opening_card_with_prefix("e4", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        assert_eq!(candidate_ordering(&card_a, &card_b), Ordering::Equal);
    }

    #[test]
    fn tactic_vs_tactic_identical() {
        let card_a = tactic_card("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let card_b = tactic_card("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        assert_eq!(candidate_ordering(&card_a, &card_b), Ordering::Equal);
    }
}
