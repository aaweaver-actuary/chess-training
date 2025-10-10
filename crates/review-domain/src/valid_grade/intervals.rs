use super::ValidGrade;

/// Returns the multiplicative interval increment associated with the grade.
#[must_use]
pub fn to_interval_increment(grade: ValidGrade) -> u8 {
    match grade {
        ValidGrade::Zero | ValidGrade::One | ValidGrade::Two => 1,
        ValidGrade::Three => 2,
        ValidGrade::Four => 3,
    }
}

impl ValidGrade {
    /// Returns the multiplicative interval increment associated with the grade.
    #[must_use]
    pub fn to_interval_increment(self) -> u8 {
        to_interval_increment(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_increments_match_grade() {
        assert_eq!(to_interval_increment(ValidGrade::Zero), 1);
        assert_eq!(to_interval_increment(ValidGrade::Three), 2);
        assert_eq!(to_interval_increment(ValidGrade::Four), 3);
    }
}
