//! Shared review stage classifications for study cards.

/// High level progress state of a review card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyStage {
    /// The card has never been studied; it is new to the learner.
    New,
    /// The card is in the initial learning phase and is being introduced to the learner.
    Learning,
    /// The card has been learned and is being reviewed at increasing intervals.
    Review,
    /// The card was previously learned but has lapsed and is being re-learned.
    Relearning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_helpers_cover_all_variants() {
        assert!(StudyStage::New.is_new());
        assert!(!StudyStage::Learning.is_new());
        assert!(StudyStage::Learning.is_learning());
        assert!(!StudyStage::Review.is_learning());
        assert!(StudyStage::Review.is_review());
        assert!(!StudyStage::New.is_review());
        assert!(StudyStage::Relearning.is_relearning());
        assert!(!StudyStage::Review.is_relearning());
        assert!(StudyStage::Learning.is_active());
        assert!(StudyStage::Review.is_active());
        assert!(StudyStage::Relearning.is_active());
        assert!(!StudyStage::New.is_active());
    }

    #[test]
    fn from_char_maps_inputs() {
        assert_eq!(StudyStage::from_char('N'), Some(StudyStage::New));
        assert_eq!(StudyStage::from_char('l'), Some(StudyStage::Learning));
        assert_eq!(StudyStage::from_char('R'), Some(StudyStage::Review));
        assert_eq!(StudyStage::from_char('e'), Some(StudyStage::Relearning));
        assert_eq!(StudyStage::from_char('x'), None);
    }

    #[test]
    fn from_char_handles_case_insensitivity() {
        assert_eq!(StudyStage::from_char('n'), Some(StudyStage::New));
        assert_eq!(StudyStage::from_char('L'), Some(StudyStage::Learning));
        assert_eq!(StudyStage::from_char('r'), Some(StudyStage::Review));
        assert_eq!(StudyStage::from_char('E'), Some(StudyStage::Relearning));
        assert_eq!(StudyStage::from_char('0'), None);
    }
}
