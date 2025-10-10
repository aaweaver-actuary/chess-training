use super::ValidGrade;

/// Returns the grade as a delta to be applied to the easiness factor.
/// The delta values are based on the `SuperMemo` 2 algorithm.
/// - Grade 0: -0.3
/// - Grade 1: -0.15
/// - Grade 2: -0.05
/// - Grade 3: 0.0
/// - Grade 4: +0.15
#[must_use]
pub fn to_grade_delta(grade: ValidGrade) -> f32 {
    match grade {
        ValidGrade::Zero => -0.3,
        ValidGrade::One => -0.15,
        ValidGrade::Two => -0.05,
        ValidGrade::Three => 0.0,
        ValidGrade::Four => 0.15,
    }
}

impl ValidGrade {
    /// Returns the grade as a delta to be applied to the easiness factor.
    /// The delta values are based on the `SuperMemo` 2 algorithm.
    /// - Grade 0: -0.3
    /// - Grade 1: -0.15
    /// - Grade 2: -0.05
    /// - Grade 3: 0.0
    /// - Grade 4: +0.15
    #[must_use]
    pub fn to_grade_delta(self) -> f32 {
        to_grade_delta(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn grade_deltas_follow_supermemo_expectations() {
        assert!((to_grade_delta(ValidGrade::Zero) - -0.3).abs() < EPSILON);
        assert!((to_grade_delta(ValidGrade::One) - -0.15).abs() < EPSILON);
        assert!((to_grade_delta(ValidGrade::Two) - -0.05).abs() < EPSILON);
        assert!((to_grade_delta(ValidGrade::Three) - 0.0).abs() < EPSILON);
        assert!((to_grade_delta(ValidGrade::Four) - 0.15).abs() < EPSILON);
    }
}
