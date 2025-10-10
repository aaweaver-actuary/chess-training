/// A grade between 0 and 4 inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidGrade {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

pub enum GradeError {
    /// The provided grade was outside the supported range of 0-4.
    GradeOutsideRangeError {
        grade: u8,
    },
    InvalidGradeError {
        grade: u8,
    },
}

impl ValidGrade {
    /// Converts a `u8` to a `ValidGrade` if it is between 0 and 4 inclusive.
    /// Returns a `GradeError` otherwise.
    /// # Errors
    /// Returns `GradeError::GradeOutsideRangeError` if the provided value is not between
    /// 0 and 4 inclusive.
    #[inline]
    pub fn from_u8(grade: u8) -> Result<Self, GradeError> {
        match grade {
            0 => Ok(ValidGrade::Zero),
            1 => Ok(ValidGrade::One),
            2 => Ok(ValidGrade::Two),
            3 => Ok(ValidGrade::Three),
            4 => Ok(ValidGrade::Four),
            _ => Err(GradeError::GradeOutsideRangeError { grade }),
        }
    }

    /// Converts a `ValidGrade` to a `u8`.
    #[inline]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Creates a new `ValidGrade` if the provided value is between 0 and 4
    /// inclusive. Returns a `GradeError` otherwise.
    /// # Errors
    /// Returns `GradeError::InvalidGrade` if the provided value is not between
    /// 0 and 4 inclusive.
    pub fn new(grade: u8) -> Result<Self, GradeError> {
        Self::from_u8(grade)
    }

    /// Returns the underlying grade as a `u8`.
    /// Alias for `to_u8()`.
    #[inline]
    pub fn as_u8(self) -> u8 {
        self.to_u8()
    }

    /// Returns `true` if the grade is 3 or 4, indicating a correct response.
    /// TODO: Check if this is how we want to define "correct".
    #[inline]
    pub fn is_correct(self) -> bool {
        (self as u8) >= 3
    }

    pub fn to_interval_increment(self) -> u8 {
        match self as u8 {
            0 | 1 => 1,
            2 => 1,
            3 => 2,
            4 => 3,
            _ => unreachable!("grade validated to be between 0 and 4"),
        }
    }

    // if grade == 0 {
    //     -0.3
    // } else if grade == 1 {
    //     -0.15
    // } else if grade == 2 {
    //     -0.05
    // } else if grade == 3 {
    //     0.0
    // } else {
    //     0.15
    // }
    pub fn to_grade_delta(self) -> f32 {
        match self as u8 {
            0 => -0.3,
            1 => -0.15,
            2 => -0.05,
            3 => 0.0,
            4 => 0.15,
            _ => unreachable!("grade validated to be between 0 and 4"),
        }
    }
}

impl TryFrom<u8> for ValidGrade {
    type Error = GradeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ValidGrade::Zero),
            1 => Ok(ValidGrade::One),
            2 => Ok(ValidGrade::Two),
            3 => Ok(ValidGrade::Three),
            4 => Ok(ValidGrade::Four),
            _ => Err(GradeError::InvalidGradeError { grade: value }),
        }
    }
}
