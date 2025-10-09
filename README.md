# chess-trainer

> **Composable, open-source spaced-repetition training system for chess.**  
> Import your PGNs or tactic packs, automatically build an **opening trie** and **tactics bank**, then review daily through an **Anki-style scheduler** and **interactive chessboard UI**.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Component Responsibilities](#component-responsibilities)
4. [Data Model](#data-model)
5. [Development Workflow](#development-workflow)
6. [Code Layout](#code-layout)
7. [Service-Level Descriptions](#service-level-descriptions)
8. [Build & Run](#build--run)
9. [Typical Data Flow](#typical-data-flow)
10. [Extending the System](#extending-the-system)
11. [Testing](#testing)
12. [Roadmap](#roadmap)
13. [License](#license)

---

## Overview

**chess-trainer** is a modular Rust + Python + TypeScript project built around three principles:

1. **Spaced Repetition, not memorization** — review openings and tactics using an Anki-compatible algorithm (SM-2/FSRS).  
2. **Composable services** — every major subsystem is a self-contained crate or app (Rust core, Node session gateway, React UI, Python workers).  
3. **Deterministic data model** — all chess data (positions, edges, tactics, cards) is canonicalized via FEN/UCIs and stable 64-bit hashes for reproducibility.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                          chess-trainer (workspace)                   │
│                                                                      │
│  ┌────────────────┐     ┌──────────────────┐     ┌─────────────────┐ │
│  │ pgn-import     │ →→  │ scheduler-core   │ →→  │ session-gateway │ │
│  │ (Rust)         │     │ (Rust)           │     │ (Node/TS)       │ │
│  └────────────────┘     └──────────────────┘     └─────────────────┘ │
│         │                               │                 │          │
│         ▼                               ▼                 ▼          │
│  ┌────────────────┐     ┌────────────────────────┐  ┌────────────────┐│
│  │ tactics-store  │     │ openings-store (Postgres)│ │ web-ui (React)││
│  │ (Rust/Python)  │     │ positions/edges/cards   │ │ chessboard.js  ││
│  └────────────────┘     └────────────────────────┘  └────────────────┘│
│         ▲                               ▲                 │          │
│         │                               │                 ▼          │
│  ┌────────────────┐     ┌────────────────────────┐  ┌────────────────┐│
│  │ analysis-worker│ ←── │ review-logs (Python FSRS)│ │ redis / NATS ││
│  │ (Python)       │     └────────────────────────┘  └────────────────┘│
└──────────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities

| Component | Language | Purpose |
|------------|-----------|----------|
| **pgn-import** | Rust | Parse PGNs → build **Opening Trie** + **Tactic Bank**; export to Postgres / JSONL. |
| **scheduler-core** | Rust | Implements Anki-style SRS (SM-2/FSRS). Handles unlocking logic (“1 move/day, shared prefixes”). |
| **openings-store / tactics-store** | Rust + SQL | Persist positions, edges, repertoires, and tactics. |
| **session-gateway** | Node/TypeScript | Session API and WebSocket orchestration between browser and scheduler. |
| **web-ui** | React/TypeScript | Interactive training front-end (chessboard.js, deck management, review stats). |
| **analysis-worker** | Python | Optional: Stockfish annotations, FSRS parameter fitting from review logs. |
| **infrastructure** | YAML / Docker | Postgres, Redis, Stockfish containers, local dev orchestration. |

---

## Data Model

### Core Entities

| Entity | Description | Primary Key |
|--------|--------------|--------------|
| `positions` | Unique board states (full FEN) | hash(fen) |
| `edges` | Directed move `(parent_fen, move_uci → child_fen)` | hash(parent_fen, move_uci) |
| `repertoire_edges` | User repertoire membership for edges | (owner, rep_key, edge_id) |
| `tactics` | Puzzles (FEN + principal variation UCIs) | hash(fen, pv_uci) |
| `cards` | SRS units (opening edge or tactic) | auto/id |
| `reviews` | Review logs (grade, latency, next interval/ease) | auto/id |
| `unlock_ledger` | Daily log of newly unlocked opening moves | (user_id, date, edge_id) |

All IDs are deterministic 64-bit FNV hashes to ensure idempotent imports.

---

## Development Workflow

1. **Import data**
   ```bash
   cargo run -p pgn-import -- \
       --input ./data/repertoires/italian_and_scandi.pgn \
       --owner "andy" \
       --repertoire "Italian + Scandinavian" \
       --out-positions ./out/positions.jsonl \
       --out-edges ./out/edges.jsonl \
       --out-tactics ./out/tactics.jsonl
   ```

2. **Load into the scheduler**
   ```bash
   cargo run -p scheduler-core -- seed ./out/edges.jsonl ./out/tactics.jsonl
   ```

3. **Start the session gateway and UI**
   ```bash
   pnpm run dev  # in /apps/web
   pnpm run start:session  # in /apps/session-gateway
   ```

4. **Review your queue**
   Open http://localhost:5173 → the dashboard shows due cards and today’s unlocked move.

---

## Code Layout

```
chess-trainer/
├── Cargo.toml                 # Workspace manifest
├── README.md                  # This file
├── crates/
│   ├── pgn-import/            # PGN → Opening Trie + Tactics extractor
│   ├── scheduler-core/        # SM-2/FSRS scheduling and unlock logic
│   ├── openings-store/        # SQL persistence layer (optional)
│   └── shared-models/         # Common structs & hashing utilities
├── apps/
│   ├── session-gateway/       # Node/TS server for live review sessions
│   └── web/                   # React front-end
├── workers/
│   └── analysis/              # Python Stockfish + FSRS parameter fitter
├── infrastructure/
│   ├── docker-compose.yml     # Postgres, Redis, Stockfish
│   ├── migrations/            # SQL migrations
│   └── config/                # Default .env / settings
└── tests/
    └── data/                  # PGN fixtures for integration tests
```

---

## Service-Level Descriptions

### 1. **pgn-import**
- Parses PGN using `shakmaty`.
- Validates SAN → UCI, applies moves to derive child FENs.
- Deduplicates positions (by FEN) and edges (by `(parent_fen, move_uci)`).
- Extracts `[FEN]`-tagged games into tactics.
- Emits:
  - JSONL snapshots (`positions`, `edges`, `tactics`)
  - or writes directly to Postgres via the `Storage` trait.
- [Detailed README → `crates/pgn-import/README.md`](./crates/pgn-import/README.md)

### 2. **scheduler-core**
- Implements SRS logic:
  - **SM-2** baseline (interval + ease factor)
  - **FSRS** (stability, difficulty) extension
- Enforces **unlock policy**:
  - 1 new opening move per day per user (merged across shared prefixes)
  - Tactics unaffected; imported directly as cards.
- Provides Rust API and optional Axum HTTP endpoints:
  - `GET /queue/today`
  - `POST /review`
- Tracks review metrics and next-due forecasts.

### 3. **openings-store / tactics-store**
- Shared Postgres schema for persistence.
- `sqlx` migrations create `positions`, `edges`, `tactics`, `cards`, `reviews`.
- Indices ensure `(parent_id, move_uci)` and `(fen)` uniqueness.
- Schema versioned via `schema_version` constant in each crate.

### 4. **session-gateway (Node/TypeScript)**
- Mediates browser sessions → scheduler-core via HTTP/WebSocket.
- Keeps short-lived review sessions in Redis:
  - active card
  - move history / board state
- Endpoints:
  - `/api/session/start` – fetch next due card
  - `/api/session/grade` – post result
  - `/api/session/stats` – live metrics for UI

### 5. **web-ui (React)**
- Built with **Vite + Tailwind**.
- Components:
  - `OpeningReviewBoard` – interactive chessboard.js board.
  - `TacticReviewBoard` – puzzle interface (reveal/solve modes).
  - `DeckManager` – manage repertoires and import new PGNs.
  - `StatsDashboard` – review history and forecast.
- Offline-first (PWA) with local cache sync to session gateway.

### 6. **analysis-worker (Python)**
- Optional background tasks:
  - Run Stockfish analysis (`depth 12–20`) to annotate tactics or openings.
  - Re-fit FSRS parameters from accumulated `reviews`.
- Communicates over Redis or NATS queue.
- Outputs JSONL/CSV summaries used by scheduler-core tuning.

### 7. **infrastructure**
- **Postgres** – canonical data store.
- **Redis** – ephemeral session & queue cache.
- **Stockfish** – engine container for analysis workers.
- Compose file exposes default ports (5432, 6379, 3000, 5173).

---

## Typical Data Flow

```
     ┌───────────┐
     │  PGN File │
     └─────┬─────┘
           │ parse
           ▼
 ┌────────────────────┐
 │ pgn-import (Rust)  │
 │ • Validates SAN    │
 │ • Builds trie       │
 │ • Extracts tactics  │
 └────────┬────────────┘
          │ write
          ▼
 ┌────────────────────┐
 │ openings-store /   │
 │ tactics-store (SQL)│
 └────────┬────────────┘
          │ daily queue
          ▼
 ┌────────────────────┐
 │ scheduler-core     │
 │ • SM-2 / FSRS      │
 │ • unlock 1 move/day│
 └────────┬────────────┘
          │ serve API
          ▼
 ┌────────────────────┐
 │ session-gateway    │
 │ • WebSocket bridge │
 └────────┬────────────┘
          │
          ▼
 ┌────────────────────┐
 │ web-ui (React)     │
 │ • Review board     │
 │ • Stats dashboard  │
 └────────────────────┘
```

---

## Extending the System

| Goal | Where to plug in |
|------|------------------|
| **Add new tactic sources** | Extend `pgn-import/tactics.rs` or build a CSV importer crate. |
| **Different SRS model** | Implement the `Scheduler` trait in `scheduler-core`. |
| **Custom UI** | Consume the `/api/session` endpoints or reuse the React components. |
| **Engine integrations** | Drop Python worker tasks into `workers/analysis/`. |
| **Multi-user auth** | Extend session-gateway with JWT middleware. |

---

## Testing

### Unit Tests
Each crate contains its own unit tests (`cargo test -p crate-name`).

Key invariants tested:
- Trie uniqueness (`(parent, move)` pair unique)
- SAN↔UCI round-trip correctness
- Idempotent imports (same PGN → same counts)
- Scheduler interval updates deterministic given identical grades

### Integration Tests
`tests/integration_mvp.rs` runs:
1. Import `tests/data/opening_and_tactic.pgn`
2. Validate counts
3. Re-import same file (dedupe)
4. Ensure identical metrics

### Frontend Tests
Run `pnpm test` for Jest/React component coverage.

### Coverage & Workspace Checks
Run `make test` to execute formatting, linting, the full Rust test suite, and
LLVM-based coverage verification. The coverage step depends on the
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) subcommand and the
`llvm-tools-preview` rustup component. Install both once per development
environment:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

With those prerequisites satisfied, `make test` will fail the build unless the
workspace maintains 100% line, function, and region coverage for the
`chess-training-pgn-import` crate.

---

## Roadmap

| Milestone | Focus |
|------------|--------|
| **v0.1** | MVP: single-user, sequential import, in-memory + JSONL store. |
| **v0.2** | Postgres storage + API endpoints for scheduler-core. |
| **v0.3** | FSRS parameter fitting + Stockfish analysis worker. |
| **v0.4** | PWA offline sync + user authentication. |
| **v0.5** | Deck sharing, Lichess study import, multi-user scaling. |

---

## License

MIT © chess-trainer contributors

---

## Quick Start (for developers)

```bash
# 1. Clone and enter
git clone https://github.com/yourorg/chess-trainer.git
cd chess-trainer

# 2. Build Rust workspace
cargo build

# 3. Launch core services
docker compose up -d postgres redis
cargo run -p pgn-import -- --input ./tests/data/opening_and_tactic.pgn --owner test --repertoire demo
cargo run -p scheduler-core

# 4. Start UI
pnpm install --prefix apps/web
pnpm --prefix apps/web dev

# 5. Open http://localhost:5173
```

---

### TL;DR

- **Import PGN → Build Trie → Unlock Moves → Review Daily**
- Rust handles all deterministic logic.
- Python adds optional analysis.
- TypeScript powers the interactive experience.
- Everything is modular, reproducible, and testable.
