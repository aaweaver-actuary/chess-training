use super::StudyStage;

/// Returns true if the card is in the New stage.
#[must_use]
pub fn is_new(stage: StudyStage) -> bool {
    matches!(stage, StudyStage::New)
}

/// Returns true if the card is in the Learning stage.
#[must_use]
pub fn is_learning(stage: StudyStage) -> bool {
    matches!(stage, StudyStage::Learning)
}

/// Returns true if the card is in the Review stage.
#[must_use]
pub fn is_review(stage: StudyStage) -> bool {
    matches!(stage, StudyStage::Review)
}

/// Returns true if the card is in the Relearning stage.
#[must_use]
pub fn is_relearning(stage: StudyStage) -> bool {
    matches!(stage, StudyStage::Relearning)
}

/// Returns true if the card is in any of the active stages (Learning, Review, Relearning).
#[must_use]
pub fn is_active(stage: StudyStage) -> bool {
    matches!(
        stage,
        StudyStage::Learning | StudyStage::Review | StudyStage::Relearning
    )
}

impl StudyStage {
    /// Returns true if the card is in the New stage.
    #[must_use]
    pub fn is_new(self) -> bool {
        is_new(self)
    }

    /// Returns true if the card is in the Learning stage.
    #[must_use]
    pub fn is_learning(self) -> bool {
        is_learning(self)
    }

    /// Returns true if the card is in the Review stage.
    #[must_use]
    pub fn is_review(self) -> bool {
        is_review(self)
    }

    /// Returns true if the card is in the Relearning stage.
    #[must_use]
    pub fn is_relearning(self) -> bool {
        is_relearning(self)
    }

    /// Returns true if the card is in any of the active stages (Learning, Review, Relearning).
    #[must_use]
    pub fn is_active(self) -> bool {
        is_active(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_helpers_cover_all_variants() {
        assert!(is_new(StudyStage::New));
        assert!(!is_new(StudyStage::Learning));
        assert!(is_learning(StudyStage::Learning));
        assert!(!is_learning(StudyStage::Review));
        assert!(is_review(StudyStage::Review));
        assert!(!is_review(StudyStage::New));
        assert!(is_relearning(StudyStage::Relearning));
        assert!(!is_relearning(StudyStage::Review));
        assert!(is_active(StudyStage::Learning));
        assert!(is_active(StudyStage::Review));
        assert!(is_active(StudyStage::Relearning));
        assert!(!is_active(StudyStage::New));
    }
}
