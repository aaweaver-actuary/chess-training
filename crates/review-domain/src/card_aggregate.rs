use chrono::NaiveDate;

use crate::{
    Card, CardKind, EdgeId, GradeError, OpeningCard, ReviewRequest, StoredCardState, TacticCard,
    ValidGrade,
};

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
        edge_id: impl Into<EdgeId>,
        state: StoredCardState,
    ) -> Self {
        let kind = CardKind::Opening(OpeningCard::new(edge_id.into()));
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

impl From<CardAggregate> for StoredReviewCard {
    fn from(aggregate: CardAggregate) -> Self {
        aggregate.into_card()
    }
}
