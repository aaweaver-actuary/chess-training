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
