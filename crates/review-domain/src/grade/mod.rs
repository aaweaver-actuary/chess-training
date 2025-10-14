pub mod error;

pub use error::GradeError;

/// A grade between 0 and 4 inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl Grade {
    /// Converts a `u8` to a `Grade` if it is between 0 and 4 inclusive.
    /// Returns a `GradeError` otherwise.
    /// # Errors
    /// Returns `GradeError::GradeOutsideRangeError` if the provided value is not between
    /// 0 and 4 inclusive.
    pub fn from_u8(grade: u8) -> Result<Self, GradeError> {
        match grade {
            0 => Ok(Grade::Zero),
            1 => Ok(Grade::One),
            2 => Ok(Grade::Two),
            3 => Ok(Grade::Three),
            4 => Ok(Grade::Four),
            _ => Err(GradeError::GradeOutsideRangeError { grade }),
        }
    }

    /// Returns the `u8` representation of the `Grade`.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns `true` if the grade is 3 or 4, indicating a correct response.
    #[must_use]
    pub fn is_correct(self) -> bool {
        matches!(self, Grade::Three | Grade::Four)
    }

    /// Returns the multiplicative interval increment associated with the grade.
    #[must_use]
    pub fn to_interval_increment(self) -> u8 {
        match self {
            Grade::Zero | Grade::One | Grade::Two => 1,
            Grade::Three => 2,
            Grade::Four => 3,
        }
    }

    #[must_use]
    pub fn to_grade_delta(self) -> f32 {
        match self {
            Grade::Zero => -0.3,
            Grade::One => -0.15,
            Grade::Two => -0.05,
            Grade::Three => 0.0,
            Grade::Four => 0.15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TEST_EPSILON;

    #[test]
    fn identifies_correct_grades() {
        assert!(!Grade::Zero.is_correct());
        assert!(!Grade::One.is_correct());
        assert!(!Grade::Two.is_correct());
        assert!(Grade::Three.is_correct());
        assert!(Grade::Four.is_correct());
        assert!(!matches!(Grade::Zero, Grade::Four));
    }

    #[test]
    fn constructs_valid_grades() {
        assert_eq!(Grade::from_u8(0).unwrap(), Grade::Zero);
        assert_eq!(Grade::from_u8(1).unwrap(), Grade::One);
        assert_eq!(Grade::from_u8(2).unwrap(), Grade::Two);
        assert_eq!(Grade::from_u8(3).unwrap(), Grade::Three);
        assert_eq!(Grade::from_u8(4).unwrap(), Grade::Four);
        assert!(Grade::from_u8(5).is_err());
        assert!(Grade::from_u8(255).is_err());
    }

    #[test]
    fn interval_increments_match_grade() {
        assert_eq!(Grade::Zero.to_interval_increment(), 1);
        assert_eq!(Grade::One.to_interval_increment(), 1);
        assert_eq!(Grade::Two.to_interval_increment(), 1);
        assert_eq!(Grade::Three.to_interval_increment(), 2);
        assert_eq!(Grade::Four.to_interval_increment(), 3);
    }

    #[test]
    fn conversion_from_u8_bounds_inputs() {
        assert_eq!(Grade::from_u8(0), Ok(Grade::Zero));
        assert_eq!(Grade::from_u8(4), Ok(Grade::Four));
        assert!(matches!(
            Grade::from_u8(5),
            Err(GradeError::GradeOutsideRangeError { grade: 5 })
        ));
    }

    #[test]
    fn conversion_to_u8_round_trips() {
        (0..=4).for_each(|i| {
            let grade = Grade::from_u8(i).unwrap();
            assert_eq!(grade.to_u8(), i);
        });
    }

    #[test]
    fn grade_deltas_follow_supermemo_expectations() {
        (0..=4).for_each(|i| {
            let grade = Grade::from_u8(i).unwrap();
            let delta = match grade {
                Grade::Zero => -0.3,
                Grade::One => -0.15,
                Grade::Two => -0.05,
                Grade::Three => 0.0,
                Grade::Four => 0.15,
            };
            assert!((grade.to_grade_delta() - delta).abs() < TEST_EPSILON);
        });
    }
}
