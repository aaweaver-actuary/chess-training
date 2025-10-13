#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{CardKind, OpeningCard, StoredCardState, TacticCard, hash64};

const OPENING_CARD_PREFIX: &[u8] = b"review-domain::card::opening";
const TACTIC_CARD_PREFIX: &[u8] = b"review-domain::card::tactic";

/// Concrete card representation pairing deterministic identifiers with validated state.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CardAggregate {
    /// Stable identifier derived from the owner and underlying payload.
    pub id: u64,
    /// Identifier of the learner that owns the card.
    pub owner_id: String,
    /// Domain-specific payload describing the review target.
    pub kind: CardKind<OpeningCard, TacticCard>,
    /// Scheduling state tracked for the card.
    pub state: StoredCardState,
}

/// Errors surfaced when constructing a [`CardAggregate`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CardAggregateError {
    /// Returned when the provided [`StoredCardState`] already records a review timestamp.
    #[error("card state must not include a last_reviewed_on value when creating an aggregate")]
    LastReviewedOnPresent,
    /// Returned when the provided [`StoredCardState`] starts with a non-zero streak.
    #[error("card state must start with zero consecutive correct reviews, found {0}")]
    NonZeroConsecutiveCorrect(u32),
}

impl CardAggregate {
    /// Builds an opening card aggregate for the provided owner and edge identifiers.
    ///
    /// # Errors
    ///
    /// Returns [`CardAggregateError::LastReviewedOnPresent`] if the initial state already
    /// records a previous review date, or
    /// [`CardAggregateError::NonZeroConsecutiveCorrect`] if the streak is not zero.
    pub fn new_opening(
        owner_id: impl Into<String>,
        edge_id: u64,
        state: StoredCardState,
    ) -> Result<Self, CardAggregateError> {
        validate_initial_state(&state)?;
        let owner_id = owner_id.into();
        let id = aggregate_identifier(OPENING_CARD_PREFIX, &owner_id, edge_id);
        Ok(Self {
            id,
            owner_id,
            kind: CardKind::Opening(OpeningCard::new(edge_id)),
            state,
        })
    }

    /// Builds a tactic card aggregate for the provided owner and tactic identifiers.
    ///
    /// # Errors
    ///
    /// Returns [`CardAggregateError::LastReviewedOnPresent`] if the initial state already
    /// records a previous review date, or
    /// [`CardAggregateError::NonZeroConsecutiveCorrect`] if the streak is not zero.
    pub fn new_tactic(
        owner_id: impl Into<String>,
        tactic_id: u64,
        state: StoredCardState,
    ) -> Result<Self, CardAggregateError> {
        validate_initial_state(&state)?;
        let owner_id = owner_id.into();
        let id = aggregate_identifier(TACTIC_CARD_PREFIX, &owner_id, tactic_id);
        Ok(Self {
            id,
            owner_id,
            kind: CardKind::Tactic(TacticCard::new(tactic_id)),
            state,
        })
    }
}

fn aggregate_identifier(prefix: &[u8], owner_id: &str, payload_id: u64) -> u64 {
    hash64(&[prefix, owner_id.as_bytes(), &payload_id.to_be_bytes()])
}

fn validate_initial_state(state: &StoredCardState) -> Result<(), CardAggregateError> {
    if state.last_reviewed_on.is_some() {
        return Err(CardAggregateError::LastReviewedOnPresent);
    }
    if state.consecutive_correct != 0 {
        return Err(CardAggregateError::NonZeroConsecutiveCorrect(
            state.consecutive_correct,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::num::NonZeroU8;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_state() -> StoredCardState {
        let interval = NonZeroU8::new(1).expect("non-zero interval");
        StoredCardState::new(naive_date(2024, 1, 1), interval, 2.5)
    }

    #[test]
    fn new_opening_builds_expected_aggregate() {
        let state = sample_state();
        let aggregate = CardAggregate::new_opening("learner", 42, state.clone())
            .expect("opening aggregate should be created");
        let expected_id = aggregate_identifier(OPENING_CARD_PREFIX, "learner", 42);
        assert_eq!(aggregate.id, expected_id);
        assert_eq!(aggregate.owner_id, "learner");
        assert_eq!(aggregate.kind, CardKind::Opening(OpeningCard::new(42)));
        assert_eq!(aggregate.state, state);
    }

    #[test]
    fn new_tactic_builds_expected_aggregate() {
        let state = sample_state();
        let aggregate = CardAggregate::new_tactic("learner", 7, state.clone())
            .expect("tactic aggregate should be created");
        let expected_id = aggregate_identifier(TACTIC_CARD_PREFIX, "learner", 7);
        assert_eq!(aggregate.id, expected_id);
        assert_eq!(aggregate.owner_id, "learner");
        assert_eq!(aggregate.kind, CardKind::Tactic(TacticCard::new(7)));
        assert_eq!(aggregate.state, state);
    }

    #[test]
    fn new_opening_rejects_state_with_last_reviewed_on() {
        let mut state = sample_state();
        state.last_reviewed_on = Some(naive_date(2023, 12, 31));
        let error = CardAggregate::new_opening("owner", 99, state)
            .expect_err("creation should fail when last_reviewed_on is set");
        assert_eq!(error, CardAggregateError::LastReviewedOnPresent);
    }

    #[test]
    fn new_tactic_rejects_state_with_consecutive_correct() {
        let mut state = sample_state();
        state.consecutive_correct = 3;
        let error = CardAggregate::new_tactic("owner", 11, state)
            .expect_err("creation should fail when streak is non-zero");
        assert_eq!(error, CardAggregateError::NonZeroConsecutiveCorrect(3));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn card_aggregate_round_trips_through_serde() {
        let state = sample_state();
        let aggregate =
            CardAggregate::new_opening("owner", 5, state).expect("aggregate should build");
        let json = serde_json::to_string(&aggregate).expect("serialize");
        let decoded: CardAggregate = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, aggregate);
    }
}
