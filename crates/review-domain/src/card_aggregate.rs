//! Aggregate representation of a review card with scheduling state.

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
    /// Returns a [`GradeError`] when the grade falls outside the supported
    /// spaced repetition scale.
    pub fn apply_review(&mut self, grade: u8, reviewed_on: NaiveDate) -> Result<(), GradeError> {
        let grade = ValidGrade::new(grade)?;
        self.state.apply_review(grade, reviewed_on);
        Ok(())
    }

    /// Apply a [`ReviewRequest`] to the aggregate by delegating to [`Self::apply_review`].
    ///
    /// # Errors
    ///
    /// Returns a [`GradeError`] when the grade embedded in the request falls
    /// outside the supported spaced repetition scale.
    pub fn apply_review_request(&mut self, review: &ReviewRequest) -> Result<(), GradeError> {
        self.apply_review(review.grade, review.reviewed_on)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OpeningCard, ReviewRequest, TacticCard};
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
            kind: CardKind::Opening(OpeningCard::new(7)),
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

    #[test]
    fn apply_review_request_delegates_to_helper() {
        let mut aggregate = sample_opening_card();
        let reviewed_on = naive_date(2023, 1, 5);
        let review = ReviewRequest {
            card_id: aggregate.id,
            reviewed_on,
            grade: 4,
        };

        aggregate
            .apply_review_request(&review)
            .expect("grade should be accepted");

        assert_eq!(aggregate.state.last_reviewed_on, Some(reviewed_on));
        assert_eq!(aggregate.state.due_on, naive_date(2023, 1, 9));
        assert_eq!(aggregate.state.interval.get(), 4);
    }
}
