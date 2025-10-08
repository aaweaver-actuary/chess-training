use chrono::NaiveDate;
use scheduler_core::{
    Card, CardKind, CardState, CardStore, InMemoryStore, ReviewGrade, Scheduler, SchedulerConfig,
    build_queue_for_day,
};
use uuid::Uuid;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("invalid date")
}

#[test]
fn sm2_good_review_promotes_new_card() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 1, 1);
    let card = Card::new(owner, CardKind::Tactic, today, &config);
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    let outcome = scheduler
        .review(card_id, ReviewGrade::Good, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state, CardState::Review);
    assert_eq!(outcome.card.interval_days, 1);
    assert_eq!(outcome.card.due, today.succ_opt().unwrap());
    assert_eq!(outcome.card.reviews, 1);

    let store = scheduler.into_store();
    let stored = store.get_card(card_id).expect("card persisted");
    assert_eq!(stored, outcome.card);
}

#[test]
fn sm2_again_resets_interval_and_ease() {
    let mut store = InMemoryStore::new();
    let mut config = SchedulerConfig::default();
    config.ease_minimum = 1.5;
    let owner = Uuid::new_v4();
    let today = date(2024, 2, 2);

    let mut card = Card::new(owner, CardKind::Tactic, today, &config);
    card.state = CardState::Review;
    card.interval_days = 10;
    card.due = today;
    card.ease_factor = 2.4;
    card.reviews = 5;
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    let outcome = scheduler
        .review(card_id, ReviewGrade::Again, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state, CardState::Relearning);
    assert_eq!(outcome.card.interval_days, 1);
    assert_eq!(outcome.card.due, today.succ_opt().unwrap());
    assert_eq!(outcome.card.lapses, 1);
    assert!(outcome.card.ease_factor >= config.ease_minimum);
    assert!(outcome.card.ease_factor < 2.4);
}

#[test]
fn unlocks_one_opening_per_prefix_per_day() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 3, 3);

    // Existing due review card.
    let mut due_card = Card::new(owner, CardKind::Tactic, today, &config);
    due_card.state = CardState::Review;
    due_card.due = today;
    store.upsert_card(due_card.clone());

    // Unlock candidates.
    let opening_a1 = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "e4 e5".to_string(),
        },
        today,
        &config,
    );
    let opening_a2 = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "e4 e5".to_string(),
        },
        today,
        &config,
    );
    let opening_b = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "d4 d5".to_string(),
        },
        today,
        &config,
    );

    for card in [&opening_a1, &opening_a2, &opening_b] {
        store.upsert_card(card.clone());
    }

    let mut scheduler = Scheduler::new(store, config.clone());
    let first_queue = scheduler.build_queue(owner, today);
    assert_eq!(first_queue.len(), 3, "one due + two unlocked openings");

    let unlocked: Vec<_> = first_queue
        .iter()
        .filter(|card| card.state == CardState::Learning)
        .collect();
    assert_eq!(unlocked.len(), 2);

    let prefixes: Vec<_> = unlocked
        .iter()
        .map(|card| match &card.kind {
            CardKind::Opening { parent_prefix } => parent_prefix.clone(),
            _ => panic!("expected opening"),
        })
        .collect();
    assert_eq!(prefixes.len(), 2);
    assert_ne!(prefixes[0], prefixes[1], "prefixes must be unique");

    let mut store = scheduler.into_store();
    let second_queue = build_queue_for_day(&mut store, &config, owner, today);
    assert_eq!(second_queue.len(), 3, "no additional unlocks on same day");
}

#[test]
fn relearning_card_graduates_on_good_review() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 2, 2);

    // Create a card in Relearning state (after a lapse)
    let mut card = Card::new(owner, CardKind::Tactic, today, &config);
    card.state = CardState::Relearning;
    card.interval_days = 1;
    card.due = today;
    card.ease_factor = 2.0;
    card.reviews = 5;
    card.lapses = 1;
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    
    // Review with Good - should graduate back to Review state
    let outcome = scheduler
        .review(card_id, ReviewGrade::Good, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state, CardState::Review, "Card should graduate from Relearning to Review after Good grade");
    assert_eq!(outcome.card.reviews, 6);
    assert_eq!(outcome.card.lapses, 1);
}

#[test]
fn relearning_card_graduates_on_hard_review() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 2, 2);

    let mut card = Card::new(owner, CardKind::Tactic, today, &config);
    card.state = CardState::Relearning;
    card.interval_days = 1;
    card.due = today;
    card.ease_factor = 2.0;
    card.reviews = 5;
    card.lapses = 1;
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    
    let outcome = scheduler
        .review(card_id, ReviewGrade::Hard, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state, CardState::Review, "Card should graduate from Relearning to Review after Hard grade");
}

#[test]
fn relearning_card_graduates_on_easy_review() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 2, 2);

    let mut card = Card::new(owner, CardKind::Tactic, today, &config);
    card.state = CardState::Relearning;
    card.interval_days = 1;
    card.due = today;
    card.ease_factor = 2.0;
    card.reviews = 5;
    card.lapses = 1;
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    
    let outcome = scheduler
        .review(card_id, ReviewGrade::Easy, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state, CardState::Review, "Card should graduate from Relearning to Review after Easy grade");
}
