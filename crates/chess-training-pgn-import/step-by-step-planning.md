# PGN Import Minimal Implementation Plan

This document organizes the work needed to deliver a minimal, end-to-end implementation of the `pgn-import` crate described in the README. The goal is to ingest PGN files, build an opening trie, extract tactics from FEN-tagged games, and expose the results via an in-memory store and optional JSONL exports.

---

## Crate Procedure Overview

1. **Initialize ingest context** from `IngestConfig`, ensuring feature flags (variations, tactics, dedupe) and deterministic hashing namespaces are respected.
2. **Stream PGN records** and normalize them into sequences of tagged moves plus metadata (tags, comments, RAVs).
3. **Traverse each game** while maintaining the current FEN, validating SAN, deriving UCI, and applying moves via a legal move generator.
4. **Populate the opening trie** by deduplicating positions (by FEN) and edges (by parent FEN + move UCI), updating repertoire membership as configured.
5. **Extract tactics** from games that include `[FEN]` tags (primary) while capturing canonical principal variations.
6. **Persist outputs** by writing to the configured storage (in-memory for minimal scope) and optionally emitting JSONL snapshots that embed schema metadata.
7. **Finalize import** by flushing buffered writes and returning deterministic aggregate statistics / diagnostics.

---

## Task Breakdown

Each task below is designed to be owned independently. Contracts state inputs, outputs, and completion criteria.

### Task Checklist

- [x] **Task 1 – Core Domain Types & Hashing**  
  Notes: Implemented domain structs in `model.rs` with deterministic FNV hashing, serde derives, and unit coverage (100%) confirmed via `make test`.
- [ ] **Task 2 – Ingest Configuration & CLI Parsing**  
  Next Up: Wire config loader to merge CLI + file inputs and document remaining help text polish.
- [ ] **Task 3 – PGN Parsing & Normalization Layer**
- [ ] **Task 4 – Move Application & Legality Engine**
- [ ] **Task 5 – Opening Trie Builder**
- [ ] **Task 6 – Tactic Extraction (FEN-Tagged Minimal Scope)**
- [ ] **Task 7 – Storage Layer (In-Memory + JSONL Export)**
- [ ] **Task 8 – Importer Orchestrator & Finalization**
- [ ] **Task 9 – Testing & Fixtures**
- [ ] **Task 10 – Observability & Dev Ergonomics (Minimal)**

### Shared Data Contracts

```rust
pub struct NormalizedGame {
  pub game_idx: usize,
  pub tags: Vec<(String, String)>,
  pub root_fen: Option<String>,
  pub moves: Vec<SanToken>,
}

pub struct SanToken {
  pub san: String,
  pub ply: u32,
  pub comment: Option<String>,
  pub nags: Vec<String>,
}

pub struct MoveRecord {
  pub parent_fen: String,
  pub move_uci: String,
  pub move_san: String,
  pub child_fen: String,
  pub ply: u32,
}
```

Parsing produces `NormalizedGame` instances without applying legality; for the MVP the `moves` array contains only the main line (variations are trimmed during parsing). The legality layer consumes these structures and produces `MoveRecord` values for downstream trie/tactic builders. These boundaries keep responsibilities crisp and testable.

### 1. Core Domain Types & Hashing
- **Goal:** Establish the shared data structures (`Position`, `Edge`, `RepertoireEdge`, `Tactic`) and stable hashing utilities keyed off explicit schema metadata.
- **Inputs:** Requirements from README, FNV/HashBrown libraries, Rust standard collections.
- **Outputs:** Rust structs with serde derives, `SCHEMA_VERSION` and `HASH_NAMESPACE` constants, hashing helpers returning deterministic `u64` IDs, unit tests covering collisions and equality semantics.
- **Procedure:**
  1. Define structs in `model.rs` (or similar) matching documented fields.
  2. Implement constructors / helpers that accept raw data and compute IDs using the namespace + schema version.
  3. Add serde + Debug derives; ensure `Position::id` and `Edge::id` reflect stable hashes.
  4. Write unit tests for hashing determinism and equality expectations, including namespace changes.
- **Acceptance Criteria:** Structs compile; hashes are stable when namespace/version unchanged and intentionally differ when namespace changes; tests pass across OS/arch.
- **Progress Notes:** Completed on 2025-10-08. `model.rs` hosts finalized structs with serde support; hashing verified by unit tests (`make test` achieves 100% coverage) covering ID stability and serialization of `Position`.

### 2. Ingest Configuration & CLI Parsing
- **Goal:** Provide runtime configuration via struct and clap-based CLI, aligning with README flags while adding ergonomic MVP toggles.
- **Inputs:** `IngestConfig` requirements, clap crate, config TOML loader (minimal: optional file parsing).
- **Outputs:**
  - `IngestConfig` struct with defaults (including `tactic_from_fen`, `include_fen_in_trie`, `require_setup_for_fen`, `skip_malformed_fen`, `max_rav_depth`).
  - CLI binary that populates config, supports optional JSONL output paths, toggles features, and accepts multiple `--input` occurrences.
- **Procedure:**
  1. Define `IngestConfig` plus enums (`SideFilter`) and limits (e.g., `max_rav_depth`).
  2. Implement `ConfigLoader` that merges CLI args, optional TOML files, and environment overrides if desired.
  3. Ensure validation errors are clear (e.g., missing input path, negative depth) and expose `--metrics-json` for Task 10.
- **Acceptance Criteria:** Running `pgn-import --help` lists documented flags; multiple `--input` values accumulate; `max_rav_depth` defaults to 8 with override; UTF-8-only policy explained in help; config struct available for importer.
- **Progress Notes:** Created `config.rs` with documented MVP assumptions and default constants, plus `IngestConfig::default()` matching the checklist expectations. On 2025-10-10 refactored `CliArgs` to use a manual `clap::Command` builder, added parser helpers (`command`, `from_matches`, `try_parse_from`), and expanded CLI tests to exercise every flag. `make test` now passes with 100% coverage across functions/lines/regions.

#### FEN Header Handling & Config Switches

Add explicit configuration knobs to manage messy PGN headers while keeping deterministic behavior:

- `tactic_from_fen: bool = true`
  - When true (default), any record containing a `[FEN]` tag is imported as a tactic.
- `include_fen_in_trie: bool = false`
  - When true, FEN-rooted games also feed the opening trie in addition to tactic extraction.
- `require_setup_for_fen: bool = false`
  - When true, reject records that include `[FEN]` without `[SetUp "1"]`.
- `skip_malformed_fen: bool = false`
  - When true, skip malformed FEN/header combinations instead of failing the import.

All four flags should surface through the CLI, TOML config, and `IngestConfig` (with documentation in `--help`). Defaults favor real-world robustness (accept FEN-only tactics, do not require `SetUp`, fail-fast on malformed data).

**Decision table (default config):**

| Case | Tags Present | FEN Valid? | Behavior | Log Level |
| ---- | ------------ | ---------- | -------- | --------- |
| A | *(none)* | — | Treat as opening from startpos | — |
| B | `SetUp="1"`, `FEN=…` | ✅ | Import as tactic; add to trie if `include_fen_in_trie` | `INFO` (tactic extracted) |
| C | `FEN=…` only | ✅ | Import as tactic; if `require_setup_for_fen` then malformed | `WARN` (`missing_setup`) |
| D | `SetUp="0"`, `FEN=…` | ✅ | Same as C; optional error if `require_setup_for_fen` | `WARN` (`setup_zero`) |
| E | `SetUp="1"` only | — | Malformed (no FEN) | `ERROR`/`WARN` based on `skip_malformed_fen` |
| F | `FEN` invalid string | ❌ | Malformed (invalid FEN) | `ERROR`/`WARN` (`invalid_fen`) |
| G | multiple `FEN` tags | mixed | Use first valid FEN, ignore rest | `WARN` (`multiple_fen`) |
| H | `FEN` + movetext | ✅ | Start board from FEN, treat as tactic by default | `INFO` |

`require_setup_for_fen = true` upgrades Cases C/D to malformed, issuing `ImportError::HeaderMismatch` unless `skip_malformed_fen` is also true. When `skip_malformed_fen = true`, malformed games are dropped with a `WARN` containing file + `game_idx` context.

### 3. PGN Parsing & Normalization Layer
- **Goal:** Convert PGN files into a normalized internal representation of games, moves, tags, and variations with deterministic ordering and guardrails.
- **Inputs:** File paths, `shakmaty` PGN parser, ingest config for variation acceptance.
- **Outputs:** Stream or iterator yielding `NormalizedGame` structures composed of:
  - Root FEN (if tagged, else startpos).
  - Ordered move list with SAN, comments, and metadata limited to the mainline for MVP.
- **Procedure:**
  1. Open files as UTF-8; reject and log non-UTF-8 inputs before parsing.
  2. Wrap `shakmaty` to parse PGN sequentially, enforcing fail-fast vs skip-illegal config.
  3. Classify headers using the decision table; derive a `HeaderStatus` enum (e.g., `Ok`, `MissingSetup`, `InvalidFen`, `SetupZero`, `MultipleFen`) for downstream consumers.
  4. Respect `max_rav_depth`: traverse the mainline plus variations up to the configured depth, trimming deeper branches with a WARN (unless fail-fast).
  5. Normalize SAN tokens, attach tag pairs, capture comment metadata if flagged.
  6. Emit moves in deterministic order: primary line in file order, then RAVs by first appearance, depth-first.
  7. Map malformed headers to `ImportError::InvalidFen` / `ImportError::HeaderMismatch` unless `skip_malformed_fen` is set.
- **Acceptance Criteria:** Parsing handles sample PGNs including variations; non-UTF-8 files are skipped with context; excessive RAV depth produces a WARN and trimming; deterministic ordering verified by repeated imports; header classification recorded on parsed games; malformed headers respect `skip_malformed_fen` and `require_setup_for_fen` policies with accurate errors (game index, ply, FEN context).

### 4. Move Application & Legality Engine
- **Goal:** Validate SAN against legal moves, convert to UCI, and produce next-position FEN snapshots using explicit normalization rules.
- **Inputs:** `NormalizedGame` stream, `shakmaty` board state utilities.
- **Outputs:** Sequence of `(parent_fen, move_uci, move_san, child_fen)` tuples ready for trie insertion; illegal moves yield structured errors or skips per config.
- **Procedure:**
  1. For each game, initialize board from root FEN (startpos or tag) while honoring `[SetUp "1"]` semantics.
  2. Generate legal move list, match SAN (including disambiguation, promotions, castling, +/# markers), and derive UCI.
  3. Apply move to board state, track ply count, and re-normalize FEN values:
     - Include full FEN components (side to move, castling rights in `KQkq` order or `-`, en passant squares, halfmove, fullmove).
     - Reset halfmove clock on captures/pawn moves; remove en passant target after one ply.
  4. On mismatch, return `ImportError::IllegalMove` with context or skip the branch per config.
- **Acceptance Criteria:** Unit tests cover SAN disambiguation (file/rank/both), promotions (including underpromotions), castling with +/# retention, and en-passant behavior; child FENs round-trip to stock startpos and promotions; same inputs yield identical FENs across OS/arch; `--fail-fast` aborts on first illegal SAN while `--skip-illegal` drops the offending branch without affecting mainline continuation.

### 5. Opening Trie Builder
- **Goal:** Maintain deduplicated positions and edges while ingesting move sequences, respecting transpositions and repertoire membership.
- **Inputs:** Stream of validated move tuples, hashing utilities, config flags (`side_filter`, `dedupe_transpositions`).
- **Outputs:**
  - In-memory `HashMap`/`HashSet` structures for positions, edges, repertoire edges.
  - Batch payloads for persistence layer (positions, edges, repertoire assignments).
- **Procedure:**
  1. Filter games based solely on root side-to-move (`SideFilter::White|Black` applies only to the initial FEN) and skip FEN-rooted tactics unless `include_fen_in_trie` is enabled.
  2. For each move tuple, compute IDs, insert/update maps in deterministic order following the parsed sequence.
  3. Track repertoire membership for edges belonging to current owner/repertoire.
  4. Accumulate metrics (counts, dedup hits) for reporting.
- **Acceptance Criteria:** Dedupe logic ensures no duplicate entries; re-importing the same PGN yields identical counts and insertion ordering; root FEN filtering keeps or discards games according to side-to-move semantics.

### 6. Tactic Extraction (FEN-Tagged Minimal Scope)
- **Goal:** Extract tactics from `[FEN]` tagged games, capturing PV lines without impacting trie unless configured.
- **Inputs:** `NormalizedGame` metadata, config flag `tactic_from_fen`.
- **Outputs:** `Tactic` records containing FEN, PV (in UCI), optional tags sourced from PGN metadata, plus metadata such as `header_status` and `source_hint` for downstream JSONL exports.
- **Procedure:**
  1. Detect games with `[FEN]` (and `[SetUp "1"]` support); trust the provided FEN verbatim while consuming `HeaderStatus` from the parser.
  2. Use move application engine to produce the entire main line (ignoring RAVs for MVP even if parsed) until game termination markers (`*`, `1-0`, etc.).
  3. Attach metadata (`header_status`, `source_hint`, side-to-move) to the tactic record.
  4. Hash `(fen, pv)` to create tactic IDs using the shared namespace; dedupe identical `(FEN, PV)` combinations across games.
- **Acceptance Criteria:** Sample PGN with FEN tag produces expected tactic; duplicate `(FEN, PV)` pairs are coalesced; same FEN with different PVs remain distinct tactics; tactics remain importable even when `SideFilter` would omit the opening; malformed FEN headers honor `skip_malformed_fen` / `require_setup_for_fen` policies and surface appropriate `header_status` values.

### 7. Storage Layer (In-Memory + JSONL Export)
- **Goal:** Implement `Storage` trait for in-memory persistence and optional JSONL batch writers.
- **Inputs:** Data batches from trie builder and tactic extractor, serde_json for serialization, file paths from config.
- **Outputs:**
  - `InMemoryStore` conforming to trait.
  - Optional JSONL writers for positions, edges, tactics when configured, emitting type-discriminated records with schema metadata.
- **Procedure:**
  1. Implement `InMemoryStore` using hashbrown maps/sets.
  2. Provide trait methods that upsert deduped data.
  3. Add optional writer that, on finalize, flushes JSONL snapshots where each line includes `type`, `schema_version: SCHEMA_VERSION`, `source_hint`, and when applicable `header_status`; ensure files are stream-concatenatable.
- **Acceptance Criteria:** Store supports insert + dump operations; JSONL files include `{"type":"position","schema_version":1,...}` style envelopes with optional `source_hint`/`header_status`; downstream tools can concatenate outputs safely; tactics serialized from Cases B–G use the correct `header_status` strings (`ok`, `missing_setup`, `invalid_fen`, `setup_zero`, `multiple_fen`).

### 8. Importer Orchestrator & Finalization
- **Goal:** Tie parsing, move validation, trie building, tactic extraction, and storage into a cohesive `Importer` API.
- **Inputs:** `IngestConfig`, storage implementation, tasks 1–7 outputs.
- **Outputs:**
  - `Importer::new(cfg, storage)` constructor.
  - `ingest_pgn_path`, `ingest_pgn_reader`, and `finalize` async methods.
  - Metrics struct summarizing processed games, positions, edges, tactics, errors.
- **Procedure:**
  1. Compose pipelines: open files (supporting multiple inputs), parse, validate, insert into trie/store with deterministic ordering.
  2. Handle buffering/batching (minimal: process sequentially, flush at finalize) while propagating error policy (fail-fast vs skip-illegal).
  3. Emit metrics/logs through `log` crate and surface them to CLI/method callers.
- **Acceptance Criteria:** Integration test runs on sample PGNs produces deterministic counts and metrics across repeated runs; `finalize` exposes metrics struct; API matches README snippets.

### 9. Testing & Fixtures
- **Goal:** Provide automated validation ensuring minimal implementation is correct and deterministic.
- **Inputs:** PGN fixtures (opening + tactic), Rust test harness, property test ideas from README.
- **Outputs:**
  - Unit tests for hashing, parsing, move validation, trie builder.
  - Integration test using small PGN verifying idempotency and tactic extraction.
- **Procedure:**
  1. Create `tests/data/*.pgn` fixtures covering mainline, FEN tactic, promotions (including underpromotion), en-passant, castling, and mixed side-filter roots.
  2. Write tests invoking `Importer` twice to confirm dedupe and identical insertion order / metrics.
  3. Include error-path tests for illegal SAN and config skip option; ensure a bad SAN in a variation does not poison the mainline under `--skip-illegal`.
  4. Add targeted unit tests for deterministic hashes under namespace changes and JSONL schema formatting.
- **Acceptance Criteria:** `cargo test` passes; idempotency double-run assertions hold; SAN edge cases and side-filter behavior verified via fixtures; JSONL outputs contain correct type/schema metadata.

### 10. Observability & Dev Ergonomics (Minimal)
- **Goal:** Ensure developers and CLI users receive actionable feedback.
- **Inputs:** `log` crate, metrics requirements from README.
- **Outputs:** Basic logging instrumentation and counters struct, optional `--metrics-json` dump.
- **Procedure:**
  1. Add structured logs at start/end of import, per-file summaries, and warnings for skipped branches / trimmed RAVs / skipped files / header anomalies (cases C–G).
  2. Collect metrics in a struct returned by `finalize`, tracking at least: `games_total`, `games_parsed`, `games_failed`, `positions_inserted`, `positions_deduped`, `edges_inserted`, `edges_deduped`, `tactics_extracted`, `illegal_moves_skipped`, and counts per `header_status`.
  3. Support emitting metrics as JSON when `--metrics-json <PATH>` is provided.
- **Acceptance Criteria:** Running CLI with sample PGN emits informative logs; metrics struct includes the required counters; JSON output matches schema when requested.

---

## Dependencies & Sequencing Guidance

- Tasks 1–2 lay the foundation; they should complete before downstream work begins.
- Task 3 (parsing) feeds Task 4 (move application); coordinate interfaces early.
- Task 5 depends on Tasks 1 and 4; Task 6 also depends on Task 4.
- Task 7 can proceed once Task 1 is available; integration with Task 8 finalizes the pipeline.
- Testing (Task 9) should run in parallel once core components have stubs, ensuring rapid feedback.
- Observability (Task 10) complements Task 8 but can adopt incremental logging during development.

## Recommended File Layout

```
src/
    lib.rs
    model.rs        # Task 1: domain structs, hashing, schema constants
    config.rs       # Task 2: IngestConfig, CLI args, config loader
    parse.rs        # Task 3: PGN -> NormalizedGame (UTF-8 + RAV guard)
    legal.rs        # Task 4: SAN legality, MoveRecord, FEN normalization
    trie.rs         # Task 5: dedupe maps + repertoire tracking
    tactics.rs      # Task 6: FEN-tagged PV extraction
    storage.rs      # Task 7: Storage trait, InMemoryStore, JSONL writer
    importer.rs     # Task 8: orchestrator plumbing
    metrics.rs      # Task 10: counter struct + JSON serialization
tests/
data/
    opening_and_tactic.pgn
    integration_mvp.rs
```

Contributors can use this skeleton to navigate responsibilities quickly; each module maps directly onto a task.

---

## Scope Confirmations

- Comment-based tactic heuristics are **explicitly excluded** from the MVP.
- External databases are **out of scope**; persistence is limited to the in-memory store plus optional JSONL snapshots.
- Imports run **sequentially** in a single thread for MVP; parallel ingestion can be revisited later.
