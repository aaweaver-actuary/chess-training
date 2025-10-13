use chrono::NaiveDate;

use crate::{Card, CardKind, EdgeId, OpeningCard, StoredCardState, TacticCard, ValidGrade};

type StoredReviewCard = Card<u64, String, CardKind<OpeningCard, TacticCard>, StoredCardState>;

/// Concrete aggregate representing a learner's review card.
#[derive(Clone, Debug, PartialEq)]
pub struct CardAggregate {
    inner: StoredReviewCard,
}

impl CardAggregate {
    /// Creates a new opening card aggregate.
    #[must_use]
    pub fn new_opening(
        card_id: u64,
        owner_id: impl Into<String>,
        edge_id: EdgeId,
        state: StoredCardState,
    ) -> Self {
        let kind = CardKind::Opening(OpeningCard::new(edge_id));
        Self::from_parts(card_id, owner_id.into(), kind, state)
    }

    /// Creates a new tactic card aggregate.
    #[must_use]
    pub fn new_tactic(
        card_id: u64,
        owner_id: impl Into<String>,
        tactic_id: u64,
        state: StoredCardState,
    ) -> Self {
        let kind = CardKind::Tactic(TacticCard::new(tactic_id));
        Self::from_parts(card_id, owner_id.into(), kind, state)
    }

    fn from_parts(
        card_id: u64,
        owner_id: String,
        kind: CardKind<OpeningCard, TacticCard>,
        state: StoredCardState,
    ) -> Self {
        Self {
            inner: Card {
                id: card_id,
                owner_id,
                kind,
                state,
            },
        }
    }

    /// Returns the identifier of the card.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.inner.id
    }

    /// Returns the owner identifier of the card.
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.inner.owner_id
    }

    /// Returns the card kind payload.
    #[must_use]
    pub fn kind(&self) -> &CardKind<OpeningCard, TacticCard> {
        &self.inner.kind
    }

    /// Returns the scheduling state for inspection.
    #[must_use]
    pub fn state(&self) -> &StoredCardState {
        &self.inner.state
    }

    /// Returns a reference to the underlying generic card representation.
    #[must_use]
    pub fn as_card(&self) -> &StoredReviewCard {
        &self.inner
    }

    /// Consumes the aggregate and returns the underlying generic card.
    #[must_use]
    pub fn into_card(self) -> StoredReviewCard {
        self.inner
    }

    /// Applies a validated grade to the aggregate, updating the scheduling state.
    pub fn apply_valid_grade(&mut self, grade: ValidGrade, reviewed_on: NaiveDate) {
        self.inner.state.apply_review(grade, reviewed_on);
    }

impl From<CardAggregate> for StoredReviewCard {
    fn from(aggregate: CardAggregate) -> Self {
        aggregate.into_card()
    }
}

use chrono::NaiveDate;

use crate::{CardKind, GradeError, ReviewRequest, StoredCardState, ValidGrade};

/// Concrete card aggregate tying together identifiers, payload, and state.
#[derive(Clone, Debug, PartialEq)]
pub struct CardAggregate<Id, Owner, Opening, Tactic> {
    /// Stable identifier of the card aggregate.
    pub id: Id,
    /// Identifier for the learner that owns the card.
    pub owner_id: Owner,
    /// Domain specific payload describing what the card reviews.
    pub kind: CardKind<Opening, Tactic>,
    /// Mutable scheduling state for the card.
    pub state: StoredCardState,
}

impl<Id, Owner, Opening, Tactic> CardAggregate<Id, Owner, Opening, Tactic> {
    /// Apply a review to the aggregate, mutating the stored state.
    ///
    /// # Errors
    ///
    /// Returns a [`GradeError`] when the provided grade falls outside the
    /// supported spaced repetition scale.
    pub fn apply_review(&mut self, grade: u8, reviewed_on: NaiveDate) -> Result<(), GradeError> {
        let grade = ValidGrade::new(grade)?;
        self.apply_valid_grade(grade, reviewed_on);
        Ok(())
    }

    /// Applies the supplied [`ReviewRequest`] to the aggregate.
    ///
    /// # Errors
    ///
    /// Returns a [`GradeError`] when the embedded grade falls outside the
    /// supported spaced repetition scale.
    pub fn apply_review_request(&mut self, review: &ReviewRequest) -> Result<(), GradeError> {
        self.apply_review(review.grade, review.reviewed_on)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeId, OpeningCard, ReviewRequest, TacticCard};
    use chrono::NaiveDate;
    use std::num::NonZeroU8;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_state() -> StoredCardState {
        StoredCardState::new(naive_date(2023, 1, 1), NonZeroU8::new(2).unwrap(), 2.5)
    }

    fn sample_opening_card() -> CardAggregate<u64, String, OpeningCard, TacticCard> {
        CardAggregate {
            id: 1,
            owner_id: String::from("owner"),
            kind: CardKind::Opening(OpeningCard::new(EdgeId::new(7))),
            state: sample_state(),
        }
    }

    #[test]
    fn apply_review_updates_underlying_state() {
        let mut aggregate = sample_opening_card();
        let reviewed_on = naive_date(2023, 1, 5);

        aggregate
            .apply_review(4, reviewed_on)
            .expect("grade should be accepted");

        assert_eq!(aggregate.state.last_reviewed_on, Some(reviewed_on));
        assert_eq!(aggregate.state.due_on, naive_date(2023, 1, 9));
        assert_eq!(aggregate.state.interval.get(), 4);
    }

    #[test]
    fn apply_review_rejects_invalid_grade() {
        let mut aggregate = sample_opening_card();
        let original_state = aggregate.state.clone();
        let reviewed_on = naive_date(2023, 1, 5);

        let error = aggregate
            .apply_review(9, reviewed_on)
            .expect_err("grade should be rejected");

        assert_eq!(error, GradeError::GradeOutsideRangeError { grade: 9 });
        assert_eq!(aggregate.state, original_state);
    }
}

impl From<CardAggregate> for StoredReviewCard {
    fn from(aggregate: CardAggregate) -> Self {
        aggregate.into_card()
    }
}
