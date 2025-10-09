# card-store — Unified Persistence Layer for Positions, Edges, Tactics & Cards

> Rust crate providing storage, retrieval, and query logic for all persistent chess-trainer data:
> **positions**, **edges**, **tactics**, **cards**, **reviews**, and **unlock ledgers**.

This crate forms the durable backbone of the system, bridging domain objects from `pgn-import` and `scheduler-core` into a consistent SQL schema (Postgres or SQLite).  
All invariants (uniqueness, referential integrity, schema versioning) are enforced here.

---

## Table of Contents

- [Overview](#overview)
- [Responsibilities](#responsibilities)
- [Supported Backends](#supported-backends)
- [Schema Overview](#schema-overview)
- [Entity Definitions](#entity-definitions)
- [Architecture](#architecture)
- [Feature Summary](#feature-summary)
- [Public API](#public-api)
- [Transactions & Batching](#transactions--batching)
- [Migrations](#migrations)
- [Configuration](#configuration)
- [Testing & Fixtures](#testing--fixtures)
- [Observability](#observability)
- [Example Usage](#example-usage)
- [Roadmap](#roadmap)

---

## Overview

`card-store` unifies all persistent data handling in chess-trainer.  
It defines the **canonical SQL schema**, **Rust models**, and **query helpers** to read/write:

- **Openings** — positions & edges (the trie from `pgn-import`)
- **Tactics** — puzzles extracted from PGNs or other sources
- **Cards** — user-specific spaced-repetition units (openings or tactics)
- **Reviews** — review logs recorded by the scheduler
- **Unlock Ledger** — per-day record of newly released opening moves

It exposes a `Storage` trait consumed by both the CLI tools and the Axum API in `scheduler-core`.

---

## Responsibilities

| Responsibility | Description |
|----------------|--------------|
| **Persistence** | Write/read all domain entities via SQLX with transactional safety. |
| **Uniqueness Enforcement** | Guarantee `(fen)` and `(parent_id, move_uci)` uniqueness for openings. |
| **Referential Integrity** | Enforce FK constraints between positions, edges, cards, and reviews. |
| **Query Interface** | Provide async trait for fetching due cards, unlock candidates, and stats. |
| **Batch Upserts** | Efficiently insert/update thousands of records per import. |
| **Migration Management** | Maintain versioned schema with `sqlx::migrate!()`. |
| **Storage Independence** | Compile-time feature flags for Postgres / SQLite / in-memory backends. |

---

## Supported Backends

| Backend | Feature flag | Notes |
|----------|---------------|-------|
| **Postgres** | `postgres` (default) | Primary production target. Uses `sqlx` async driver. |
| **SQLite** | `sqlite` | Lightweight local mode (tests, offline). |
| **In-Memory** | `inmemory` | HashMap-based reference implementation for tests. |

```toml
[features]
default = ["postgres"]
postgres = ["sqlx/postgres"]
sqlite = ["sqlx/sqlite"]
inmemory = []
```

### Note on In-Memory Stores

This crate provides `InMemoryCardStore`, which is a **full-featured, thread-safe** in-memory implementation supporting the complete domain model (positions, edges, tactics, cards, reviews, unlock ledgers).

There is also a separate `InMemoryStore` in the **scheduler-core** crate, which is a simpler, single-threaded store focused only on scheduling operations (cards and unlock records). The two serve different purposes:

- **Use `card-store::InMemoryCardStore`** when you need the complete storage layer, thread safety, or are testing the full system integration
- **Use `scheduler-core::InMemoryStore`** for lightweight scheduler examples and unit tests that don't require the full domain model

See the scheduler-core [Persistence documentation](../scheduler-core/README.md#persistence) for more details on when to use each.

---

## Schema Overview

```
           ┌──────────────────────────────────────────────────────────┐
           │                         users                            │
           └──────────────────────────────────────────────────────────┘
                           │ 1
                           │
                           ▼
┌──────────────┐    1 ┌──────────────┐ 1     ┌──────────────┐
│ positions    │──────│ edges        │──────▶│ tactics      │
└──────────────┘      └──────────────┘       └──────────────┘
      ▲                    │ 1                     │
      │                    ▼ *                     │
      │            ┌────────────────┐              │
      │            │ cards          │◀─────────────┘
      │            └────────────────┘
      │                    │
      │                    ▼
      │            ┌────────────────┐
      │            │ reviews        │
      │            └────────────────┘
      │                    │
      ▼                    ▼
      ┌────────────────────────────┐
      │ unlock_ledger              │
      └────────────────────────────┘
```

---

## Entity Definitions

### Position
```rust
pub struct Position {
    pub id: i64,              // hash(fen)
    pub fen: String,          // full FEN
    pub side_to_move: char,   // 'w' or 'b'
    pub ply: i32,             // distance from startpos
}
```

### Edge (Opening Move)
```rust
pub struct Edge {
    pub id: i64,                  // hash(parent_fen, move_uci)
    pub parent_id: i64,
    pub move_uci: String,
    pub move_san: String,
    pub child_id: i64,
    pub source_hint: Option<String>,
}
```

### Tactic
```rust
pub struct Tactic {
    pub id: i64,                   // hash(fen, pv_uci)
    pub fen: String,
    pub pv_uci: Vec<String>,
    pub tags: Vec<String>,
    pub source_hint: Option<String>,
}
```

### Card
```rust
pub struct Card {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: CardKind,            // Opening | Tactic
    pub ref_id: i64,               // Edge.id or Tactic.id
    pub state: StoredCardState,    // Scheduling metadata (due date, interval, ease, streak)
    pub due_date: NaiveDate,
    pub interval: i32,
    pub ease: f32,
    pub stability: f32,
    pub difficulty: f32,
    pub reps: i32,
    pub lapses: i32,
}
```

### Review
```rust
pub struct Review {
    pub id: Uuid,
    pub card_id: Uuid,
    pub reviewed_at: NaiveDateTime,
    pub grade: ReviewGrade,
    pub latency_ms: i32,
    pub next_due: NaiveDate,
    pub next_interval: i32,
    pub next_ease: f32,
    pub next_stability: f32,
}
```

### Unlock Ledger
```rust
pub struct UnlockRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub date: NaiveDate,
    pub edge_id: i64,
}
```

---

## Architecture

```plaintext
 ┌──────────────────────────┐
 │ card-store (lib.rs)      │
 │   ├─ model.rs            │ domain structs (serde + sqlx)
 │   ├─ schema.rs           │ migrations + SQL DDL
 │   ├─ postgres_store.rs   │ sqlx::PgPool implementation
 │   ├─ sqlite_store.rs     │ sqlx::SqlitePool implementation
 │   ├─ inmemory_store.rs   │ HashMap test implementation
 │   ├─ queries.rs          │ strongly-typed SQL helpers
 │   └─ metrics.rs          │ counters, timings
 └──────────────────────────┘
```

---

## Feature Summary

- SQL migrations managed via `sqlx::migrate!()`
- Transactional upserts (`ON CONFLICT DO NOTHING/UPDATE`)
- Referential integrity enforced via FK constraints
- Deterministic hashing consistent with `pgn-import` outputs
- Streamed batch inserts for performance
- Asynchronous `Storage` trait used by other crates
- Optional JSON export for debugging / integration tests
- Schema version introspection (`SELECT schema_version()`)

---

## Public API

```rust
use card_store::{Store, CardStore, StorageConfig};
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = StorageConfig::postgres("postgres://app:secret@localhost/chess")?;
    let pool = config.connect().await?;
    let store = CardStore::new(pool);

    // Fetch due cards
    let due_cards = store.fetch_due_cards(user_id, NaiveDate::from_ymd_opt(2025,10,8).unwrap()).await?;

    // Insert new review log
    store.insert_review(&review).await?;

    // Insert tactics or edges in batch
    store.upsert_edges(&edges).await?;
    store.upsert_tactics(&tactics).await?;

    Ok(())
}
```

### Storage Trait

```rust
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    async fn upsert_positions(&self, positions: &[Position]) -> Result<()>;
    async fn upsert_edges(&self, edges: &[Edge]) -> Result<()>;
    async fn upsert_tactics(&self, tactics: &[Tactic]) -> Result<()>;
    async fn upsert_cards(&self, cards: &[Card]) -> Result<()>;
    async fn record_reviews(&self, reviews: &[Review]) -> Result<()>;
    async fn fetch_due_cards(&self, user_id: Uuid, today: NaiveDate) -> Result<Vec<Card>>;
    async fn fetch_unlock_candidates(&self, user_id: Uuid) -> Result<Vec<Edge>>;
    async fn record_unlocks(&self, unlocks: &[UnlockRecord]) -> Result<()>;
}
```

---

## Transactions & Batching

- **Batch inserts**: Default 5–20k records per transaction (configurable).
- **Atomic imports**: `begin_tx()` / `commit()` ensures complete rollback on failure.
- **Async streams**: bulk imports from PGN use `Stream<Item = Vec<Edge>>`.

Example:
```rust
let mut tx = store.begin_tx().await?;
store.upsert_edges_tx(&mut tx, &edges).await?;
store.upsert_positions_tx(&mut tx, &positions).await?;
tx.commit().await?;
```

---

## Migrations

Located in `crates/card-store/migrations/`.

### Example: `V1__initial.sql`
```sql
create table positions (
    id bigint primary key,
    fen text not null unique,
    side_to_move char(1) not null,
    ply int not null
);

create table edges (
    id bigint primary key,
    parent_id bigint not null references positions(id),
    move_uci text not null,
    move_san text not null,
    child_id bigint not null references positions(id),
    unique (parent_id, move_uci)
);

create table tactics (
    id bigint primary key,
    fen text not null,
    pv_uci text[] not null,
    tags text[] not null default '{}',
    source_hint text
);
-- plus cards, reviews, unlock_ledger...
```

Run migrations automatically on startup:
```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

---

## Configuration

```rust
pub struct StorageConfig {
    pub dsn: String,                 // postgres:// or sqlite://
    pub max_connections: u32,        // default: 10
    pub batch_size: usize,           // default: 5000
    pub retry_attempts: u8,          // for transient failures
}
```

Set via environment variables:
```
CARD_STORE_DSN=postgres://app:secret@localhost:5432/chess
CARD_STORE_BATCH_SIZE=5000
```

---

## Testing & Fixtures

### Unit Tests
- `test_upsert_idempotency`: same inputs twice → identical counts.
- `test_foreign_keys`: edge insertion fails if parent position missing.
- `test_due_card_query`: returns only due cards (<= today).
- `test_unlock_record_unique`: one unlock per (user_id, date, edge_id).

### Integration Tests
- Reuse PGN fixture from `tests/data/opening_and_tactic.pgn`.
- Full import → flush to Postgres → read back → verify counts.

### In-Memory Store
- `feature = "inmemory"` enables HashMap-based store for tests.

```bash
cargo test -p card-store --features inmemory
```

---

## Observability

### Metrics
| Metric | Description |
|---------|-------------|
| `positions_upserted_total` | Total inserted positions |
| `edges_upserted_total` | Total inserted edges |
| `tactics_upserted_total` | Total inserted tactics |
| `cards_due_total` | Cards returned by due query |
| `reviews_recorded_total` | New reviews logged |
| `unlock_records_total` | Unlock ledger entries |

### Logging
```
INFO  card_store::postgres > upserted 15,233 edges (12,010 deduped)
INFO  card_store::reviews  > recorded 125 reviews (mean grade=3.7)
```

---

## Example Usage

### Store integration in scheduler-core

```rust
let due = store.fetch_due_cards(user_id, today).await?;
let unlocks = scheduler.unlock_new_moves(user_id, today, &store).await?;
store.record_unlocks(&unlocks).await?;
```

### JSONL Export (debug)
```bash
cargo run -p card-store --features inmemory -- dump --tables positions,edges,tactics > dump.jsonl
```

---

## Roadmap

| Version | Feature |
|----------|----------|
| **v0.1** | Postgres backend, schema migrations, in-memory store |
| **v0.2** | SQLite backend, batch streaming from PGN importer |
| **v0.3** | Query indexes for performance (due cards, unlock ledger) |
| **v0.4** | Parquet/Arrow export for analytics |
| **v0.5** | Schema migrations CLI (`card-store migrate up/down`) |

---

### Summary

`crates/card-store` provides a **single, consistent persistence interface** for everything the chess-trainer system knows: positions, moves, puzzles, and review states.  
It’s designed for **idempotent imports**, **safe updates**, and **deterministic replay**, serving as the shared data backbone for the PGN importer, scheduler, and session gateway.

---