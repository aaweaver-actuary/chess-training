# pgn-import — Opening Trie & Tactic Extraction

Sub-crate that ingests PGN files and produces:
1.	a deduplicated opening trie (positions + move edges), and
2.	a tactics bank from FEN-tagged records or inline comments.

This crate is designed to be the single source of truth for converting heterogeneous PGN sources (Chessable, Lichess studies, ChessBase exports, hand-written files) into well-typed data structures we can store and serve downstream. It prioritizes idempotency, deterministic merging, and explicit invariants so you can trust repeated imports across versions.

⸻

## Table of Contents

- What this crate does
- Key concepts
- Features at a glance
- Installation
- CLI usage
- Library usage
- Data model
- Merging & idempotency rules
- Tactic extraction rules
- Opening trie construction
- Error handling
- Configuration
- Performance notes
- Testing & reproducibility
- Metrics & logging
- FAQ
- Roadmap

⸻

## What this crate does

1.	Parses PGN files into normalized sequences of legal moves with tags, comments, and variations (RAVs).
2.	Builds / merges an opening trie:
- Nodes = positions (FEN).
- Edges = moves (UCI/SAN) from parent → child.
- Repertoire membership is tracked without duplicating nodes/edges.
3.	Extracts tactics:
- From [FEN "..."] PGN tags (preferred).
- Optionally from inline comments (motifs, NAGs, “!”/“!!”/“?!”) if enabled.
4.	Emits in-memory structures and/or persists them via a Storage trait to Postgres, SQLite, or an in-memory store.

This sub-crate does not schedule SRS or run engines; it just produces clean chess data ready for the scheduler and UI.

⸻

## Key concepts

- **Position:** A unique fen string (including side-to-move, castling, ep, half/fullmove).
- **Edge (Move):** A directed transition (parent_fen, move_uci) -> child_fen, with canonical SAN stored for display.
- **Repertoire:** A labeled set of edges attributed to a user/deck (e.g., “Italian White Mainline”).
- **Tactic:** A puzzle defined by a start fen, one or more principal variation(s) in UCI, and optional tags.

⸻

## Features at a glance

- Robust PGN parsing (variations, comments, NAGs, clock, glyphs).
- Legal move validation and SAN↔UCI normalization.
- Transposition-safe merging (keys by FEN+move).
- Idempotent imports (same PGN in twice → no dupes).
- Pluggable storage via Storage trait (Postgres/SQLite/in-mem examples).
- Deterministic traversal & tie-breaking for variations.
- Optional heuristics to extract tactics from comments/NAGs.

⸻

## Installation

Add to the workspace and enable the default features (Rust 1.75+ recommended):

```rust
// Cargo.toml (workspace)
[workspace]
members = [
  "crates/pgn-import",   // this crate
  // ... other members
]
```

```rust
// crates/pgn-import/Cargo.toml
[package]
name = "pgn-import"
version = "0.1.0"
edition = "2021"

[features]
default = ["engine-free"]
engine-free = []             // no engine calls
comment-tactic = []          // enable tactic extraction from comments/NAGs

[dependencies]
thiserror = "1"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hashbrown = "0.14"
fnv = "1"
log = "0.4"
time = "0.3"
itertools = "0.13"

// Choose one PGN library; we wrap behind our parser module.
// Using 'chess-pgn' / 'shakmaty' style ecosystem (as examples).
shakmaty = { version = "0.26", features = ["fen", "uci", "san"] }

// Storage examples (optional)
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio", "macros"], optional = true }
tokio = { version = "1", features = ["macros", "rt-multi-thread"], optional = true }
```


⸻

## CLI usage

The crate provides a thin CLI for batch imports. Build it by enabling the bin target in this crate.

```bash
# Basic import: create/merge an opening trie and tactics into a JSONL snapshot.
pgn-import \
  --input ./data/repertoires/italian_and_scandi.pgn \
  --owner "user_andy" \
  --repertoire "Italian + Scandinavian" \
  --side white \
  --out-positions ./out/positions.jsonl \
  --out-edges ./out/edges.jsonl \
  --out-tactics ./out/tactics.jsonl

With database persistence (Postgres):

pgn-import \
  --input ./data/my_studies.pgn \
  --owner "user_andy" \
  --repertoire "KG Romantic" \
  --side white \
  --dsn "postgres://app:secret@localhost:5432/chess" \
  --persist
```

## Flags (abbreviated):
- `--input <PATH>`: PGN file (can repeat).
- `--owner <STRING>`: logical user/tenant id.
- `--repertoire <STRING>`: label for repertoire membership.
- `--side <white|black|both>`: filter lines by side to move at root.
- `--max-depth <N>`: cap line depth (default: unlimited).
- `--dedupe-transpositions`: enable transposition merging (default: on).
- `--accept-variations`: include RAVs (default: on).
- `--tactic-from-fen`: extract FEN-tagged games as tactics (default: on).
- `--tactic-from-comments`: enable comment/NAG heuristic (requires comment-tactic feature).
- `--out-positions|--out-edges|--out-tactics`: write JSONL snapshots.
- `--dsn`: sqlx DSN (postgres://... | sqlite://...).
- `--persist`: write via Storage implementation.

⸻

## Library usage

```rust
use pgn_import::{
    IngestConfig, Importer, Storage, InMemoryStore, SideFilter,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = IngestConfig {
        owner: "user_andy".into(),
        repertoire: Some("Italian + Scandinavian".into()),
        side_filter: SideFilter::White,   // root side to move constraint
        max_depth: None,
        accept_variations: true,
        tactic_from_fen: true,
        tactic_from_comments: false,
        dedupe_transpositions: true,
    };

    let mut store = InMemoryStore::default(); // or PostgresStore::connect(dsn).await?
    let mut importer = Importer::new(cfg, &mut store);

    importer.ingest_pgn_path("./data/italian_and_scandi.pgn").await?;
    importer.finalize().await?;

    // Access merged outputs
    let positions = store.dump_positions();
    let edges = store.dump_edges();
    let tactics = store.dump_tactics();

    println!("positions: {}, edges: {}, tactics: {}",
             positions.len(), edges.len(), tactics.len());
    Ok(())
}
```


⸻

## Data model

### Position

```rust
pub struct Position {
    pub id: u64,            // stable hash of fen
    pub fen: String,        // full FEN (incl. stm, castling, ep, halves)
    pub side_to_move: char, // 'w' | 'b'
    pub ply: u32,           // distance from startpos in current line (for stats)
}
```

### Edge (Move)

```rust
pub struct Edge {
    pub id: u64,                 // stable hash of (parent_id, move_uci)
    pub parent_id: u64,
    pub move_uci: String,        // e2e4, g1f3, etc.
    pub move_san: String,        // e4, Nf3, etc. (canonicalized)
    pub child_id: u64,
    pub source_hint: Option<String>, // optional: PGN src or tag
}
```

### Repertoire membership

```rust
pub struct RepertoireEdge {
    pub repertoire_key: String,  // "Italian + Scandinavian"
    pub owner: String,           // "user_andy"
    pub edge_id: u64,
}
```

### Tactic

```rust
pub struct Tactic {
    pub id: u64,                   // stable hash of (fen, pv_uci)
    pub fen: String,
    pub pv_uci: Vec<String>,       // principal variation moves in UCI
    pub tags: Vec<String>,         // ["pin","mate-in-2"] ...
    pub source_hint: Option<String>,
}
```

**Hashing:** we use a stable 64-bit hash (e.g., FNV-1a) for ids to keep storage light and diffable. Collisions are extremely unlikely; the storage layer enforces uniqueness by (fen) and (parent, move) anyway.

⸻

## Merging & idempotency rules

1.	Position uniqueness: keyed by full fen.
2.	Edge uniqueness: keyed by (parent_fen, move_uci).
- SAN is recalculated from legal move gen at import, then stored.
3.	Repertoire membership: adding the same edge to the same repertoire is a no-op.
4.	Transpositions: if two different sequences reach the same fen, they converge on the same Position.id and future edges unify automatically.
5.	Idempotent imports: same PGN (or overlapping lines) can be imported repeatedly; no duplication occurs.

⸻

## Tactic extraction rules

### Primary (FEN-tagged)

- If a game (or PGN record) has a [FEN "..."] tag:
- Treat it as a tactic; the starting board is the tag’s FEN.
- The main line (and optionally selected RAV(s)) becomes pv_uci.
- We do not add these moves to the opening trie unless `--include-fen-in-trie` is explicitly enabled.
- The presence of [SetUp "1"] is respected if present; we ignore it otherwise.

### Optional (comments/NAGs)

- With `--tactic-from-comments` or `comment-tactic` feature:
    - If a comment contains motif keys (e.g., `{ tactic: "pin, fork" }`, `{ #tactic mate }`, or `NAG !!, !`), we may create a tactic:
    - Start FEN = the position immediately before the “!” move (or current ply if unambiguous).
    - PV = the following best line until either material advantage or mate marker is observed (heuristic, engine-free).
- This is best-effort and intentionally conservative to avoid noise.

**Rationale:** Many high-quality tactic packs are FEN-tagged. Comment heuristics help salvage value from annotated PGNs but should be used sparingly.

⸻

## Opening trie construction

1.	**Root detection**
- If no `[FEN]`, the root is startpos.
- If a `[FEN]` exists and `--include-fen-in-trie` is on, we treat that as the root; otherwise it’s a tactic (see above).
2.	**Side filtering**
- With `--side white|black`, we only accept lines that begin with that side to move at the root. This does not discard the other side’s moves later in the line—it only filters which trees we ingest.
3.	**Variations (RAVs)**
- If `--accept-variations` is on (default), all RAVs are walked depth-first, in a deterministic order:
- Primary line first, then RAVs sorted by PGN occurrence index.
- When multiple RAVs share the same prefix move from a node, they all map to the same edge (dedup), and downstream repertoire membership simply lists that edge once.
4.	**Legal move enforcement**
- SAN tokens are validated against the generated legal move list from the current FEN.
- Ambiguities are resolved using SAN disambiguators.
- Non-legal SANs produce a structured error (see below); you can opt to `--skip-illegal` or `--fail-fast`.
5.	**Normalization**
- All persisted moves store both SAN (for display) and UCI (for correctness).
- FENs include castling rights and en passant squares to avoid accidental conflation.

⸻

## Error handling

### Representative error enum:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("PGN parse error at game {game_idx}: {msg}")]
    PgnParse { game_idx: usize, msg: String },

    #[error("Illegal SAN `{san}` at ply {ply} (game {game_idx}) from FEN `{fen}`")]
    IllegalMove { game_idx: usize, ply: u32, san: String, fen: String },

    #[error("Unsupported PGN feature: {msg}")]
    Unsupported { msg: String },

    #[error("Storage error: {0}")]
    Storage(#[from] anyhow::Error),
}
```

### Policies

- **Default:** fail-fast on illegal moves.
- `--skip-illegal`: logs and skips malformed branches but continues other games.
- All errors include game index, ply, and FEN to make reproduction trivial.

⸻

## Configuration

You can pass a TOML config (overrides CLI flags):

```toml
# pgn-import.toml
owner = "user_andy"
repertoire = "Italian + Scandinavian"
side = "white"
max_depth = 60
accept_variations = true
tactic_from_fen = true
tactic_from_comments = false
dedupe_transpositions = true
skip_illegal = false

[persistence]
dsn = "postgres://app:secret@localhost:5432/chess"
persist = true

[output]
positions = "./out/positions.jsonl"
edges     = "./out/edges.jsonl"
tactics   = "./out/tactics.jsonl"
```

Load with:

```bash
pgn-import --config ./pgn-import.toml --input ./data/lines.pgn
```


⸻

## Performance notes

- **Streaming parse:** games are processed sequentially; memory footprint bounded by largest game + variation set.
- **Hash maps:** positions and edges dedupe with `hashbrown::HashMap` + FNV hashes for speed.
- **Batching:** if persisting, we buffer upserts (e.g., 5–20k edges) per transaction for Postgres.
- **Transpositions:** enabling dedupe drastically reduces edge writes in rich repertoires.
- **Parallelism:** Safe to shard by file (multiple `Importers`) and merge via storage (unique indexes) if you need throughput.

⸻

## Testing & reproducibility

### Unit & property tests

- **Trie invariants:** (`parent_fen`, `move_uci`) uniqueness; applying the move yields `child_fen`.
- **Idempotency:** importing the same PGN twice yields identical counts & ids.
- **SAN/Legal:** randomly sample legal moves from positions and round-trip SAN↔UCI.

### Golden tests

- Small PGNs in `tests/data/*.pgn` with expected JSONL outputs.
Commit these fixtures to quickly detect behavior drift.

### Determinism

- We ensure deterministic traversal order:
    - Main line before variations.
    - RAVs in file order.
    - Stable sorting for equal cases.

⸻

## Metrics & logging
- Counters
    - `pgn_games_total`, `pgn_games_failed`
    - `positions_inserted`, `positions_deduped`
    - `edges_inserted`, `edges_deduped`
    - `tactics_extracted_fen`, `tactics_extracted_comments`
- Timers
    - `parse_ms`, `merge_ms`, `persist_ms`
- Logs
    - `INFO`: file start/end, counts.
    - `WARN`: skipped branches (if `--skip-illegal`).
    - `ERROR`: failures with context (game, ply, fen).

Integrate with the workspace’s tracing subscriber if available.

⸻

## FAQ

Q: Do you preserve comments and NAGs?
A: We don’t persist arbitrary comments in the core model. We do keep a `source_hint` (e.g., Event/Source) and can expose a side-channel blob if needed later.

Q: How do you handle Chess960?
A: Currently out of scope; FENs will parse but castling semantics aren’t validated. Planned as a feature flag.

Q: What about checkmate/stalemate markers?
A: SAN “#”, “+” are parsed and normalized; they don’t affect merging.

Q: Engine evaluations in comments?
A: Ignored by default. Future: a parser hook to stash evals in a separate table.

⸻

## Roadmap

- v0.2
    - SQLite storage example.
    - Optional include-fen-in-trie for FEN-tagged lines (opening drills from arbitrary starts).
    - Better tactic heuristics from comments (#tactic syntax & NAG weights).
- v0.3
    - Lichess study import helpers (chapter tags → repertoire labels).
    - Incremental import journaling to support resumption on failure.
    - Schema migrations & sqlx query macros behind a feature.
- v0.4
    - Compressed JSONL and Parquet export.

⸻

## Appendix: Storage trait & examples

### Trait

```rust
use crate::{Position, Edge, RepertoireEdge, Tactic};
#[async_trait::async_trait]
pub trait Storage {
    async fn upsert_positions(&mut self, batch: Vec<Position>) -> anyhow::Result<()>;
    async fn upsert_edges(&mut self, batch: Vec<Edge>) -> anyhow::Result<()>;
    async fn upsert_repertoire_edges(&mut self, batch: Vec<RepertoireEdge>) -> anyhow::Result<()>;
    async fn upsert_tactics(&mut self, batch: Vec<Tactic>) -> anyhow::Result<()>;
}
```

### In-memory reference

```rust
pub struct InMemoryStore {
    positions: hashbrown::HashMap<String, Position>,            // fen -> Position
    edges: hashbrown::HashMap<(u64, String), Edge>,             // (parent_id, uci) -> Edge
    repertoire_edges: hashbrown::HashSet<(String, String, u64)>,// (owner, rep_key, edge_id)
    tactics: hashbrown::HashMap<(String, Vec<String>), Tactic>, // (fen, pv_uci)
}
```

### SQL schema sketch (Postgres)

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

create table repertoire_edges (
  owner text not null,
  repertoire_key text not null,
  edge_id bigint not null references edges(id),
  primary key (owner, repertoire_key, edge_id)
);

create table tactics (
  id bigint primary key,
  fen text not null,
  pv_uci text[] not null,
  tags text[] not null default '{}',
  source_hint text
);
```


⸻

## Minimal end-to-end example

### Input PGN (snippet)

```toml
[Event "Study"]
[Site "Local"]
[Date "2025.10.08"]
[Round "-"]
[White "White"]
[Black "Black"]

1. e4 e5 2. Nf3 Nc6 (2... Nf6 3. Nxe5) 3. Bc4 Bc5 4. c3 Nf6 *

[Event "Tactic"]
[Site "Local"]
[Date "2025.10.08"]
[Round "-"]
[White "White"]
[Black "Black"]
[SetUp "1"]
[FEN "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQ1RK1 w kq - 4 6"]

6. Nxe5 Nxe5 7. d4 Nxc4 8. dxc5 *
```

### Result

- Opening trie gains edges for startpos→e2e4, ...→e7e5, ...→g1f3, etc. Variation (2...Nf6 3.Nxe5) merges correctly at the 2...Nf6 node if encountered elsewhere.
- Tactics bank gets one entry:
    - fen = r1bqk2r/... w kq - 4 6
    - pv_uci = ["e4f5", "g6f5", "d2d4", "c5c4", "d1d4"] (example mapping)
