//! SM-2 scheduling logic extracted into focused helper functions.

use chrono::{Duration, NaiveDate};
use num_traits::ToPrimitive;

use crate::config::SchedulerConfig;
use crate::domain::{Card, CardState};
use crate::grade::ReviewGrade;

pub(super) fn apply_sm2(
    card: &mut Card,
    grade: ReviewGrade,
    config: &SchedulerConfig,
    today: NaiveDate,
) {
    let previous_reviews = card.state.reviews;
    let previous_interval = card.state.interval_days.max(1);
    let ease = update_ease(card.state.ease_factor, grade, config);
    let interval = interval_for_grade(previous_reviews, previous_interval, grade, ease);
    finalize_review(card, interval, ease, today, grade);
}

pub(super) fn update_ease(current: f32, grade: ReviewGrade, config: &SchedulerConfig) -> f32 {
    let quality = match grade {
        ReviewGrade::Again => 0.0,
        ReviewGrade::Hard => 3.0,
        ReviewGrade::Good => 4.0,
        ReviewGrade::Easy => 5.0,
    };
    let delta = 0.1 - (5.0 - quality) * (0.08 + (5.0 - quality) * 0.02);
    let next = current + delta;
    next.clamp(config.ease_minimum, config.ease_maximum)
}

fn interval_for_grade(
    previous_reviews: u32,
    previous_interval: u32,
    grade: ReviewGrade,
    ease: f32,
) -> u32 {
    match grade {
        ReviewGrade::Again => 1,
        ReviewGrade::Hard => hard_interval(previous_reviews, previous_interval),
        ReviewGrade::Good => good_interval(previous_reviews, previous_interval, ease),
        ReviewGrade::Easy => easy_interval(previous_reviews, previous_interval, ease),
    }
}

fn hard_interval(previous_reviews: u32, previous_interval: u32) -> u32 {
    match previous_reviews {
        0 => 1,
        1 => 4,
        _ => scaled_interval(previous_interval, 1.2),
    }
}

fn good_interval(previous_reviews: u32, previous_interval: u32, ease: f32) -> u32 {
    match previous_reviews {
        0 => 1,
        1 => 6,
        _ => scaled_interval(previous_interval, f64::from(ease)),
    }
}

fn easy_interval(previous_reviews: u32, previous_interval: u32, ease: f32) -> u32 {
    match previous_reviews {
        0 => 1,
        1 => 6,
        _ => scaled_interval(previous_interval, f64::from(ease) * 1.3_f64),
    }
}

fn scaled_interval(previous_interval: u32, factor: f64) -> u32 {
    let scaled = f64::from(previous_interval) * factor;
    if !scaled.is_finite() {
        return u32::MAX;
    }
    let rounded = scaled.round();
    let clamped = rounded.clamp(1.0, f64::from(u32::MAX));
    clamped.to_u32().expect("clamped value should always fit in u32")
}

fn finalize_review(
    card: &mut Card,
    interval: u32,
    ease: f32,
    today: NaiveDate,
    grade: ReviewGrade,
) {
    let due = due_after_interval(today, interval);
    card.state.due = due;
    card.state.interval_days = interval;
    card.state.ease_factor = ease;
    card.state.reviews = card.state.reviews.saturating_add(1);
    card.state.stage = state_after_grade(card.state.stage, grade);
    if matches!(grade, ReviewGrade::Again) {
        card.state.lapses = card.state.lapses.saturating_add(1);
    }
}

fn due_after_interval(today: NaiveDate, interval: u32) -> NaiveDate {
    today
        .checked_add_signed(Duration::days(i64::from(interval)))
        .unwrap_or(today)
}

fn state_after_grade(_current: CardState, grade: ReviewGrade) -> CardState {
    match grade {
        ReviewGrade::Again => CardState::Relearning,
        ReviewGrade::Hard | ReviewGrade::Good | ReviewGrade::Easy => CardState::Review,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SchedulerConfig;
    use crate::domain::{CardKind, CardState, SchedulerTacticCard, new_card};

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_card(stage: CardState) -> Card {
        let config = SchedulerConfig::default();
        let mut card = new_card(
            uuid::Uuid::new_v4(),
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );
        card.state.stage = stage;
        card
    }

    #[test]
    fn update_ease_clamps_values() {
        let config = SchedulerConfig {
            initial_ease_factor: 2.0,
            ease_minimum: 1.4,
            ease_maximum: 2.3,
            learning_steps_minutes: vec![],
        };
        assert!((update_ease(2.5, ReviewGrade::Hard, &config) - 2.3).abs() < f32::EPSILON);
        assert!((update_ease(1.0, ReviewGrade::Again, &config) - 1.4).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_sm2_updates_due_and_state() {
        let config = SchedulerConfig::default();
        let mut card = sample_card(CardState::Review);
        apply_sm2(
            &mut card,
            ReviewGrade::Good,
            &config,
            naive_date(2023, 1, 1),
        );
        assert!(card.state.due >= naive_date(2023, 1, 2));
        assert_eq!(card.state.stage, CardState::Review);
        assert_eq!(card.state.reviews, 1);
    }

    #[test]
    fn apply_sm2_tracks_lapses_for_again_grade() {
        let config = SchedulerConfig::default();
        let mut card = sample_card(CardState::Review);
        apply_sm2(
            &mut card,
            ReviewGrade::Again,
            &config,
            naive_date(2023, 1, 1),
        );
        assert_eq!(card.state.stage, CardState::Relearning);
        assert_eq!(card.state.lapses, 1);
    }

    #[test]
    fn state_after_grade_promotes_relearning_cards() {
        let next = state_after_grade(CardState::Relearning, ReviewGrade::Good);
        assert_eq!(next, CardState::Review);
        let hard = state_after_grade(CardState::Relearning, ReviewGrade::Hard);
        assert_eq!(hard, CardState::Review);
        let easy = state_after_grade(CardState::Relearning, ReviewGrade::Easy);
        assert_eq!(easy, CardState::Review);
    }
}
