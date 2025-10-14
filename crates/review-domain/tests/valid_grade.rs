use review_domain::{Grade, GradeError, TEST_EPSILON, assert_is_close};

#[test]
fn from_u8_accepts_all_valid_grades() {
    for (value, expected) in [
        (0, Grade::Zero),
        (1, Grade::One),
        (2, Grade::Two),
        (3, Grade::Three),
        (4, Grade::Four),
    ] {
        match Grade::from_u8(value) {
            Ok(parsed) => {
                assert_eq!(parsed, expected);
                assert_eq!(parsed.to_u8(), value);
            }
            Err(err) => panic!(
                "grade {value} should parse but returned error variant {}",
                err_label(err)
            ),
        }
    }
}

#[test]
fn from_u8_rejects_out_of_range_grades() {
    for value in [5, 6, u8::MAX] {
        let err = Grade::from_u8(value).expect_err("grade should fail");

        assert_eq!(err_variant(err), ErrVariant::OutsideRange(value));
    }
}

#[test]
fn new_forwards_to_from_u8() {
    assert_eq!(Grade::from_u8(2), Ok(Grade::Two));
    assert_eq!(
        Grade::from_u8(9),
        Err(GradeError::GradeOutsideRangeError { grade: 9 })
    );
}

#[test]
fn is_correct_checks_threshold() {
    let correct = [Grade::Three, Grade::Four];
    let incorrect = [Grade::Zero, Grade::One, Grade::Two];

    for grade in correct {
        assert!(grade.is_correct(), "grade {grade:?} should be correct");
    }

    for grade in incorrect {
        assert!(!grade.is_correct(), "grade {grade:?} should be incorrect");
    }
}

#[test]
fn to_interval_increment_matches_expected_schedule() {
    assert_eq!(Grade::Zero.to_interval_increment(), 1);
    assert_eq!(Grade::One.to_interval_increment(), 1);
    assert_eq!(Grade::Two.to_interval_increment(), 1);
    assert_eq!(Grade::Three.to_interval_increment(), 2);
    assert_eq!(Grade::Four.to_interval_increment(), 3);
}

#[test]
fn to_grade_delta_returns_supermemo_values() {
    assert_is_close!(Grade::Zero.to_grade_delta(), -0.3, TEST_EPSILON);
    assert_is_close!(Grade::One.to_grade_delta(), -0.15, TEST_EPSILON);
    assert_is_close!(Grade::Two.to_grade_delta(), -0.05, TEST_EPSILON);
    assert_is_close!(Grade::Three.to_grade_delta(), 0.0, TEST_EPSILON);
    assert_is_close!(Grade::Four.to_grade_delta(), 0.15, TEST_EPSILON);
}

#[test]
fn try_from_accepts_valid_grades() {
    for (value, expected) in [
        (0, Grade::Zero),
        (1, Grade::One),
        (2, Grade::Two),
        (3, Grade::Three),
        (4, Grade::Four),
    ] {
        assert_eq!(Grade::from_u8(value), Ok(expected));
    }
}

#[test]
fn try_from_rejects_invalid_grades() {
    for value in [5, 6, u8::MAX] {
        let err: GradeError =
            Grade::from_u8(value).expect_err("grade should fail for invalid value");

        assert_eq!(err_variant(err), ErrVariant::OutsideRange(value));
    }
}

#[test]
fn grade_error_equality_distinguishes_variants() {
    let outside = GradeError::GradeOutsideRangeError { grade: 7 };
    let invalid = GradeError::InvalidGradeError { grade: 7 };
    assert_ne!(outside, invalid);
}

#[test]
fn grade_error_equality_matches_invalid_variant() {
    let left = GradeError::InvalidGradeError { grade: 2 };
    let right = GradeError::InvalidGradeError { grade: 2 };
    assert_eq!(left, right);
}

#[derive(Debug, PartialEq)]
enum ErrVariant {
    OutsideRange(u8),
    Invalid(u8),
}

fn err_variant(error: GradeError) -> ErrVariant {
    match error {
        GradeError::GradeOutsideRangeError { grade } => ErrVariant::OutsideRange(grade),
        GradeError::InvalidGradeError { grade } => ErrVariant::Invalid(grade),
    }
}

fn err_label(error: GradeError) -> &'static str {
    match error {
        GradeError::GradeOutsideRangeError { .. } => "GradeOutsideRangeError",
        GradeError::InvalidGradeError { .. } => "InvalidGradeError",
    }
}
