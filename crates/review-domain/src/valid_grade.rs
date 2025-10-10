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
    #[must_use]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Creates a new `ValidGrade` if the provided value is between 0 and 4
    /// inclusive. Returns a `GradeError` otherwise.
    /// # Errors
    /// Returns `GradeError::InvalidGrade` if the provided value is not between
    /// 0 and 4 inclusive.
    #[inline]
    pub fn new(grade: u8) -> Result<Self, GradeError> {
        Self::from_u8(grade)
    }

    /// Returns the underlying grade as a `u8`.
    /// Alias for `to_u8()`.
    #[inline]
    #[must_use]
    pub fn as_u8(self) -> u8 {
        self.to_u8()
    }

    /// Returns `true` if the grade is 3 or 4, indicating a correct response.
    /// TODO: Check if this is how we want to define "correct".
    #[inline]
    #[must_use]
    pub fn is_correct(self) -> bool {
        (self as u8) >= 3
    }

    #[inline]
    #[must_use]
    pub fn to_interval_increment(self) -> u8 {
        match self {
            ValidGrade::Zero | ValidGrade::One | ValidGrade::Two => 1,
            ValidGrade::Three => 2,
            ValidGrade::Four => 3,
        }
    }

    /// Returns the grade as a delta to be applied to the easiness factor.
    /// The delta values are based on the `SuperMemo` 2 algorithm.
    /// - Grade 0: -0.3
    /// - Grade 1: -0.15
    /// - Grade 2: -0.05
    /// - Grade 3: 0.0
    /// - Grade 4: +0.15
    #[inline]
    #[must_use]
    pub fn to_grade_delta(self) -> f32 {
        match self {
            ValidGrade::Zero => -0.3,
            ValidGrade::One => -0.15,
            ValidGrade::Two => -0.05,
            ValidGrade::Three => 0.0,
            ValidGrade::Four => 0.15,
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
