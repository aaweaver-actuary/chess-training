use super::ValidGrade;

/// Returns `true` if the grade is 3 or 4, indicating a correct response.
#[must_use]
pub fn is_correct(grade: ValidGrade) -> bool {
    (grade as u8) >= 3
}

impl ValidGrade {
    /// Returns `true` if the grade is 3 or 4, indicating a correct response.
    #[must_use]
    pub fn is_correct(self) -> bool {
        is_correct(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_correct_grades() {
        assert!(!is_correct(ValidGrade::Two));
        assert!(is_correct(ValidGrade::Three));
        assert!(is_correct(ValidGrade::Four));
    }
}
