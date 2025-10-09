use std::collections::HashMap;

use chrono::NaiveDate;

use crate::model::{Card, CardKind, CardState, Edge, OpeningCard, ReviewRequest};
use crate::store::StoreError;

pub(super) fn store_opening_card(
    cards: &mut HashMap<u64, Card>,
    owner_id: &str,
    edge: &Edge,
    state: CardState,
    card_id: u64,
) -> Result<Card, StoreError> {
    match cards.entry(card_id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
            validate_existing_opening_card(entry.get(), owner_id, edge)?;
            Ok(entry.get().clone())
        }
        std::collections::hash_map::Entry::Vacant(slot) => {
            let card = build_opening_card(owner_id, edge, state, card_id);
            slot.insert(card.clone());
            Ok(card)
        }
    }
}

pub(super) fn collect_due_cards_for_owner(
    cards: &HashMap<u64, Card>,
    owner_id: &str,
    as_of: NaiveDate,
) -> Vec<Card> {
    let mut result: Vec<Card> = cards
        .values()
        .filter(|card| card.owner_id == owner_id && card.state.due_on <= as_of)
        .cloned()
        .collect();
    result.sort_by_key(|card| (card.state.due_on, card.id));
    result
}

pub(super) fn borrow_card_for_review<'a>(
    cards: &'a mut HashMap<u64, Card>,
    review: &ReviewRequest,
) -> Result<&'a mut Card, StoreError> {
    cards
        .get_mut(&review.card_id)
        .ok_or(StoreError::MissingCard { id: review.card_id })
}

fn validate_existing_opening_card(
    card: &Card,
    owner_id: &str,
    edge: &Edge,
) -> Result<(), StoreError> {
    if card.owner_id == owner_id
        && matches!(
            card.kind,
            CardKind::Opening(ref opening) if opening.edge_id == edge.id
        )
    {
        Ok(())
    } else {
        Err(StoreError::HashCollision { entity: "card" })
    }
}

fn build_opening_card(owner_id: &str, edge: &Edge, state: CardState, card_id: u64) -> Card {
    Card {
        id: card_id,
        owner_id: owner_id.to_string(),
        kind: CardKind::Opening(OpeningCard { edge_id: edge.id }),
        state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::num::NonZeroU8;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_card_state(due_on: NaiveDate) -> CardState {
        CardState::new(due_on, NonZeroU8::new(1).unwrap(), 2.5)
    }

    fn sample_edge(id: u64) -> Edge {
        Edge {
            id,
            parent_id: 1,
            child_id: 2,
            move_uci: "e2e4".into(),
            move_san: "e4".into(),
        }
    }

    #[test]
    fn store_opening_card_reuses_existing_matching_entry() {
        let mut cards = HashMap::new();
        let edge = sample_edge(5);
        let card = build_opening_card(
            "owner",
            &edge,
            sample_card_state(naive_date(2023, 1, 1)),
            10,
        );
        cards.insert(card.id, card.clone());

        let result = store_opening_card(
            &mut cards,
            "owner",
            &edge,
            sample_card_state(naive_date(2023, 1, 2)),
            card.id,
        )
        .expect("existing card should be returned");
        assert_eq!(result, card);
    }

    #[test]
    fn store_opening_card_errors_on_hash_collision() {
        let mut cards = HashMap::new();
        let edge = sample_edge(5);
        let mut different_owner = build_opening_card(
            "someone_else",
            &edge,
            sample_card_state(naive_date(2023, 1, 1)),
            10,
        );
        different_owner.kind = CardKind::Opening(OpeningCard { edge_id: 99 });
        cards.insert(10, different_owner);

        let err = store_opening_card(
            &mut cards,
            "owner",
            &edge,
            sample_card_state(naive_date(2023, 1, 2)),
            10,
        )
        .unwrap_err();
        assert!(matches!(err, StoreError::HashCollision { entity } if entity == "card"));
    }

    #[test]
    fn collect_due_cards_for_owner_sorts_by_due_date_then_id() {
        let mut cards = HashMap::new();
        let edge_one = sample_edge(1);
        let edge_two = sample_edge(2);
        cards.insert(
            1,
            build_opening_card(
                "owner",
                &edge_one,
                sample_card_state(naive_date(2023, 1, 3)),
                1,
            ),
        );
        cards.insert(
            2,
            build_opening_card(
                "owner",
                &edge_two,
                sample_card_state(naive_date(2023, 1, 2)),
                2,
            ),
        );

        let due = collect_due_cards_for_owner(&cards, "owner", naive_date(2023, 1, 3));
        assert_eq!(
            due.iter().map(|card| card.id).collect::<Vec<_>>(),
            vec![2, 1]
        );
    }

    #[test]
    fn borrow_card_for_review_returns_mutable_reference() {
        let mut cards = HashMap::new();
        let edge = sample_edge(3);
        let card = build_opening_card("owner", &edge, sample_card_state(naive_date(2023, 1, 1)), 7);
        cards.insert(card.id, card.clone());
        let review = ReviewRequest {
            card_id: card.id,
            reviewed_on: naive_date(2023, 1, 1),
            grade: 2,
        };

        let borrowed = borrow_card_for_review(&mut cards, &review).expect("card exists");
        borrowed.state.due_on = naive_date(2023, 1, 5);
        assert_eq!(
            cards.get(&card.id).unwrap().state.due_on,
            naive_date(2023, 1, 5)
        );
    }

    #[test]
    fn borrow_card_for_review_errors_when_missing() {
        let mut cards = HashMap::new();
        let review = ReviewRequest {
            card_id: 999,
            reviewed_on: naive_date(2023, 1, 1),
            grade: 2,
        };
        let err = borrow_card_for_review(&mut cards, &review).unwrap_err();
        assert!(matches!(err, StoreError::MissingCard { id } if id == 999));
    }
}
