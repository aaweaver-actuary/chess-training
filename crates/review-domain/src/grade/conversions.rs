use super::{GradeError, ValidGrade};

/// Converts a `u8` to a `ValidGrade` if it is between 0 and 4 inclusive.
/// Returns a `GradeError` otherwise.
/// # Errors
/// Returns `GradeError::GradeOutsideRangeError` if the provided value is not between
/// 0 and 4 inclusive.
pub fn from_u8(grade: u8) -> Result<ValidGrade, GradeError> {
    match grade {
        0 => Ok(ValidGrade::Zero),
        1 => Ok(ValidGrade::One),
        2 => Ok(ValidGrade::Two),
        3 => Ok(ValidGrade::Three),
        4 => Ok(ValidGrade::Four),
        _ => Err(GradeError::GradeOutsideRangeError { grade }),
    }
}

/// Creates a new `ValidGrade` if the provided value is between 0 and 4 inclusive.
/// Returns a `GradeError` otherwise.
/// # Errors
/// Returns `GradeError::GradeOutsideRangeError` if the provided value is not between 0 and 4 inclusive.
pub fn new(grade: u8) -> Result<ValidGrade, GradeError> {
    from_u8(grade)
}

/// Converts a `ValidGrade` to a `u8`.
#[must_use]
pub fn to_u8(grade: ValidGrade) -> u8 {
    grade as u8
}

/// Returns the underlying grade as a `u8`.
/// Alias for `to_u8()`.
#[must_use]
pub fn as_u8(grade: ValidGrade) -> u8 {
    to_u8(grade)
}

impl ValidGrade {
    /// Converts a `u8` to a `ValidGrade` if it is between 0 and 4 inclusive.
    /// Returns a `GradeError` otherwise.
    /// # Errors
    /// Returns `GradeError::GradeOutsideRangeError` if the provided value is not between
    /// 0 and 4 inclusive.
    pub fn from_u8(grade: u8) -> Result<Self, GradeError> {
        from_u8(grade)
    }

    /// Creates a new `ValidGrade` if the provided value is between 0 and 4 inclusive.
    /// Returns a `GradeError` otherwise.
    /// # Errors
    /// Returns `GradeError::GradeOutsideRangeError` if the provided value is not between 0 and 4 inclusive.
    pub fn new(grade: u8) -> Result<Self, GradeError> {
        new(grade)
    }

    /// Converts a `ValidGrade` to a `u8`.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        to_u8(self)
    }

    /// Returns the underlying grade as a `u8`.
    /// Alias for `to_u8()`.
    #[must_use]
    pub fn as_u8(self) -> u8 {
        as_u8(self)
    }
}

impl TryFrom<u8> for ValidGrade {
    type Error = GradeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match from_u8(value) {
            Ok(grade) => Ok(grade),
            Err(_) => Err(GradeError::InvalidGradeError { grade: value }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_from_u8_bounds_inputs() {
        assert_eq!(from_u8(0), Ok(ValidGrade::Zero));
        assert_eq!(from_u8(4), Ok(ValidGrade::Four));
        assert!(matches!(
            from_u8(5),
            Err(GradeError::GradeOutsideRangeError { grade: 5 })
        ));
    }

    #[test]
    fn conversion_to_u8_round_trips() {
        assert_eq!(to_u8(ValidGrade::Zero), 0);
        assert_eq!(as_u8(ValidGrade::Three), 3);
    }
}
