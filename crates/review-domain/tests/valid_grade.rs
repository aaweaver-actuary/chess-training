use review_domain::{GradeError, ValidGrade};

#[test]
fn from_u8_accepts_all_valid_grades() {
    for (value, expected) in [
        (0, ValidGrade::Zero),
        (1, ValidGrade::One),
        (2, ValidGrade::Two),
        (3, ValidGrade::Three),
        (4, ValidGrade::Four),
    ] {
        match ValidGrade::from_u8(value) {
            Ok(parsed) => {
                assert_eq!(parsed, expected);
                assert_eq!(parsed.to_u8(), value);
                assert_eq!(parsed.as_u8(), value);
            }
            Err(err) => panic!(
                "grade {value} should parse but returned error variant {}",
                err_label(&err)
            ),
        }
    }
}

#[test]
fn from_u8_rejects_out_of_range_grades() {
    for value in [5, 6, u8::MAX] {
        let err = ValidGrade::from_u8(value).expect_err("grade should fail");

        assert_eq!(err_variant(&err), ErrVariant::OutsideRange(value));
    }
}

#[test]
fn new_forwards_to_from_u8() {
    let created = ValidGrade::new(2).unwrap_or_else(|_| panic!("grade 2 should parse"));
    assert_eq!(created, ValidGrade::Two);
}

#[test]
fn is_correct_checks_threshold() {
    let correct = [ValidGrade::Three, ValidGrade::Four];
    let incorrect = [ValidGrade::Zero, ValidGrade::One, ValidGrade::Two];

    for grade in correct {
        assert!(grade.is_correct(), "grade {grade:?} should be correct");
    }

    for grade in incorrect {
        assert!(!grade.is_correct(), "grade {grade:?} should be incorrect");
    }
}

#[test]
fn to_interval_increment_matches_expected_schedule() {
    assert_eq!(ValidGrade::Zero.to_interval_increment(), 1);
    assert_eq!(ValidGrade::One.to_interval_increment(), 1);
    assert_eq!(ValidGrade::Two.to_interval_increment(), 1);
    assert_eq!(ValidGrade::Three.to_interval_increment(), 2);
    assert_eq!(ValidGrade::Four.to_interval_increment(), 3);
}

#[test]
fn to_grade_delta_returns_supermemo_values() {
    assert!((ValidGrade::Zero.to_grade_delta() - -0.3).abs() < f32::EPSILON);
    assert!((ValidGrade::One.to_grade_delta() - -0.15).abs() < f32::EPSILON);
    assert!((ValidGrade::Two.to_grade_delta() - -0.05).abs() < f32::EPSILON);
    assert!((ValidGrade::Three.to_grade_delta() - 0.0).abs() < f32::EPSILON);
    assert!((ValidGrade::Four.to_grade_delta() - 0.15).abs() < f32::EPSILON);
}

#[test]
fn try_from_accepts_valid_grades() {
    for (value, expected) in [
        (0, ValidGrade::Zero),
        (1, ValidGrade::One),
        (2, ValidGrade::Two),
        (3, ValidGrade::Three),
        (4, ValidGrade::Four),
    ] {
        let parsed: ValidGrade = match ValidGrade::try_from(value) {
            Ok(parsed) => parsed,
            Err(err) => panic!(
                "grade {value} should parse but returned error variant {}",
                err_label(&err)
            ),
        };

        assert_eq!(parsed, expected);
    }
}

#[test]
fn try_from_rejects_invalid_grades() {
    for value in [5, 6, u8::MAX] {
        let err: GradeError =
            ValidGrade::try_from(value).expect_err("grade should fail for invalid value");

        assert_eq!(err_variant(&err), ErrVariant::Invalid(value));
    }
}

#[derive(Debug, PartialEq)]
enum ErrVariant {
    OutsideRange(u8),
    Invalid(u8),
}

fn err_variant(error: &GradeError) -> ErrVariant {
    match error {
        GradeError::GradeOutsideRangeError { grade } => ErrVariant::OutsideRange(*grade),
        GradeError::InvalidGradeError { grade } => ErrVariant::Invalid(*grade),
    }
}

fn err_label(error: &GradeError) -> &'static str {
    match error {
        GradeError::GradeOutsideRangeError { .. } => "GradeOutsideRangeError",
        GradeError::InvalidGradeError { .. } => "InvalidGradeError",
    }
}
