use chrono::NaiveDate;
use scheduler_core::domain::{SchedulerOpeningCard, SchedulerTacticCard};
use scheduler_core::{
    CardKind, CardState, InMemoryStore, ReviewGrade, Scheduler, SchedulerConfig,
    build_queue_for_day, new_card,
};
use uuid::Uuid;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("invalid date")
}

struct RelearningFixture {
    scheduler: Scheduler<InMemoryStore>,
    card_id: Uuid,
    today: NaiveDate,
}

fn relearning_fixture() -> RelearningFixture {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 2, 2);

    let mut card = new_card(
        owner,
        CardKind::Tactic(SchedulerTacticCard::new()),
        today,
        &config,
    );
    card.state.stage = CardState::Relearning;
    card.state.interval_days = 1;
    card.state.due = today;
    card.state.ease_factor = 2.0;
    card.state.reviews = 5;
    card.state.lapses = 1;
    let card_id = card.id;
    store.upsert_card(card);

    let scheduler = Scheduler::new(store, config);
    RelearningFixture {
        scheduler,
        card_id,
        today,
    }
}

#[test]
fn sm2_good_review_promotes_new_card() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 1, 1);
    let card = new_card(
        owner,
        CardKind::Tactic(SchedulerTacticCard::new()),
        today,
        &config,
    );
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    let outcome = scheduler
        .review(card_id, ReviewGrade::Good, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state.stage, CardState::Review);
    assert_eq!(outcome.card.state.interval_days, 1);
    assert_eq!(
        outcome.card.state.due,
        today.succ_opt().expect("successor date should exist")
    );
    assert_eq!(outcome.card.state.reviews, 1);

    let store = scheduler.into_store();
    let stored = store.get_card(card_id).expect("card persisted");
    assert_eq!(stored, outcome.card);
}

#[test]
fn sm2_again_resets_interval_and_ease() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig {
        ease_minimum: 1.5,
        ..SchedulerConfig::default()
    };
    let owner = Uuid::new_v4();
    let today = date(2024, 2, 2);

    let mut card = new_card(
        owner,
        CardKind::Tactic(SchedulerTacticCard::new()),
        today,
        &config,
    );
    card.state.stage = CardState::Review;
    card.state.interval_days = 10;
    card.state.due = today;
    card.state.ease_factor = 2.4;
    card.state.reviews = 5;
    let card_id = card.id;
    store.upsert_card(card);

    let mut scheduler = Scheduler::new(store, config.clone());
    let outcome = scheduler
        .review(card_id, ReviewGrade::Again, today)
        .expect("review should succeed");

    assert_eq!(outcome.card.state.stage, CardState::Relearning);
    assert_eq!(outcome.card.state.interval_days, 1);
    assert_eq!(
        outcome.card.state.due,
        today.succ_opt().expect("successor date should exist")
    );
    assert_eq!(outcome.card.state.lapses, 1);
    assert!(outcome.card.state.ease_factor >= config.ease_minimum);
    assert!(outcome.card.state.ease_factor < 2.4);
}

#[test]
fn unlocks_one_opening_per_prefix_per_day() {
    let mut store = InMemoryStore::new();
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let today = date(2024, 3, 3);

    // Existing due review card.
    let mut due_card = new_card(
        owner,
        CardKind::Tactic(SchedulerTacticCard::new()),
        today,
        &config,
    );
    due_card.state.stage = CardState::Review;
    due_card.state.due = today;
    store.upsert_card(due_card.clone());

    // Unlock candidates.
    let opening_a1 = new_card(
        owner,
        CardKind::Opening(SchedulerOpeningCard::new("e4 e5")),
        today,
        &config,
    );
    let opening_a2 = new_card(
        owner,
        CardKind::Opening(SchedulerOpeningCard::new("e4 e5")),
        today,
        &config,
    );
    let opening_b = new_card(
        owner,
        CardKind::Opening(SchedulerOpeningCard::new("d4 d5")),
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
        .filter(|card| card.state.stage == CardState::Learning)
        .collect();
    assert_eq!(unlocked.len(), 2);

    let prefixes: Vec<_> = unlocked
        .iter()
        .map(|card| match &card.kind {
            CardKind::Opening(opening) => opening.parent_prefix.clone(),
            CardKind::Tactic(_) => panic!("expected opening"),
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
    let RelearningFixture {
        mut scheduler,
        card_id,
        today,
    } = relearning_fixture();

    let outcome = scheduler
        .review(card_id, ReviewGrade::Good, today)
        .expect("review should succeed");

    assert_eq!(
        outcome.card.state.stage,
        CardState::Review,
        "Card should graduate from Relearning to Review after Good grade"
    );
    assert_eq!(outcome.card.state.reviews, 6);
    assert_eq!(outcome.card.state.lapses, 1);
}

#[test]
fn relearning_card_graduates_on_hard_review() {
    let RelearningFixture {
        mut scheduler,
        card_id,
        today,
    } = relearning_fixture();

    let outcome = scheduler
        .review(card_id, ReviewGrade::Hard, today)
        .expect("review should succeed");

    assert_eq!(
        outcome.card.state.stage,
        CardState::Review,
        "Card should graduate from Relearning to Review after Hard grade"
    );
}

#[test]
fn relearning_card_graduates_on_easy_review() {
    let RelearningFixture {
        mut scheduler,
        card_id,
        today,
    } = relearning_fixture();

    let outcome = scheduler
        .review(card_id, ReviewGrade::Easy, today)
        .expect("review should succeed");

    assert_eq!(
        outcome.card.state.stage,
        CardState::Review,
        "Card should graduate from Relearning to Review after Easy grade"
    );
}
