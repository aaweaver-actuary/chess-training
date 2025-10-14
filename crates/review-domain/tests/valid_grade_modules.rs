use review_domain::{Grade, GradeError, TEST_EPSILON, assert_is_close};

#[test]
fn conversions_module_exposes_grade_parsing() {
    assert_eq!(Grade::from_u8(3), Ok(Grade::Three));
    assert!(matches!(
        Grade::from_u8(5),
        Err(GradeError::GradeOutsideRangeError { grade: 5 })
    ));
    assert_eq!(Grade::Zero.to_u8(), 0);
}

#[test]
fn conversions_helpers_cover_all_entry_points() {
    assert_eq!(Grade::from_u8(2), Ok(Grade::Two));
    assert!(matches!(
        Grade::from_u8(9),
        Err(GradeError::GradeOutsideRangeError { grade: 9 })
    ));

    let grade = Grade::Three;
    assert_eq!(grade.to_u8(), 3);

    assert_eq!(Grade::from_u8(4), Ok(Grade::Four));
    assert_eq!(Grade::from_u8(1), Ok(Grade::One));
    assert_eq!(Grade::from_u8(0_u8), Ok(Grade::Zero));
}

#[test]
fn accuracy_and_interval_modules_share_responsibilities() {
    assert!(Grade::Four.is_correct());
    assert_eq!(Grade::Three.to_interval_increment(), 2);
    assert_is_close!(Grade::One.to_interval_increment(), 0.15, TEST_EPSILON);
}
