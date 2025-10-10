//! Shared payloads for recording card reviews.

use chrono::NaiveDate;

/// Request payload for recording a review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRequest {
    /// Target card identifier.
    pub card_id: u64,
    /// Date of the review.
    pub reviewed_on: NaiveDate,
    /// Grade (0-4) awarded by the learner.
    pub grade: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_request_fields_are_public() {
        let request = ReviewRequest {
            card_id: 42,
            reviewed_on: NaiveDate::from_ymd_opt(2023, 1, 1).expect("valid date"),
            grade: 4,
        };
        assert_eq!(request.card_id, 42);
        assert_eq!(request.grade, 4);
    }
}
