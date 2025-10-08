//! Review grades supported by the scheduler.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewGrade {
    Again,
    Hard,
    Good,
    Easy,
}

#[cfg(test)]
mod tests {
    use super::ReviewGrade;

    #[test]
    fn grades_are_comparable() {
        assert_eq!(ReviewGrade::Again, ReviewGrade::Again);
        assert_ne!(ReviewGrade::Hard, ReviewGrade::Easy);
    }
}
