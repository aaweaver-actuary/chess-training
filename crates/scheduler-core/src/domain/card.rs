use chrono::NaiveDate;
use uuid::Uuid;

use review_domain::Card as GenericCard;

use crate::{CardKind, CardState, SchedulerConfig};

use super::Sm2State;

/// Concrete card type used by the scheduler.
pub type Card = GenericCard<Uuid, Uuid, CardKind, Sm2State>;

/// Constructs a new scheduler card with SM-2 defaults.
pub fn new_card(
    owner_id: Uuid,
    kind: CardKind,
    today: NaiveDate,
    config: &SchedulerConfig,
) -> Card {
    Card {
        id: Uuid::new_v4(),
        owner_id,
        kind,
        state: Sm2State::new(CardState::New, today, config.initial_ease_factor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SchedulerOpeningCard;

    fn common_setup() -> (Uuid, CardKind, NaiveDate, SchedulerConfig) {
        let owner_id = Uuid::new_v4();
        let kind = CardKind::Opening(SchedulerOpeningCard::new("e4"));
        let today = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let config = SchedulerConfig {
            initial_ease_factor: 2.5,
            ease_minimum: 1.3,
            ease_maximum: 2.8,
            learning_steps_minutes: vec![1, 10],
        };
        (owner_id, kind, today, config)
    }

    fn common_card() -> Card {
        let (owner_id, kind, today, config) = common_setup();
        new_card(owner_id, kind, today, &config)
    }

    #[test]
    fn card_new_should_set_id_to_unique_uuid() {
        let (owner_id, kind, today, config) = common_setup();
        let card1 = new_card(owner_id, kind.clone(), today, &config);
        let card2 = new_card(owner_id, kind, today, &config);
        assert_ne!(card1.id, card2.id);
    }

    #[test]
    fn card_new_should_set_owner_id_correctly() {
        let (owner_id, _kind, _today, _config) = common_setup();
        let card = new_card(
            owner_id,
            CardKind::Opening(SchedulerOpeningCard::new("e4")),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            &SchedulerConfig::default(),
        );
        assert_eq!(card.owner_id, owner_id);
    }

    #[test]
    fn card_new_should_set_kind_correctly() {
        let (owner_id, kind, today, config) = common_setup();
        let card = new_card(owner_id, kind.clone(), today, &config);
        assert_eq!(card.kind, kind);
    }

    #[test]
    fn card_new_should_set_stage_to_new() {
        let card = common_card();
        assert_eq!(card.state.stage, CardState::New);
    }

    #[test]
    fn card_new_should_set_ease_factor_from_config() {
        let (.., config) = common_setup();
        let card = common_card();
        assert!((card.state.ease_factor - config.initial_ease_factor).abs() < f32::EPSILON);
    }

    #[test]
    fn card_new_should_set_interval_days_to_zero() {
        let card = common_card();
        assert_eq!(card.state.interval_days, 0);
    }

    #[test]
    fn card_new_should_set_due_to_today() {
        let (.., today, _config) = common_setup();
        let card = common_card();
        assert_eq!(card.state.due, today);
    }

    #[test]
    fn card_new_should_set_lapses_to_zero() {
        let card = common_card();
        assert_eq!(card.state.lapses, 0);
    }

    #[test]
    fn card_new_should_set_reviews_to_zero() {
        let card = common_card();
        assert_eq!(card.state.reviews, 0);
    }

    #[test]
    fn card_struct_should_be_cloneable_and_equal_when_cloned() {
        let card = common_card();
        let card_clone = card.clone();
        assert_eq!(card, card_clone);
    }

    #[test]
    fn card_struct_should_not_be_equal_if_any_field_differs() {
        let card1 = common_card();
        let mut card2 = card1.clone();
        card2.state.lapses += 1;
        assert_ne!(card1, card2);
    }

    #[test]
    fn card_struct_should_allow_setting_due_to_past_date() {
        let mut card = common_card();
        let past_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        card.state.due = past_date;
        assert_eq!(card.state.due, past_date);
    }

    #[test]
    fn card_struct_should_allow_setting_due_to_future_date() {
        let mut card = common_card();
        let future_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap();
        card.state.due = future_date;
        assert_eq!(card.state.due, future_date);
    }

    #[test]
    fn card_struct_should_allow_setting_negative_ease_factor_even_if_illogical() {
        let mut card = common_card();
        card.state.ease_factor = -1.0;
        assert!((card.state.ease_factor + 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn card_struct_should_allow_setting_interval_days_to_large_value() {
        let mut card = common_card();
        card.state.interval_days = u32::MAX;
        assert_eq!(card.state.interval_days, u32::MAX);
    }

    #[test]
    fn card_struct_should_allow_setting_lapses_to_large_value() {
        let mut card = common_card();
        card.state.lapses = u32::MAX;
        assert_eq!(card.state.lapses, u32::MAX);
    }

    #[test]
    fn card_struct_should_allow_setting_reviews_to_large_value() {
        let mut card = common_card();
        card.state.reviews = u32::MAX;
        assert_eq!(card.state.reviews, u32::MAX);
    }
}
