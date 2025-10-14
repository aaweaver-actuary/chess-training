//! Shared data structures for representing chess study artifacts.

/// Asserts that the absolute difference between two floating-point values is within the provided tolerance.
///
/// # Examples
/// ```
/// use review_domain::assert_is_close;
/// assert_is_close!(1.0_f32, 1.0 + 1e-6, 1e-5);
/// ```
#[macro_export]
macro_rules! assert_is_close {
    ($left:expr, $right:expr, $tol:expr $(,)?) => {{
        let left = $left;
        let right = $right;
        let tolerance = $tol;
        let difference = (f64::from(left) - f64::from(right)).abs();
        assert!(
            difference <= f64::from(tolerance),
            "assertion failed: |{left:?} - {right:?}| (= {difference:?}) > {tolerance:?}",
            left = left,
            right = right,
            difference = difference,
            tolerance = tolerance,
        );
    }};
}
