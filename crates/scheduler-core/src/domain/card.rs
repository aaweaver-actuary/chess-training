use chrono::NaiveDate;
use uuid::Uuid;

use crate::{CardKind, CardState, SchedulerConfig};

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub kind: CardKind,
    pub state: CardState,
    pub ease_factor: f32,
    pub interval_days: u32,
    pub due: NaiveDate,
    pub lapses: u32,
    pub reviews: u32,
}

impl Card {
    pub fn new(owner_id: Uuid, kind: CardKind, today: NaiveDate, config: &SchedulerConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            kind,
            state: CardState::New,
            ease_factor: config.initial_ease_factor,
            interval_days: 0,
            due: today,
            lapses: 0,
            reviews: 0,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn common_setup() -> (Uuid, CardKind, NaiveDate, SchedulerConfig) {
        let owner_id = Uuid::new_v4();
        let kind = CardKind::Opening {
            parent_prefix: "e4".to_string(),
        };
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
        Card::new(owner_id, kind, today, &config)
    }

    #[test]
    fn card_new_should_set_id_to_unique_uuid() {
        let (owner_id, kind, today, config) = common_setup();
        let card1 = Card::new(owner_id, kind.clone(), today, &config);
        let card2 = Card::new(owner_id, kind, today, &config);
        assert_ne!(card1.id, card2.id);
    }

    #[test]
    fn card_new_should_set_owner_id_correctly() {
        let (owner_id, _kind, _today, _config) = common_setup();
        let card = Card::new(
            owner_id,
            CardKind::Opening {
                parent_prefix: "e4".to_string(),
            },
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            &SchedulerConfig::default(),
        );
        assert_eq!(card.owner_id, owner_id);
    }

    #[test]
    fn card_new_should_set_kind_correctly() {
        let (owner_id, kind, today, config) = common_setup();
        let card = Card::new(owner_id, kind.clone(), today, &config);
        assert_eq!(card.kind, kind);
    }

    #[test]
    fn card_new_should_set_state_to_new() {
        let card = common_card();
        assert_eq!(card.state, CardState::New);
    }

    #[test]
    fn card_new_should_set_ease_factor_from_config() {
        let (.., config) = common_setup();
        let card = common_card();
        assert!((card.ease_factor - config.initial_ease_factor).abs() < f32::EPSILON);
    }

    #[test]
    fn card_new_should_set_interval_days_to_zero() {
        let card = common_card();
        assert_eq!(card.interval_days, 0);
    }

    #[test]
    fn card_new_should_set_due_to_today() {
        let (.., today, _config) = common_setup();
        let card = common_card();
        assert_eq!(card.due, today);
    }

    #[test]
    fn card_new_should_set_lapses_to_zero() {
        let card = common_card();
        assert_eq!(card.lapses, 0);
    }

    #[test]
    fn card_new_should_set_reviews_to_zero() {
        let card = common_card();
        assert_eq!(card.reviews, 0);
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
        card2.lapses += 1;
        assert_ne!(card1, card2);
    }

    #[test]
    fn card_struct_should_allow_setting_due_to_past_date() {
        let mut card = common_card();
        let past_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        card.due = past_date;
        assert_eq!(card.due, past_date);
    }

    #[test]
    fn card_struct_should_allow_setting_due_to_future_date() {
        let mut card = common_card();
        let future_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap();
        card.due = future_date;
        assert_eq!(card.due, future_date);
    }

    #[test]
    fn card_struct_should_allow_setting_negative_ease_factor_even_if_illogical() {
        let mut card = common_card();
        card.ease_factor = -1.0;
        assert!((card.ease_factor + 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn card_struct_should_allow_setting_interval_days_to_large_value() {
        let mut card = common_card();
        card.interval_days = u32::MAX;
        assert_eq!(card.interval_days, u32::MAX);
    }

    #[test]
    fn card_struct_should_allow_setting_lapses_to_large_value() {
        let mut card = common_card();
        card.lapses = u32::MAX;
        assert_eq!(card.lapses, u32::MAX);
    }

    #[test]
    fn card_struct_should_allow_setting_reviews_to_large_value() {
        let mut card = common_card();
        card.reviews = u32::MAX;
        assert_eq!(card.reviews, u32::MAX);
    }
}
