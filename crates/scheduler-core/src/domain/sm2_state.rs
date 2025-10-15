use super::CardState;
use chrono::NaiveDate;

/// Mutable SM-2 scheduling data tracked for a card.
#[derive(Debug, Clone, PartialEq)]
pub struct Sm2State {
    /// Conceptual study stage for the card.
    pub stage: CardState,
    /// Ease factor controlling interval growth.
    pub ease_factor: f32,
    /// Current interval in days.
    pub interval_days: u32,
    /// Next due date for the card.
    pub due: NaiveDate,
    /// Total number of lapses recorded.
    pub lapses: u32,
    /// Total number of reviews completed.
    pub reviews: u32,
}

impl Sm2State {
    /// Constructs a new SM-2 state for a freshly created card.
    #[must_use]
    pub fn new(stage: CardState, today: NaiveDate, initial_ease: f32) -> Self {
        Self {
            stage,
            ease_factor: initial_ease,
            interval_days: 0,
            due: today,
            lapses: 0,
            reviews: 0,
        }
    }
}

impl Default for Sm2State {
    fn default() -> Self {
        Self {
            stage: CardState::New,
            ease_factor: 2.5,
            interval_days: 0,
            due: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            lapses: 0,
            reviews: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use review_domain::{TEST_EPSILON, assert_is_close};

    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()
    }

    #[test]
    fn test_new_sm2_state_defaults() {
        let stage = CardState::New;
        let initial_ease = 2.5;
        let test_state = Sm2State::new(stage, today(), initial_ease);

        assert_eq!(test_state.stage, stage);
        assert_is_close!(test_state.ease_factor, initial_ease, TEST_EPSILON);
        assert_eq!(test_state.interval_days, 0);
        assert_eq!(test_state.due, today());
        assert_eq!(test_state.lapses, 0);
        assert_eq!(test_state.reviews, 0);
    }

    #[test]
    fn test_new_sm2_state_with_different_stages() {
        let stages = [
            CardState::New,
            CardState::Learning,
            CardState::Review,
            CardState::Suspended,
        ];
        for stage in stages {
            let state = Sm2State::new(stage, today(), 2.0);
            assert_eq!(state.stage, stage);
        }
    }

    #[test]
    fn test_new_sm2_state_with_various_ease_factors() {
        let ease_factors = [
            1.3,
            2.5,
            3.0,
            0.0,
            -1.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ];
        for &ease in &ease_factors {
            let state = Sm2State::new(CardState::Review, today(), ease);
            if ease.is_nan() {
                assert!(state.ease_factor.is_nan());
            } else if ease.is_infinite() {
                assert!(state.ease_factor.is_infinite());
                assert_eq!(
                    state.ease_factor.is_sign_positive(),
                    ease.is_sign_positive()
                );
            } else {
                assert_is_close!(state.ease_factor, ease, TEST_EPSILON);
            }
        }
    }

    #[test]
    fn test_new_sm2_state_with_various_dates() {
        let dates = [
            NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(),
        ];
        for &date in &dates {
            let state = Sm2State::new(CardState::Learning, date, 2.5);
            assert_eq!(state.due, date);
        }
    }

    #[test]
    fn test_sm2_state_clone_and_eq() {
        let state1 = Sm2State::new(CardState::Review, today(), 2.5);
        let state2 = state1.clone();
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_sm2_state_debug_format() {
        let state = Sm2State::new(CardState::New, today(), 2.5);
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("Sm2State"));
        assert!(debug_str.contains("stage"));
        assert!(debug_str.contains("ease_factor"));
        assert!(debug_str.contains("interval_days"));
        assert!(debug_str.contains("due"));
        assert!(debug_str.contains("lapses"));
        assert!(debug_str.contains("reviews"));
    }
}
