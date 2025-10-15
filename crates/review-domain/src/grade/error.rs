/// Errors produced when attempting to construct a [`Grade`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradeError {
    /// The provided grade was outside the supported range of 0-4.
    GradeOutsideRangeError { grade: u8 },
    /// The provided grade could not be interpreted as a known review grade.
    InvalidGradeError { grade: u8 },
}
