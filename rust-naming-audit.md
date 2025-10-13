# Rust Naming Audit

This audit reorganizes the earlier glossary by broader use cases while still covering every struct, enum, type alias, and free function defined in the Rust crates. Similar items with overlapping responsibilities are listed together so naming inconsistencies are easier to spot. Every description answers *what the item is for* and *why it exists* in plain English.

---

## 1. Configuration, Defaults, and CLI Plumbing

These types centralize tunable knobs, offer defaults, or translate user input (files/CLI/JS) into runtime settings.

- **`StorageConfig`** (`crates/card-store/src/config.rs`)
  - Stores connection pool limits, batching, and retry counts for card-store backends so deployments can tune persistence without code changes.
  - *Related items:* `SchedulerConfig` (SM-2 tuning), `IngestConfig` (PGN importer knobs), `SchedulerConfigDto`/`SchedulerConfigPatch` (wasm serialization/patching), `FileConfig` and `CliArgs` (PGN CLI).

- **`IngestConfig`**, **`FileConfig::from_path`**, **`CliArgs::{command, from_matches, try_parse_from, into_ingest_config}`** (`crates/chess-training-pgn-import/src/config.rs`)
  - Collect configuration inputs from TOML and command line, ensuring PGN ingestion has all required flags and default fallbacks.
  - *Related items:* `Importer::new` uses these settings; `SchedulerFacade::new` and `WasmScheduler::new` also merge optional patches before instantiating services.

- **`SchedulerConfig`** (`crates/scheduler-core/src/config.rs`)
  - Keeps SM-2 defaults (initial ease, clamps, learning steps) so schedulers can be created consistently.
  - *Related items:* `SchedulerConfigDto`/`SchedulerConfigPatch` expose the same fields to wasm; `Sm2State::new` depends on it to seed state.

- **`SchedulerConfigDto::from`** & **`SchedulerConfigPatch::apply`** (`crates/scheduler-wasm/src/config.rs`)
  - Provide JSON-friendly snapshots and merge logic so JS callers can inspect and tweak SM-2 parameters before handing them back to Rust.
  - *Related items:* `WasmScheduler::new` and `SchedulerFacade::new` rely on the patch output.

- **`SchedulerFacade::new`** (`crates/scheduler-wasm/src/scheduler.rs`) and **`WasmScheduler::new`** (`crates/scheduler-wasm/src/bindings.rs`)
  - Bundle configuration and in-memory stores into ready-to-use schedulers for wasm consumers, optionally honoring user-supplied patches.
  - *Related items:* `Scheduler::new` inside scheduler-core performs the same wiring for native use.

- **Binary `main` stubs** (`crates/chess-training-pgn-import/src/main.rs`, `crates/scheduler-core/src/main.rs`, root `src/main.rs`)
  - Provide minimal entry points (currently printing “Hello, world!”) so crates build as binaries.
  - *Related items:* tests guarding they don’t panic; all of these share placeholder naming and could eventually converge on `fn run()` helpers for consistency.

**Naming observations for this group:**
- Config structs consistently end with `Config`, but helper methods vary between `from_matches`, `try_parse_from`, `apply`, and `into_ingest_config`. They follow domain idioms, yet the CLI pipeline mixes `into_*` and `from_*` verbs. Consider renaming `CliArgs::into_ingest_config` to `build_ingest_config` to mirror other builders.

---

## 2. Domain Models, IDs, and Hashing

These types capture chess domain entities and strongly typed identifiers to avoid collisions.

- **Type aliases in `crates/card-store/src/model.rs`** (`Edge`, `EdgeMap`, `CardMap`, `UnlockSet`, etc.)
  - Tailor generic review-domain types to store-specific key/value maps for clarity when manipulating in-memory data.
  - *Related items:* `CardAggregate` (review-domain) and scheduler-specific aliases (`Card`, `CardKind`). Consistency could improve by suffixing all aliases with `_Map` or `_Set` uniformly (some already do).

- **`build_opening_card_id` / `build_tactic_card_id`** (`crates/card-store/src/model.rs`)
  - Deterministically hash owners with edges/tactics to create reproducible card IDs and prevent duplicates.
  - *Related items:* `hash_with_seed` and `Position::new` in the importer, plus `hash64` in review-domain. The `build_*` prefix now matches other constructors like `build_opening_card`, though aligning on `hash_*_id` could further emphasize the hashing step.

- **`Position`, `OpeningEdgeRecord`, `RepertoireEdge`, `Tactic`, `hash_with_seed`** (`crates/chess-training-pgn-import/src/model.rs`)
  - Encapsulate deterministic hashing and payload construction when importing PGNs, ensuring consistent IDs across runs.
  - *Related items:* `OpeningEdge` and `TacticCard` in review-domain use similar naming; however, importer structs append `Record`/`Edge`, while domain structs use `OpeningEdge`. Consider harmonizing suffixes (e.g., `OpeningEdgeRecord` vs. `OpeningEdge`).

- **`ChessPosition`, `OpeningEdge`, `RepertoireMove`, `Repertoire`, `RepertoireBuilder`, `TacticCard`, `OpeningCard`, `UnlockRecord`, `UnlockDetail`** (`crates/review-domain/src/*.rs`)
  - Define the canonical data structures for chess positions, openings, repertoire moves, tactics, and unlock metadata shared across crates.
  - *Related items:* Scheduler reuses these through aliases. Naming is largely consistent (`*Card`, `*Move`, `*Record`), though `UnlockDetail::new` sits beside generic `UnlockRecord`, unlike other modules where constructors are `::new_*` (e.g., `CardAggregate::new_opening`).

- **`hash64`** (`crates/review-domain/src/hash.rs`)
  - Wraps BLAKE3 hashing for deterministic 64-bit IDs.
  - *Related items:* `hash_with_seed`, `build_opening_card_id`. All use “hash” but some embed the target entity while others don’t; consider exposing a shared `fn hash_entity(namespace, bytes)` helper for uniform terminology.

- **Strong ID macros in `crates/review-domain/src/ids.rs`**
  - Generate newtype wrappers (`PositionId`, `EdgeId`, `CardId`, etc.) to prevent ID misuse.
  - *Related items:* Scheduler’s `Card` alias binds these IDs to UUIDs, showing consistent `Id` suffix usage.

**Naming observations for this group:**
- Constructors are mostly `::new`, but some domain-specific ones are `::new_opening`, `into_edge`, etc. Ensure “into” is only used for consuming conversions (as in `EdgeInput::into_edge`) and consider renaming `RepertoireEdge::new` to `::from_move` if it consumes multiple IDs, for clarity.

---

## 3. Storage Traits, In-Memory Stores, and Persistence Helpers

Everything here orchestrates saving/retrieving positions, edges, cards, and unlocks.

- **`CardStore` trait & `StoreError` enum** (`crates/card-store/src/store.rs`)
  - Abstract persistence across backends with operations to upsert positions, edges, cards, unlocks, and reviews; enumerate failure cases (missing data, collisions, invalid inputs).
  - *Related items:* `Storage` trait (`chess-training-pgn-import/src/storage.rs`) and scheduler-core’s `CardStore` trait (`crates/scheduler-core/src/store.rs`). The duplicate trait names (`CardStore`) could confuse consumers when both crates are in scope—consider namespacing (e.g., `ReviewCardStore`, `SchedulerStore`).

- **`InMemoryCardStore`** and helper lock guards (`crates/card-store/src/memory/in_memory_card_store.rs`)
  - Provide a thread-safe demo backend using RwLocks; wrap lock acquisition in `*_read`/`*_write` helpers to centralize poison handling.
  - *Related items:* `InMemoryImportStore` (PGN importer) and scheduler-core’s `InMemoryStore`. Naming now aligns on the `InMemory*Store` prefix across fixtures.

- **`store_opening_card`, `collect_due_cards_for_owner`, `borrow_card_for_review`, `validate_existing_opening_card`, `build_opening_card`** (`crates/card-store/src/memory/cards.rs`)
  - Manage the card map by inserting or reusing deterministic cards, retrieving due cards, and validating collisions.
  - *Related items:* `store_canonical_position`, `store_canonical_edge`, `insert_unlock_or_error`. Verb choices mix `store_*`, `insert_*`, and `build_*`. If consistency is desired, consider `build_opening_card` → `make_opening_card` or `store_*` → `upsert_*` to match trait terminology.

- **`store_canonical_edge`, `validate_edge_collision`** (`crates/card-store/src/memory/edges.rs`)
  - Ensure edges are deduplicated and collisions flagged.
  - *Related items:* `store_canonical_position` shares the `store_` prefix; adding `validate_*` pairs to both modules is consistent.

- **`canonicalize_position_for_storage`, `store_canonical_position`, `validate_position_collision`** (`crates/card-store/src/memory/position_helpers.rs`)
  - Normalize and de-duplicate chess positions before storage, returning errors when hashes collide with mismatched FEN strings.
  - *Related items:* Similar naming to edge helpers; consistent use of `canonical*` conveys purpose.

- **`apply_review`**, **`map_grade_error`** (`crates/card-store/src/memory/reviews.rs`)
  - Apply learner grades to stored card state and convert domain errors into storage-layer errors.
  - *Related items:* `apply_sm2` (scheduler-core) and `CardAggregate::apply_review`. While verbs align (`apply_*`), `map_grade_error` could become `map_grade_error_to_store_error` for explicitness.

- **`insert_unlock_or_error`** (`crates/card-store/src/memory/unlocks.rs`)
  - Insert unlock records unless a duplicate date/edge combination already exists.
  - *Related items:* Scheduler-core’s `record_unlock` uses the `record_*` prefix; aligning on `record_unlock` vs. `insert_unlock` would help cross-crate comprehension.

- **`Storage` trait, `UpsertOutcome`, `InMemoryImportStore`** (`crates/chess-training-pgn-import/src/storage.rs`)
  - Wrap card-store persistence behind a simpler interface tailored for importer needs, tracking whether upserts inserted or replaced.
  - *Related items:* `CardStore` trait shares method names (`upsert_*`). `InMemoryImportStore` parallels other in-memory stores but adds `*_records` getters; consider `into_*` naming for getters returning owned data to distinguish from clones.

- **Scheduler-core `CardStore` trait & `InMemoryStore`** (`crates/scheduler-core/src/store.rs`)
  - Handle SM-2 card persistence, due card queries, unlock candidate retrieval, and unlock logging.
  - *Related items:* Card-store’s trait; method names align (`upsert_card`, `due_cards_for_owner`), which is good, but trait names colliding remains a concern.

**Naming observations for this group:**
- The verbs `store_*`, `insert_*`, `record_*`, `upsert_*`, and `build_*` mix across modules. Picking one convention per action type (e.g., `upsert_` for persistence, `build_*` for constructors) would reduce mental load. The pairing of `build_opening_card` and `build_opening_card_id` now reflects this symmetry in practice.

---

## 4. PGN Importer Workflow and Parsing Helpers

These items transform PGN text into stored openings and tactics.

- **`Importer` struct & methods (`new`, `with_in_memory_store`, `ingest_pgn_str`, `process_game`, `ensure_setup_requirement_for_fen_games`, `initialize_game_context`, `load_initial_board_from_optional_fen`, `store_opening_data_if_requested`, `finalize_tactic_if_requested`)** (`crates/chess-training-pgn-import/src/importer.rs`)
  - Drive the ingest pipeline, enforcing configuration (e.g., `[SetUp]` tags), tracking per-game state, and writing to storage.
  - *Related items:* `GameContext` and `MoveContext` methods handle per-move state. Method prefixes vary between `ensure_`, `initialize_`, `load_`, `store_`, `finalize_`; overall consistent with their responsibilities.

- **`ImportMetrics` & helpers (`note_*`)** (`crates/chess-training-pgn-import/src/importer.rs`)
  - Count inserted entities during import for reporting/testing.
  - *Related items:* Could align with `UpsertOutcome::is_inserted`; naming is consistent by using the `note_*` prefix.

- **`ImportError`, `IoError`, `ParseError`, `ConfigError`** (`crates/chess-training-pgn-import/src/errors.rs` & importer module)
  - Capture PGN parsing failures, IO errors, and configuration problems for higher-level handling.
  - *Related items:* `StoreError`, `SchedulerError`. Error naming is consistent with `*Error` suffix.

- **`GameContext::{record_starting_position, advance, into_tactic}`** & **`MoveContext::{new, execute_full_move_sequence, process_single_san_move, parse_san, convert_san_to_move}`** (`crates/chess-training-pgn-import/src/importer.rs`)
  - Manage in-game progression and SAN parsing.
  - *Related items:* `parse_games`, `parse_tag`, `sanitize_tokens`, `sanitize_token`, `load_fen`, `move_to_uci`, `board_to_ply`, `position_from_board`. Parsers use `parse_*` or `sanitize_*`, consistently reflecting their action.

**Naming observations for this group:**
- `with_in_memory_store` mirrors naming from other modules and aligns the importer helper with other `InMemory*Store` fixtures.
- `ensure_setup_requirement_for_fen_games` is long but descriptive; similar functions use `ensure_*`. All good.

---

## 5. Review Domain, Card Aggregates, and Grading Logic

These items encode review cards, states, and grade validation.

- **`Card<Id, Owner, Kind, State>`** (`crates/review-domain/src/card.rs`)
  - Generic container for any review card, storing ID, owner, payload, and mutable state.
  - *Related items:* `CardAggregate` (both specialized and generic) wrap this base struct.

- **`CardAggregate` (specialized) & `CardAggregate<Id, Owner, Opening, Tactic>` (generic)** (`crates/review-domain/src/card_aggregate.rs`)
  - Provide constructors (`new_opening`, `new_tactic`) and grade application for cards, either using default domain types or caller-supplied payloads.
  - *Naming concern:* Sharing the same type name for specialized and generic versions is confusing. Consider renaming the generic version to `GenericCardAggregate` or splitting into modules.

- **`CardKind<Opening, Tactic>` & helpers (`map_opening`, `map_tactic`, `as_ref`)** (`crates/review-domain/src/card_kind.rs`)
  - Classify cards as openings or tactics while providing transformation helpers.
  - *Related items:* Scheduler’s `CardKind` alias; names align well.

- **`StoredCardState`, `apply_review`, `next_interval`, etc.** (`crates/review-domain/src/card_state.rs`)
  - Maintain scheduling metadata and encapsulate SM-2 state transitions after each review.
  - *Related items:* `CardStateInvariants` and `CardStateInvariantError` ensure states remain valid.

- **Grade modules (`ValidGrade`, `GradeError`, `accuracy::is_correct`, `adjustments::to_grade_delta`, `conversions::from_u8/new/to_u8/as_u8`, `intervals::to_interval_increment`)** (`crates/review-domain/src/grade/*`)
  - Define valid review grades, conversions, and SM-2 adjustments.
  - *Related items:* Scheduler’s `ReviewOutcome` and `apply_sm2`. Naming is cohesive, though modules mix noun phrases (`adjustments`) with verbs (`conversions`).

- **`ReviewRequest`** (`crates/review-domain/src/review.rs`) and **`ReviewGrade`** (`crates/review-domain/src/review_grade.rs`)
  - Represent grade submissions and high-level descriptors for reviews.
  - *Related items:* `apply_review` functions in stores/scheduler.

- **`apply_sm2`, `update_ease`, `interval_for_grade`, `hard_interval`, `good_interval`, `easy_interval`, `scaled_interval`, `finalize_review`, `due_after_interval`, `state_after_grade`** (`crates/scheduler-core/src/sm2.rs`)
  - Execute SM-2 algorithm steps for scheduler cards.
  - *Related items:* `apply_review` functions elsewhere. Verbs are consistently `apply_` and `update_`.

- **`Sm2State::new`** (`crates/scheduler-core/src/domain/sm2_state.rs`) & **`ReviewOutcome`** (`crates/scheduler-core/src/domain/mod.rs`)
  - Encapsulate scheduler-specific SM-2 state and outcomes.
  - *Related items:* `StoredCardState` in review-domain. Naming parallels (SM-2 vs generic) are acceptable.

**Naming observations for this group:**
- The double `CardAggregate` definitions should be resolved for clarity.
- Grade conversion functions mix `to_` and `as_`. They follow Rust convention (`to_` for owned, `as_` for cheap), so keep as-is.

---

## 6. Scheduler Facade, Queue Building, and Unlock Flow

These items orchestrate SM-2 reviews, queue construction, and unlock tracking.

- **`Scheduler` struct & methods (`new`, `review`, `build_queue`, `into_store`)** (`crates/scheduler-core/src/scheduler.rs`)
  - Wrap the scheduler store and SM-2 config, exposing review execution and queue building.
  - *Related items:* `SchedulerFacade` (wasm), `SchedulerConfig`.

- **`build_queue_for_day`**, **`extend_queue_with_unlocks`**, **`skip_candidate`**, **`unlock_card`**, **`extract_prefix`**, **`ExistingUnlocks::{from_records, contains_prefix, contains_card, track_new_unlock}`** (`crates/scheduler-core/src/queue.rs`)
  - Assemble the daily review queue, merging due cards with unlocks and preventing duplicates.
  - *Related items:* `queue_length` in wasm calls into these helpers. Verb prefixes vary between `build_`, `extend_`, `skip_`, `unlock_`, which match their roles.

- **`queue_length`** (`crates/scheduler-wasm/src/scheduler.rs`) & **`build_queue_length`** (`crates/scheduler-wasm/src/bindings.rs`)
  - Provide wasm-friendly access to queue sizes.
  - *Related items:* `build_queue_for_day`. Mixed naming (`queue_length` vs. `build_queue_length`) could standardize on `queue_length`.

- **Unlock handling**
  - `SchedulerUnlockDetail`, `UnlockRecord` alias (`crates/scheduler-core/src/domain/mod.rs`), scheduler store methods (`record_unlock`, `unlock_candidates`), and wasm binding helpers (`default_config`, `init_panic_hook` for environment setup).
  - *Related items:* `insert_unlock_or_error` (card-store). Method names `record_*` vs. `insert_*` highlight cross-crate inconsistency.

**Naming observations for this group:**
- `build_queue_length` vs. `queue_length` is an easy win—rename the wasm binding to `queue_length` or `queue_size` for clarity.
- Scheduler store methods like `due_cards_for_owner` could align with card-store’s `collect_due_cards_for_owner` by picking either `due_cards` or `collect_due_cards` across crates.

---

## 7. Supporting Utilities and Re-exports

- **Module re-exports** (`crates/*/src/lib.rs`, `root/src/lib.rs`)
  - Expose internal modules for crate users, providing a single import point.
  - *Naming observation:* Most follow `pub use module::*;` patterns. Keep ensuring module names are nouns (e.g., `config`, `errors`).

- **`docs/rust-structs-glossary.md`** (referenced for guidance)
  - Already documents structs; this audit should be cross-referenced when updating naming.

---

## Cross-Cutting Naming Recommendations

1. **Unify “build/make/store/insert/record/upsert” verbs.**
   - Constructors: prefer `build_*` or `new_*`. Recent updates to `build_opening_card_id`/`build_tactic_card_id` follow this pattern; consider similarly renaming `build_opening_card` → `build_opening_card_payload` (if needed) and `make_input` closures in tests → `build_input` to match production code.
   - Persistence: reserve `upsert_*` for trait APIs, and ensure helpers underneath mirror the same verb (`store_canonical_position` → `upsert_canonical_position`).
   - Unlock operations: align on `record_unlock` (scheduler) or `insert_unlock` (card-store). Pick one and cascade.

2. **Differentiate similarly named traits/stores.**
   - Having two `CardStore` traits (card-store crate and scheduler-core crate) is confusing. Consider `ReviewCardStore` vs. `SchedulerStore` to clarify domain boundaries.

3. **Rename duplicated `CardAggregate`.**
   - Split into `StoredCardAggregate` (specialized) and `GenericCardAggregate` or move the generic type into a `generic` module to avoid import ambiguity.

4. **Standardize queue terminology.**
   - Use `queue_length` everywhere instead of mixing `build_queue` (verb) with `build_queue_length`. Perhaps expose `fn queue(owner, date)` returning the full vector and separate `fn queue_len`. Consistency helps API consumers.

5. **Harmonize importer store naming.**
   - ✅ Completed: `ImportInMemoryStore` is now `InMemoryImportStore`, and the importer helper was renamed to `with_in_memory_store` to align with other fixtures.

6. **Constructor verb consistency.**
   - Within domain models, prefer `::new_*` for specialized constructors (`CardAggregate::new_opening`, `SchedulerOpeningCard::new`). Avoid mixing `into_*` for builders unless performing conversions.

Implementing these changes would reduce cognitive overhead for new contributors and make the API more discoverable, especially when scanning for similarly named helpers during reviews.

