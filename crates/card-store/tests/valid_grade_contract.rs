use review_domain::{GradeError, ValidGrade};

#[test]
fn valid_grades_round_trip_between_enum_and_u8() {
    for (value, grade) in [
        (0, ValidGrade::Zero),
        (1, ValidGrade::One),
        (2, ValidGrade::Two),
        (3, ValidGrade::Three),
        (4, ValidGrade::Four),
    ] {
        let parsed = ValidGrade::from_u8(value).unwrap_or_else(|err| {
            panic!(
                "expected {value} to be valid but encountered {}",
                err_label(err)
            )
        });

        assert_eq!(parsed, grade);
        assert_eq!(parsed.to_u8(), value);
        assert_eq!(parsed.as_u8(), value);
        let parsed_try = match ValidGrade::try_from(value) {
            Ok(parsed) => parsed,
            Err(err) => panic!(
                "expected {value} to be valid but encountered {}",
                err_label(err)
            ),
        };
        assert_eq!(parsed_try, grade);
    }
}

#[test]
fn invalid_grades_surface_distinct_errors() {
    for value in [5, 6, u8::MAX] {
        match ValidGrade::from_u8(value) {
            Ok(_) => panic!("expected {value} to be outside range"),
            Err(GradeError::GradeOutsideRangeError { grade }) => assert_eq!(grade, value),
            Err(other) => panic!(
                "unexpected error variant for value {value}: {}",
                err_label(other)
            ),
        }

        match ValidGrade::try_from(value) {
            Ok(_) => panic!("expected {value} to be outside range"),
            Err(GradeError::GradeOutsideRangeError { grade }) => assert_eq!(grade, value),
            Err(other) => panic!(
                "unexpected error variant for value {value}: {}",
                err_label(other)
            ),
        }
    }
}

#[test]
fn grade_helpers_cover_interval_and_ease_adjustments() {
    assert_eq!(ValidGrade::Zero.to_interval_increment(), 1);
    assert_eq!(ValidGrade::One.to_interval_increment(), 1);
    assert_eq!(ValidGrade::Two.to_interval_increment(), 1);
    assert_eq!(ValidGrade::Three.to_interval_increment(), 2);
    assert_eq!(ValidGrade::Four.to_interval_increment(), 3);

    let eps = f32::EPSILON;
    assert!((ValidGrade::Zero.to_grade_delta() - -0.3).abs() < eps);
    assert!((ValidGrade::One.to_grade_delta() - -0.15).abs() < eps);
    assert!((ValidGrade::Two.to_grade_delta() - -0.05).abs() < eps);
    assert!((ValidGrade::Three.to_grade_delta() - 0.0).abs() < eps);
    assert!((ValidGrade::Four.to_grade_delta() - 0.15).abs() < eps);

    assert!(!ValidGrade::Zero.is_correct());
    assert!(!ValidGrade::One.is_correct());
    assert!(!ValidGrade::Two.is_correct());
    assert!(ValidGrade::Three.is_correct());
    assert!(ValidGrade::Four.is_correct());
}

#[test]
fn grade_error_equality_is_value_based() {
    let outside_left = GradeError::GradeOutsideRangeError { grade: 2 };
    let outside_right = GradeError::GradeOutsideRangeError { grade: 2 };
    let invalid_left = GradeError::InvalidGradeError { grade: 5 };
    let invalid_right = GradeError::InvalidGradeError { grade: 5 };

    assert_eq!(outside_left, outside_right);
    assert_eq!(invalid_left, invalid_right);
    assert_ne!(
        outside_left,
        GradeError::GradeOutsideRangeError { grade: 3 }
    );
    assert_ne!(invalid_left, GradeError::InvalidGradeError { grade: 6 });
    assert_ne!(outside_left, invalid_left);
}

fn err_label(error: GradeError) -> &'static str {
    match error {
        GradeError::GradeOutsideRangeError { .. } => "GradeOutsideRangeError",
        GradeError::InvalidGradeError { .. } => "InvalidGradeError",
    }
}
