use crate::model::{ReviewRequest, StoredCardState};
use crate::store::StoreError;
use review_domain::{GradeError, ValidGrade};

pub(super) fn apply_review(
    state: &mut StoredCardState,
    review: &ReviewRequest,
) -> Result<(), StoreError> {
    let grade = ValidGrade::new(review.grade)
        .map_err(|error: review_domain::GradeError| map_grade_error(&error))?;
    state.apply_review(grade, review.reviewed_on);
    Ok(())
}

fn map_grade_error(error: &GradeError) -> StoreError {
    let grade = match error {
        GradeError::GradeOutsideRangeError { grade } | GradeError::InvalidGradeError { grade } => {
            grade
        }
    };
    StoreError::InvalidGrade { grade: *grade }
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
        StoredCardState::new(naive_date(2023, 1, 1), NonZeroU8::new(2).unwrap(), 2.5)
    }

    fn sample_review(grade: u8) -> ReviewRequest {
        ReviewRequest {
            card_id: 1,
            reviewed_on: naive_date(2023, 1, 5),
            grade,
        }
    }

    #[test]
    fn apply_review_mutates_state_via_domain_logic() {
        let mut state = sample_state();
        let review = sample_review(4);
        apply_review(&mut state, &review).expect("valid review");
        assert_eq!(state.interval.get(), 4);
        assert_eq!(state.due_on, naive_date(2023, 1, 9));
        assert_eq!(state.last_reviewed_on, Some(review.reviewed_on));
    }

    #[test]
    fn apply_review_returns_store_error_for_invalid_grade() {
        let mut state = sample_state();
        let review = sample_review(9);
        let err = apply_review(&mut state, &review).unwrap_err();
        assert_eq!(err, StoreError::InvalidGrade { grade: 9 });
    }

    #[test]
    fn map_grade_error_handles_invalid_grade_variant() {
        let err = map_grade_error(&GradeError::InvalidGradeError { grade: 7 });
        assert_eq!(err, StoreError::InvalidGrade { grade: 7 });
    }
}
