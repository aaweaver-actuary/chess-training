//! Minimal example demonstrating how to use the scheduler.
//!
//! Run with: `cargo run -p scheduler-core --example quickstart`

use chrono::NaiveDate;
use scheduler_core::domain::{OpeningCard, TacticCard};
use scheduler_core::{
    Card, CardKind, CardStore, InMemoryStore, ReviewGrade, Scheduler, SchedulerConfig,
};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a configuration (or use defaults)
    let config = SchedulerConfig::default();

    // 2. Initialize an in-memory store
    let mut store = InMemoryStore::new();

    // 3. Add some sample cards to the store
    let owner_id = Uuid::new_v4();
    let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

    // Create a new opening card
    let card1 = Card::new(
        owner_id,
        CardKind::Opening(OpeningCard::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
        )),
        today,
        &config,
    );
    store.upsert_card(card1.clone());

    // Create a tactic card
    let card2 = Card::new(
        owner_id,
        CardKind::Tactic(TacticCard::new()),
        today,
        &config,
    );
    store.upsert_card(card2.clone());

    // 4. Build the scheduler
    let mut scheduler = Scheduler::new(store, config);

    // 5. Build today's queue (includes due reviews + new unlocks)
    let queue = scheduler.build_queue(owner_id, today);
    println!("Cards in today's queue: {}", queue.len());

    // 6. Review a card
    let outcome = scheduler.review(card1.id, ReviewGrade::Good, today)?;
    println!(
        "Reviewed card {}: next due on {}",
        outcome.card.id, outcome.card.due
    );

    Ok(())
}
