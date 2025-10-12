//! Review grades supported by the review domain.

/// Possible outcomes of a learner's review session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub enum ReviewGrade {
    /// The user failed to recall the item; schedule for immediate relearning.
    Again,
    /// The user recalled the item with difficulty; schedule for a shorter interval.
    Hard,
    /// The user recalled the item with reasonable ease; schedule for a normal interval.
    Good,
    /// The user recalled the item effortlessly; schedule for a longer interval.
    Easy,
}

#[cfg(test)]
mod tests {
    use super::ReviewGrade;

    #[test]
    fn grades_are_comparable() {
        assert_eq!(ReviewGrade::Again, ReviewGrade::Again);
        assert_ne!(ReviewGrade::Hard, ReviewGrade::Easy);
    }
}
