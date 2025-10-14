use review_domain::{Grade, GradeError};

#[test]
fn valid_grades_round_trip_between_enum_and_u8() {
    for (value, grade) in [
        (0, Grade::Zero),
        (1, Grade::One),
        (2, Grade::Two),
        (3, Grade::Three),
        (4, Grade::Four),
    ] {
        let parsed = Grade::from_u8(value).unwrap_or_else(|err| {
            panic!(
                "expected {value} to be valid but encountered {}",
                err_label(err)
            )
        });

        assert_eq!(parsed, grade);
        assert_eq!(parsed.to_u8(), value);
        let parsed_try = match Grade::from_u8(value) {
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
        match Grade::from_u8(value) {
            Ok(_) => panic!("expected {value} to be outside range"),
            Err(GradeError::GradeOutsideRangeError { grade }) => assert_eq!(grade, value),
            Err(other) => panic!(
                "unexpected error variant for value {value}: {}",
                err_label(other)
            ),
        }

        match Grade::from_u8(value) {
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
    assert_eq!(Grade::Zero.to_interval_increment(), 1);
    assert_eq!(Grade::One.to_interval_increment(), 1);
    assert_eq!(Grade::Two.to_interval_increment(), 1);
    assert_eq!(Grade::Three.to_interval_increment(), 2);
    assert_eq!(Grade::Four.to_interval_increment(), 3);

    let eps = f32::EPSILON;
    assert!((Grade::Zero.to_grade_delta() - -0.3).abs() < eps);
    assert!((Grade::One.to_grade_delta() - -0.15).abs() < eps);
    assert!((Grade::Two.to_grade_delta() - -0.05).abs() < eps);
    assert!((Grade::Three.to_grade_delta() - 0.0).abs() < eps);
    assert!((Grade::Four.to_grade_delta() - 0.15).abs() < eps);

    assert!(!Grade::Zero.is_correct());
    assert!(!Grade::One.is_correct());
    assert!(!Grade::Two.is_correct());
    assert!(Grade::Three.is_correct());
    assert!(Grade::Four.is_correct());
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
