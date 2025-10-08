# scheduler-core — Spaced Repetition Engine & Unlock Policy

> Rust crate implementing **Anki-compatible SRS scheduling** (SM-2 and FSRS variants),  
> plus the **daily unlocking logic** that releases exactly one new opening move per shared prefix across repertoires.

This crate forms the cognitive backbone of **chess-trainer**, handling all learning-state transitions, review interval updates, and next-review predictions for every card (opening or tactic).

---

## Table of Contents

- [Overview](#overview)
- [Quickstart](#quickstart)
- [Responsibilities](#responsibilities)
- [Key Concepts](#key-concepts)
- [Feature Summary](#feature-summary)
- [Architecture](#architecture)
- [Public API](#public-api)
- [Unlock Policy for Openings](#unlock-policy-for-openings)
- [Scheduling Models](#scheduling-models)
- [Data Model](#data-model)
- [Configuration](#configuration)
- [Persistence](#persistence)
- [CLI / Service Usage](#cli--service-usage)
- [Metrics & Observability](#metrics--observability)
- [Testing Strategy](#testing-strategy)
- [Extending the Scheduler](#extending-the-scheduler)
- [Roadmap](#roadmap)

---

## Overview

The scheduler’s job is to decide **what to study next and when**.

It consumes:
- `edges` (opening moves) and `tactics` (puzzles),
- prior `reviews` (user grades, response times), and
- the current day’s date.

It produces:
- a **review queue** (`Vec<Card>`),
- updated **card states** after each review,
- optional next-day unlocks for new opening moves.

It is deterministic: given the same review history, the same future queue is produced every time.

---

## Quickstart

Here's a minimal working example showing how to wire up the scheduler, build a queue, and review cards:

```rust
use scheduler_core::{
    Card, CardKind, CardState, InMemoryStore, ReviewGrade, Scheduler, SchedulerConfig,
};
use chrono::NaiveDate;
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
        CardKind::Opening {
            parent_prefix: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string(),
        },
        today,
        &config,
    );
    store.upsert_card(card1.clone());
    
    // Create a tactic card
    let card2 = Card::new(
        owner_id,
        CardKind::Tactic,
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
    println!("Reviewed card {}: next due on {}", outcome.card.id, outcome.card.due);
    
    Ok(())
}
```

This example demonstrates the core workflow:
- **Configure** the scheduler with `SchedulerConfig`
- **Store** cards using `InMemoryStore` (for testing/examples)
- **Schedule** reviews with the `Scheduler`
- **Build queues** to get today's cards
- **Process reviews** and update card states

For production use, replace `InMemoryStore` with a persistent backend implementing the `CardStore` trait (see [Persistence](#persistence)).

You can also run this example directly:
```bash
cargo run -p scheduler-core --example quickstart
```

---

## Responsibilities

| Responsibility | Description |
|----------------|--------------|
| **Scheduling** | Compute next review date, interval, and ease/stability after each review. |
| **Unlocking** | Enforce “one new opening move per day across shared prefixes” policy. |
| **Queuing** | Select and order due cards for today (mix of due reviews + new unlocks + optional tactics). |
| **Review logging** | Record outcomes with timestamps, grades, latency. |
| **Persistence hooks** | Provide a trait-based abstraction for card storage so backends can be Postgres, in-memory, or JSONL. |
| **Statistics** | Summarize performance, success rates, and forecast upcoming load. |

---

## Key Concepts

### Cards
- A **card** represents one learnable item:
  - **Opening card:** a single move (edge) connecting two positions.
  - **Tactic card:** one puzzle (FEN + PV).
- Each card has:
  - a `state` (`New`, `Learning`, `Review`, `Relearning`),
  - review parameters (`interval`, `ease`, `stability`, `difficulty`, `due_date`),
  - review logs (grade outcomes).

### Queue
- The list of cards due for review today.
- Computed from:
  - all cards where `due_date <= today`
  - plus the day’s unlocked new openings
  - plus (optionally) a sample of new tactics.

### Unlock Ledger
- Tracks which opening edges have been unlocked per day.
- Ensures only one **new move per shared prefix** is introduced per user/day.

---

## Feature Summary

- **SM-2 algorithm** (classic Anki)
- **FSRS** (Free Spaced Repetition Scheduler) parameters (optional)
- Configurable intervals, ease factor, lapse penalties
- Shared-prefix unlock logic for opening moves
- Tactic cards independent of unlock schedule
- Review logging with full traceability
- Deterministic queue generation for testability
- Optional Axum-based HTTP API (`feature = "server"`)

---

## Architecture

```
 ┌──────────────────────┐
 │ scheduler-core (lib) │
 │   ├─ state.rs        │  Card, SchedulerState
 │   ├─ algo_sm2.rs     │  SM-2 model
 │   ├─ algo_fsrs.rs    │  FSRS model (optional)
 │   ├─ unlock.rs       │  daily unlock policy
 │   ├─ queue.rs        │  queue builder
 │   ├─ storage.rs      │  trait abstraction for card storage
 │   ├─ review.rs       │  grade → next interval computation
 │   └─ metrics.rs      │  counters, summaries
 └──────────────────────┘
```

---

## Public API

```rust
use scheduler_core::{
    Scheduler, SchedulerConfig, CardKind, ReviewGrade, ReviewLog, MemoryModel,
    InMemoryStore, QueueBuilder,
};
use chrono::NaiveDate;

fn example() -> anyhow::Result<()> {
    // Initialize scheduler with default SM-2 model
    let mut store = InMemoryStore::default();
    let config = SchedulerConfig::default();
    let mut scheduler = Scheduler::new(config, MemoryModel::Sm2, &mut store);

    // Build today’s queue
    let queue = scheduler.build_daily_queue(NaiveDate::from_ymd_opt(2025, 10, 8).unwrap())?;
    println!("Cards due today: {}", queue.len());

    // Process a review
    scheduler.review_card(
        "card_id_123",
        ReviewGrade::Good,
        3200, // latency_ms
        NaiveDate::from_ymd_opt(2025, 10, 8).unwrap()
    )?;

    scheduler.flush()?;
    Ok(())
}
```

---

## Unlock Policy for Openings

### Goal
Each day, unlock **at most one new move** per user across all repertoires that share the same prefix.

### Core logic
1. Gather all candidate edges:
   - Those belonging to any active repertoire.
   - Whose parent position is already “mature” (learned or mastered).
   - That have no card yet.
2. Group by parent position (shared prefix).
3. Choose one edge per parent group (minimal depth or random tie-break).
4. Create `Card { kind = Opening, state = New, due_date = today }`.
5. Record in `unlock_ledger`.

```rust
fn unlock_new_moves(user_id: Uuid, today: NaiveDate) -> Vec<Card> {
    let eligible = find_eligible_edges(user_id);
    let grouped = group_by_parent(eligible);
    grouped
        .into_iter()
        .filter(|(parent, _)| !already_unlocked_today(user_id, parent, today))
        .map(|(_, edges)| pick_one(edges))
        .collect()
}
```

**Result:** If you have both Italian and Scandinavian repertoires starting with `1.e4`, you only learn that shared `e4` once; the second repertoire “inherits” that move automatically.

---

## Scheduling Models

### SM-2 (default)
- Based on the original SuperMemo algorithm.
- Tracks:
  - `interval`, `ease_factor`, `lapses`.
- Grades map to:
  | Grade | Effect |
  |--------|---------|
  | Again | reset interval (1d) |
  | Hard | reduce ease |
  | Good | normal growth |
  | Easy | increase ease |

### FSRS (optional)
- Modern probabilistic model (stability/difficulty).
- Adds parameters:
  - `stability`, `difficulty`, `retrievability`.
- Each review updates these via regression-like formulas.
- Ideal for future adaptive tuning from user logs.

---

## Data Model

| Field | Type | Description |
|--------|------|-------------|
| `card_id` | `Uuid` | Unique card key |
| `kind` | enum { Opening, Tactic } | Card type |
| `ref_id` | `u64` | Linked edge/tactic |
| `state` | enum { New, Learning, Review, Relearning } | Learning phase |
| `due_date` | `NaiveDate` | Next scheduled review |
| `interval` | `u32` | Days until next review |
| `ease` | `f32` | Ease factor (SM-2) |
| `stability` | `f32` | Memory stability (FSRS) |
| `difficulty` | `f32` | Memory difficulty (FSRS) |
| `reps` | `u32` | Total repetitions |
| `lapses` | `u32` | Times failed |
| `last_review` | `Option<DateTime>` | Timestamp of last review |
| `review_logs` | `Vec<ReviewLog>` | History of grades/latencies |

### ReviewLog
```rust
pub struct ReviewLog {
    pub reviewed_at: NaiveDate,
    pub grade: ReviewGrade,     // Again, Hard, Good, Easy
    pub latency_ms: u64,
    pub next_interval: u32,
    pub next_ease: f32,
    pub next_due: NaiveDate,
}
```

---

## Configuration

```rust
pub struct SchedulerConfig {
    pub max_new_openings_per_day: usize, // default: 1
    pub include_tactics: bool,           // include tactics in daily queue
    pub model: MemoryModel,              // Sm2 or Fsrs
    pub min_interval_days: u32,          // floor for interval
    pub max_interval_days: u32,          // cap for interval
    pub randomize_order: bool,           // shuffle daily queue
    pub lapse_penalty: f32,              // ease penalty on failure
    pub ease_floor: f32,                 // min ease
    pub ease_ceiling: f32,               // max ease
}
```

**Defaults:**
```rust
SchedulerConfig {
    max_new_openings_per_day: 1,
    include_tactics: true,
    model: MemoryModel::Sm2,
    min_interval_days: 1,
    max_interval_days: 365,
    randomize_order: true,
    lapse_penalty: 0.8,
    ease_floor: 1.3,
    ease_ceiling: 2.5,
}
```

---

## Persistence

This crate is **backend-agnostic**.  
Persistence is abstracted behind a `Storage` trait.

```rust
#[async_trait::async_trait]
pub trait Storage {
    async fn fetch_due_cards(&self, user_id: Uuid, today: NaiveDate) -> Result<Vec<Card>>;
    async fn fetch_unlock_candidates(&self, user_id: Uuid) -> Result<Vec<Edge>>;
    async fn upsert_cards(&self, cards: &[Card]) -> Result<()>;
    async fn record_reviews(&self, reviews: &[ReviewLog]) -> Result<()>;
    async fn record_unlocks(&self, unlocks: &[UnlockRecord]) -> Result<()>;
}
```

**Provided implementations:**
- `InMemoryStore` (for testing and CLI)
- `PostgresStore` (via `sqlx`, optional feature)

### Understanding the Two In-Memory Stores

You may notice **two separate in-memory store implementations** in this codebase:

1. **`scheduler-core::InMemoryStore`** (this crate)
   - **Purpose:** Lightweight test/example store for the scheduler
   - **Scope:** Cards, unlock records, and basic scheduling state
   - **Use when:** Writing unit tests for scheduler logic, creating minimal examples, or running CLI demos
   - **Thread-safety:** Single-threaded (uses `BTreeMap`)
   - **Location:** `crates/scheduler-core/src/lib.rs`

2. **`card-store::InMemoryCardStore`** (sibling crate)
   - **Purpose:** Full-featured reference implementation of the `CardStore` trait
   - **Scope:** Positions, edges, tactics, cards, reviews, and unlock ledgers
   - **Use when:** Testing the complete storage layer, integrating with PGN import, or needing thread-safe operations
   - **Thread-safety:** Multi-threaded (uses `RwLock<HashMap>`)
   - **Location:** `crates/card-store/src/memory.rs`

**Why two implementations?**
- **Separation of concerns:** `scheduler-core` focuses purely on scheduling algorithms and doesn't need the full domain model (positions, edges, tactics)
- **Minimal dependencies:** `scheduler-core::InMemoryStore` has no external dependencies, making it ideal for simple examples
- **Different use cases:** The scheduler's store is for quick tests/demos; `card-store`'s is for complete system integration

**Which should you use?**
- **For learning/examples:** Use `scheduler-core::InMemoryStore` (as shown in [Quickstart](#quickstart))
- **For production:** Use `card-store::InMemoryCardStore` or a persistent backend (Postgres/SQLite)
- **For integration tests:** Use `card-store::InMemoryCardStore` to test the full stack

---

## CLI / Service Usage

The crate can run as a standalone binary with `--feature server`.

```bash
cargo run -p scheduler-core --features server -- serve --dsn postgres://user:pass@localhost/chess
```

### Endpoints
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/queue/today` | Get due cards for the current user |
| `POST` | `/review` | Submit a review result `{ card_id, grade, latency_ms }` |
| `GET` | `/stats` | Return aggregate stats (reps, accuracy, next_due forecast) |

---

## Metrics & Observability

Exports Prometheus-friendly counters and histograms:

| Metric | Description |
|---------|-------------|
| `cards_due_total` | Number of due cards computed for day |
| `cards_unlocked_total` | Number of newly unlocked opening cards |
| `reviews_total` | Number of processed reviews |
| `reviews_by_grade` | Histogram by review grade |
| `average_interval_days` | Mean interval length |
| `average_ease` | Mean ease factor |

Logs via `tracing`:
```
INFO  scheduler_core::queue > built queue: 35 due, 1 unlocked, 5 tactics
INFO  scheduler_core::review > reviewed card c123: grade=Good, next_due=2025-10-09
```

---

## Testing Strategy

### Unit tests
- Verify SM-2 and FSRS math correctness.
- Test unlock policy invariants (max 1 per prefix/day).
- Ensure card updates are deterministic given identical inputs.

### Property tests
- Randomized review sequences → identical results on replay.
- Stable ID hashing for unlock ledger.

### Integration tests
- `tests/integration_queue.rs` builds queue from mock storage.
- `tests/integration_review.rs` applies sequential grades and checks intervals.

---

## Extending the Scheduler

To add a new model:
1. Implement the `MemoryModel` trait:
   ```rust
   pub trait MemoryModelImpl {
       fn next_state(&self, card: &Card, grade: ReviewGrade) -> Card;
   }
   ```
2. Add your variant to `MemoryModel` enum.
3. Register it in `SchedulerFactory`.

To adjust unlock policy:
- Modify `unlock.rs`, but maintain invariants:
  - No duplicates per `(user_id, date, parent_fen)`.

---

## Roadmap

| Version | Feature |
|----------|----------|
| **v0.1** | SM-2 baseline, in-memory store, CLI queue builder |
| **v0.2** | FSRS model, Postgres backend, unlock ledger persistence |
| **v0.3** | API server (Axum) + metrics |
| **v0.4** | Cross-user concurrency & daily digest report |
| **v0.5** | Adaptive difficulty calibration from real review logs |

---

### Summary

`crates/scheduler-core` is the **learning engine** of chess-trainer.  
It connects cognitive science (SRS algorithms) with your opening & tactic data, ensuring consistent, explainable learning progression.

- Deterministic scheduling ✅  
- Configurable unlock policy ✅  
- Modular, tested, and extensible ✅

---