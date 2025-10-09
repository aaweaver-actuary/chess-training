#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    /// The card has never been studied; it is new to the learner.
    New,
    /// The card is in the initial learning phase and is being introduced to the learner.
    Learning,
    /// The card has been learned and is being reviewed at increasing intervals.
    Review,
    /// The card was previously learned but has lapsed and is being re-learned.
    Relearning,
}

impl CardState {
    /// Returns true if the card is in the New state.
    pub fn is_new(&self) -> bool {
        matches!(self, CardState::New)
    }

    /// Returns true if the card is in the Learning state.
    pub fn is_learning(&self) -> bool {
        matches!(self, CardState::Learning)
    }

    /// Returns true if the card is in the Review state.
    pub fn is_review(&self) -> bool {
        matches!(self, CardState::Review)
    }

    /// Returns true if the card is in the Relearning state.
    pub fn is_relearning(&self) -> bool {
        matches!(self, CardState::Relearning)
    }

    /// Returns true if the card is in any of the active states (Learning, Review, Relearning).
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            CardState::Learning | CardState::Review | CardState::Relearning
        )
    }

    /// Converts a character to a `CardState`.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'N' | 'n' => Some(CardState::New),
            'L' | 'l' => Some(CardState::Learning),
            'R' | 'r' => Some(CardState::Review),
            'E' | 'e' => Some(CardState::Relearning),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_new() {
        assert!(CardState::New.is_new());
        assert!(!CardState::Learning.is_new());
        assert!(!CardState::Review.is_new());
        assert!(!CardState::Relearning.is_new());
    }

    #[test]
    fn test_is_learning() {
        assert!(!CardState::New.is_learning());
        assert!(CardState::Learning.is_learning());
        assert!(!CardState::Review.is_learning());
        assert!(!CardState::Relearning.is_learning());
    }

    #[test]
    fn test_is_review() {
        assert!(!CardState::New.is_review());
        assert!(!CardState::Learning.is_review());
        assert!(CardState::Review.is_review());
        assert!(!CardState::Relearning.is_review());
    }

    #[test]
    fn test_is_relearning() {
        assert!(!CardState::New.is_relearning());
        assert!(!CardState::Learning.is_relearning());
        assert!(!CardState::Review.is_relearning());
        assert!(CardState::Relearning.is_relearning());
    }

    #[test]
    fn test_is_active() {
        assert!(!CardState::New.is_active());
        assert!(CardState::Learning.is_active());
        assert!(CardState::Review.is_active());
        assert!(CardState::Relearning.is_active());
    }

    #[test]
    fn test_from_char() {
        assert_eq!(CardState::from_char('N'), Some(CardState::New));
        assert_eq!(CardState::from_char('n'), Some(CardState::New));
        assert_eq!(CardState::from_char('L'), Some(CardState::Learning));
        assert_eq!(CardState::from_char('l'), Some(CardState::Learning));
        assert_eq!(CardState::from_char('R'), Some(CardState::Review));
        assert_eq!(CardState::from_char('r'), Some(CardState::Review));
        assert_eq!(CardState::from_char('E'), Some(CardState::Relearning));
        assert_eq!(CardState::from_char('e'), Some(CardState::Relearning));
        assert_eq!(CardState::from_char('X'), None);
    }
}
