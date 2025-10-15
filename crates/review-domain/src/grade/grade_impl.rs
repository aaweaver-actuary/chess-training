use crate::GradeError;

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

    /// Returns the grade delta as a floating point value.
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
