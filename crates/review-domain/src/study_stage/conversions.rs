use super::StudyStage;

/// Converts a character to a `StudyStage`.
#[must_use]
pub fn from_char(c: char) -> Option<StudyStage> {
    match c {
        'N' | 'n' => Some(StudyStage::New),
        'L' | 'l' => Some(StudyStage::Learning),
        'R' | 'r' => Some(StudyStage::Review),
        'E' | 'e' => Some(StudyStage::Relearning),
        _ => None,
    }
}

impl StudyStage {
    /// Converts a character to a `StudyStage`.
    #[must_use]
    pub fn from_char(c: char) -> Option<Self> {
        from_char(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_char_maps_inputs() {
        assert_eq!(from_char('N'), Some(StudyStage::New));
        assert_eq!(from_char('l'), Some(StudyStage::Learning));
        assert_eq!(from_char('R'), Some(StudyStage::Review));
        assert_eq!(from_char('e'), Some(StudyStage::Relearning));
        assert_eq!(from_char('x'), None);
    }

    #[test]
    fn from_char_handles_case_insensitivity() {
        assert_eq!(from_char('n'), Some(StudyStage::New));
        assert_eq!(from_char('L'), Some(StudyStage::Learning));
        assert_eq!(from_char('r'), Some(StudyStage::Review));
        assert_eq!(from_char('E'), Some(StudyStage::Relearning));
        assert_eq!(from_char('0'), None);
    }
}
