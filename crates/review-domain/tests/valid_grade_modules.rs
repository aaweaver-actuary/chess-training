use review_domain::valid_grade::{self, GradeError, accuracy, adjustments, conversions, intervals};

#[test]
fn conversions_module_exposes_grade_parsing() {
    assert_eq!(conversions::from_u8(3), Ok(valid_grade::ValidGrade::Three));
    assert!(matches!(
        conversions::from_u8(5),
        Err(GradeError::GradeOutsideRangeError { grade: 5 })
    ));
    assert_eq!(conversions::to_u8(valid_grade::ValidGrade::Zero), 0);
}

#[test]
fn accuracy_and_interval_modules_share_responsibilities() {
    assert!(accuracy::is_correct(valid_grade::ValidGrade::Four));
    assert_eq!(
        intervals::to_interval_increment(valid_grade::ValidGrade::Three),
        2
    );
    assert!((adjustments::to_grade_delta(valid_grade::ValidGrade::One) - -0.15).abs() < 1e-6);
}
