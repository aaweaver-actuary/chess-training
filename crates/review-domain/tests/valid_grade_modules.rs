use review_domain::grade::{accuracy, adjustments, conversions, intervals};
use review_domain::{GradeError, ValidGrade};

#[test]
fn conversions_module_exposes_grade_parsing() {
    assert_eq!(conversions::from_u8(3), Ok(ValidGrade::Three));
    assert!(matches!(
        conversions::from_u8(5),
        Err(GradeError::GradeOutsideRangeError { grade: 5 })
    ));
    assert_eq!(conversions::to_u8(ValidGrade::Zero), 0);
}

#[test]
fn conversions_helpers_cover_all_entry_points() {
    assert_eq!(conversions::new(2), Ok(ValidGrade::Two));
    assert!(matches!(
        conversions::new(9),
        Err(GradeError::GradeOutsideRangeError { grade: 9 })
    ));

    let grade = ValidGrade::Three;
    assert_eq!(conversions::as_u8(grade), 3);
    assert_eq!(grade.to_u8(), 3);
    assert_eq!(grade.as_u8(), 3);

    assert_eq!(ValidGrade::from_u8(4), Ok(ValidGrade::Four));
    assert_eq!(ValidGrade::new(1), Ok(ValidGrade::One));
    assert_eq!(ValidGrade::try_from(0_u8), Ok(ValidGrade::Zero));
}

#[test]
fn accuracy_and_interval_modules_share_responsibilities() {
    assert!(accuracy::is_correct(ValidGrade::Four));
    assert_eq!(intervals::to_interval_increment(ValidGrade::Three), 2);
    assert!((adjustments::to_grade_delta(ValidGrade::One) - -0.15).abs() < 1e-6);
}
