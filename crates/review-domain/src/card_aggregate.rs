use chrono::NaiveDate;

use crate::{Card, CardKind, OpeningCard, StoredCardState, TacticCard, ValidGrade};

/// Concrete aggregate representing a learner's review card.
#[derive(Clone, Debug, PartialEq)]
pub struct CardAggregate {
    inner: Card<u64, u64, CardKind<OpeningCard, TacticCard>, StoredCardState>,
}

impl CardAggregate {
    /// Creates a new opening card aggregate.
    #[must_use]
    pub fn new_opening(card_id: u64, owner_id: u64, edge_id: u64, state: StoredCardState) -> Self {
        let kind = CardKind::Opening(OpeningCard::new(edge_id));
        Self::from_parts(card_id, owner_id, kind, state)
    }

    /// Creates a new tactic card aggregate.
    #[must_use]
    pub fn new_tactic(card_id: u64, owner_id: u64, tactic_id: u64, state: StoredCardState) -> Self {
        let kind = CardKind::Tactic(TacticCard::new(tactic_id));
        Self::from_parts(card_id, owner_id, kind, state)
    }

    fn from_parts(
        card_id: u64,
        owner_id: u64,
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
    pub fn owner_id(&self) -> u64 {
        self.inner.owner_id
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
    pub fn as_card(&self) -> &Card<u64, u64, CardKind<OpeningCard, TacticCard>, StoredCardState> {
        &self.inner
    }

    /// Consumes the aggregate and returns the underlying generic card.
    #[must_use]
    pub fn into_card(self) -> Card<u64, u64, CardKind<OpeningCard, TacticCard>, StoredCardState> {
        self.inner
    }

    /// Applies a review to the aggregate, updating the scheduling state.
    pub fn apply_review(&mut self, grade: ValidGrade, reviewed_on: NaiveDate) {
        self.inner.state.apply_review(grade, reviewed_on);
    }
}

impl From<Card<u64, u64, CardKind<OpeningCard, TacticCard>, StoredCardState>> for CardAggregate {
    fn from(card: Card<u64, u64, CardKind<OpeningCard, TacticCard>, StoredCardState>) -> Self {
        Self { inner: card }
    }
}

impl From<CardAggregate> for Card<u64, u64, CardKind<OpeningCard, TacticCard>, StoredCardState> {
    fn from(aggregate: CardAggregate) -> Self {
        aggregate.into_card()
    }
}
