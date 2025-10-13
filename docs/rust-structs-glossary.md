# Rust Struct Glossary

This glossary summarizes the Rust structs defined across the chess-training repository. Each entry explains the big-picture purpose of the struct in plain English, why it exists, and points to related structs with similar responsibilities so reviewers can quickly compare options.

## Review and Scheduling Core

- **Card<Id, Owner, Kind, State>** (`crates/review-domain/src/card.rs`): Shared template for a study card that carries an identifier, the learner who owns it, what type of content it represents, and the mutable scheduling state. *Reason for including:* gives every subsystem a common shape for cards so data can flow between services without translation.  
  - Similar: **Sm2State** stores the scheduler-specific state inside this template, while **StoredCardState** plays the same role for the persistence layer.
- **StoredCardState** (`crates/review-domain/src/card_state.rs`): Tracks when a stored card is due, how difficult it is, and streak information used to schedule future reviews. *Reason for including:* lets storage backends remember when to next show a card without recalculating from scratch.  
  - Similar: **Sm2State** mirrors these scheduling fields inside the scheduler runtime.
- **CardStateInvariants** (`crates/review-domain/src/card_state/invariants.rs`): Describes the rules that a `StoredCardState` must obey, such as minimum intervals and valid ease ranges. *Reason for including:* provides a single place to validate card state before it is persisted or acted upon.  
  - Similar: **SchedulerConfig** sets comparable bounds used by the scheduler when updating `Sm2State`.
- **UnlockRecord<Owner, Detail>** (`crates/review-domain/src/unlock.rs`): Records the learner, payload, and date for newly unlocked material. *Reason for including:* allows systems to audit unlock history and avoid unlocking the same content twice.  
  - Similar: **SchedulerUnlockDetail** supplies the scheduler-specific payload carried inside these records.
- **UnlockDetail** (`crates/review-domain/src/unlock.rs`): Minimal payload describing which opening edge was unlocked. *Reason for including:* captures the chess-specific context that accompanies an unlock event.  
  - Similar: **SchedulerUnlockDetail** adds richer metadata for the scheduler, and **OpeningCard** refers to the same edge identifier during reviews.
- **ReviewRequest** (`crates/review-domain/src/review.rs`): Simple request body for recording that a learner reviewed a card on a given day with a grade. *Reason for including:* standardizes how services submit review outcomes to storage.  
  - Similar: **ReviewOutcome** reflects the same information after the scheduler processes a review.
- **SchedulerConfig** (`crates/scheduler-core/src/config.rs`): Tunable parameters for the SM-2 scheduler such as ease factor limits and learning steps. *Reason for including:* centralizes the knobs that control scheduling behavior.  
  - Similar: **IngestConfig** holds import-time toggles, and **SchedulerConfigPatch** updates these values for the WebAssembly facade.
- **Sm2State** (`crates/scheduler-core/src/domain/sm2_state.rs`): Runtime scheduling data for a card, including due dates, ease, interval, lapse count, and total reviews. *Reason for including:* stores the SM-2 state that the scheduler adjusts after each review.  
  - Similar: **StoredCardState** persists comparable fields in the card-store layer.
- **SchedulerOpeningCard** (`crates/scheduler-core/src/domain/card_kind.rs`): Payload tagging a card as part of an opening line and recording its parent prefix. *Reason for including:* lets the scheduler group opening cards so it can control unlock pacing.  
  - Similar: **OpeningCard** in the review domain carries the same edge identifier when cards reach storage.
- **SchedulerTacticCard** (`crates/scheduler-core/src/domain/card_kind.rs`): Marker struct for tactic cards in the scheduler. *Reason for including:* keeps room for future tactic metadata while distinguishing them from openings today.  
  - Similar: **TacticCard** in the review domain signals the same concept downstream.
- **SchedulerUnlockDetail** (`crates/scheduler-core/src/domain/mod.rs`): Scheduler-specific payload nested inside `UnlockRecord`, storing the unlocked card ID and its parent opening prefix if any. *Reason for including:* helps the scheduler remember which prefixes already unlocked content each day.  
  - Similar: **ExistingUnlocks** keeps the same information in-memory while building a queue.
- **ReviewOutcome** (`crates/scheduler-core/src/domain/mod.rs`): Returned after the scheduler processes a review, containing the updated card, previous due date, and the grade used. *Reason for including:* packages the essential facts a caller needs to react to a review result.  
  - Similar: **ReviewRequest** captures the input side of the same workflow.
- **InMemoryStore** (`crates/scheduler-core/src/store.rs`): Reference implementation of the `CardStore` trait using in-memory collections. *Reason for including:* offers a lightweight storage option for tests and examples.  
  - Similar: **ImportInMemoryStore** provides an analogous in-memory backend for the PGN importer.
- **ExistingUnlocks** (`crates/scheduler-core/src/queue.rs`): Helper that tracks which cards and opening prefixes were already unlocked for the day to prevent duplicates. *Reason for including:* enforces the scheduler’s “one opening per prefix per day” rule while building queues.  
  - Similar: **SchedulerUnlockDetail** stores the same identifiers inside persisted unlock logs.
- **Scheduler<S: CardStore>** (`crates/scheduler-core/src/scheduler.rs`): High-level coordinator that applies SM-2 reviews and builds daily queues against a storage backend. *Reason for including:* encapsulates scheduling behavior so consumers only provide storage and configuration.  
  - Similar: **SchedulerFacade** wraps this scheduler for WebAssembly clients.

## Integrations and Facades

- **SchedulerFacade** (`crates/scheduler-wasm/src/scheduler.rs`): Lightweight wrapper around the scheduler paired with its configuration, shared by unit tests and the wasm bindings. *Reason for including:* exposes a stable interface that can be called from JavaScript without leaking scheduler internals.  
  - Similar: **WasmScheduler** owns a `SchedulerFacade` when compiled to WebAssembly.
- **WasmScheduler** (`crates/scheduler-wasm/src/bindings.rs`): wasm-bindgen struct exposed to JavaScript, handling configuration conversion and queue-length queries. *Reason for including:* bridges the Rust scheduler with browser or Node environments.  
  - Similar: **SchedulerFacade** provides the underlying functionality, and **SchedulerConfigDto** represents configuration snapshots sent across the boundary.
- **SchedulerConfigDto** (`crates/scheduler-wasm/src/config.rs`): Serializable view of `SchedulerConfig` that JavaScript clients can consume. *Reason for including:* transmits scheduler settings across the wasm interface in a friendly format.  
  - Similar: **SchedulerConfigPatch** applies partial updates in the opposite direction.
- **SchedulerConfigPatch** (`crates/scheduler-wasm/src/config.rs`): Partial update object decoded from JavaScript to override selected scheduler settings. *Reason for including:* lets the wasm API accept user tweaks without requiring every field.  
  - Similar: **SchedulerConfig** holds the full set of fields that this patch mutates.

## Opening and Tactic Content Models

- **ChessPosition** (`crates/review-domain/src/position.rs`): Canonical representation of a chess position derived from a FEN string, including the side to move and ply count. *Reason for including:* supplies a stable ID and metadata whenever the system references a board position.  
  - Similar: **Position** in the PGN importer mirrors this idea when hashing imported positions.
- **Repertoire** (`crates/review-domain/src/repertoire/repertoire_.rs`): Collection of repertoire moves under a human-friendly name. *Reason for including:* gives learners and services a packaged view of the opening lines they are studying.  
  - Similar: **RepertoireBuilder** streamlines constructing the same data, and **RepertoireEdge** links the repertoire to stored edges.
- **RepertoireBuilder** (`crates/review-domain/src/repertoire/repertoire_.rs`): Fluent helper for assembling a `Repertoire` with a chosen name and move list. *Reason for including:* makes it easier for callers to build repertoires without managing vectors manually.  
  - Similar: **RepertoireMove** instances populate the move list it produces.
- **RepertoireMove** (`crates/review-domain/src/repertoire/move_.rs`): Represents an individual opening move with parent/child positions and notation in UCI and SAN. *Reason for including:* captures the minimum information needed to navigate an opening tree.  
  - Similar: **OpeningEdge** stores equivalent details once the move is normalized for storage.
- **OpeningEdge** (`crates/review-domain/src/opening/edge.rs`): Canonical edge in the opening tree with identifiers and move notation. *Reason for including:* provides a normalized record that can be shared across services.  
  - Similar: **OpeningEdgeRecord** adds source metadata during PGN import, and **EdgeInput** is the user-facing request to create one.
- **EdgeInput** (`crates/review-domain/src/opening/edge_input.rs`): Input payload for creating or updating an opening edge before normalization. *Reason for including:* lets clients submit moves without knowing the deterministic edge ID.  
  - Similar: **OpeningEdge** is the normalized output produced from this payload.
- **OpeningCard** (`crates/review-domain/src/opening/card.rs`): Minimal payload identifying which opening edge a review card covers. *Reason for including:* ties scheduled reviews back to the specific opening move.  
  - Similar: **SchedulerOpeningCard** tracks the same concept while cards live in the scheduler.
- **TacticCard** (`crates/review-domain/src/tactic.rs`): Minimal payload naming the tactic a review card is about. *Reason for including:* distinguishes tactic reviews from opening reviews in storage.  
  - Similar: **SchedulerTacticCard** handles the scheduler-side representation.
- **Position** (`crates/chess-training-pgn-import/src/model.rs`): Imported position record with deterministic hashing, stored during PGN ingest. *Reason for including:* ensures each unique board state receives a stable ID in the import database.  
  - Similar: **ChessPosition** fills the same role within the review domain proper.
- **OpeningEdgeRecord** (`crates/chess-training-pgn-import/src/model.rs`): Combines a canonical `OpeningEdge` with optional source metadata from the PGN. *Reason for including:* keeps analytics-friendly context when storing opening moves discovered during import.  
  - Similar: **OpeningEdge** is the shared structure embedded inside this record.
- **RepertoireEdge** (`crates/chess-training-pgn-import/src/model.rs`): Links an owner and repertoire key to an opening edge ID. *Reason for including:* records which imported moves belong to which learner’s repertoire.  
  - Similar: **Repertoire** groups the same relationships at a higher level.
- **Tactic** (`crates/chess-training-pgn-import/src/model.rs`): Imported tactic opportunity with hashed ID, FEN, move sequence, tags, and optional source hint. *Reason for including:* captures tactic data generated during PGN ingestion so it can later become review material.  
  - Similar: **TacticCard** references these tactics once they become review cards.
- **PositionId, EdgeId, MoveId, CardId, LearnerId, UnlockId** (`crates/review-domain/src/ids.rs`): Lightweight wrappers around `u64` identifiers that enforce type safety across the review domain. *Reason for including:* prevent mix-ups between different ID types while still being cheap to copy.  
  - Similar: **Uuid**-based identifiers in the scheduler (`Card` IDs, owner IDs) provide the same safety in another module.

## Import and Storage Infrastructure

- **InMemoryCardStore** (`crates/card-store/src/memory/in_memory_card_store.rs`): Thread-safe in-memory implementation of the review card storage trait, using locks around hash maps and sets. *Reason for including:* enables local development and testing without provisioning a database.  
  - Similar: **ImportInMemoryStore** offers an in-memory store for the PGN importer.
- **StorageConfig** (`crates/card-store/src/config.rs`): Configuration options for card-store backends such as connection limits and batch sizes. *Reason for including:* captures operational knobs needed when deploying a persistent store.  
  - Similar: **IngestConfig** supplies analogous toggles for the importer.
- **ImportMetrics** (`crates/chess-training-pgn-import/src/importer.rs`): Counters for how many positions, edges, repertoire entries, and tactics were inserted during an import run. *Reason for including:* provides visibility into what the importer accomplished.  
  - Similar: **ImportInMemoryStore** exposes accessors so these metrics can be validated in tests.
- **Importer<S: Storage>** (`crates/chess-training-pgn-import/src/importer.rs`): High-level PGN ingestion engine that orchestrates parsing games, updating storage, and tracking metrics. *Reason for including:* centralizes the logic for turning PGN text into structured training data.  
  - Similar: **Scheduler<S>** plays a comparable orchestration role for scheduling instead of importing.
- **GameContext** (`crates/chess-training-pgn-import/src/importer.rs`): Internal tracker for the state of a single PGN game as it is processed, including the board, ply, and whether to record tactics. *Reason for including:* keeps per-game state organized while the importer walks through moves.  
  - Similar: **MoveContext** focuses on a single move transition inside this process.
- **MoveContext** (`crates/chess-training-pgn-import/src/importer.rs`): Holds the UCI string, resulting board, and ply after playing one move. *Reason for including:* packages the per-move data needed to update `GameContext`.  
  - Similar: **OpeningEdgeRecord** stores comparable move information once it is ready for persistence.
- **RawGame** (`crates/chess-training-pgn-import/src/importer.rs`): Simplified representation of a parsed PGN game with tags and SAN tokens before validation. *Reason for including:* acts as the staging format between raw PGN text and structured import logic.  
  - Similar: **GameContext** consumes this data to build normalized records.
- **ImportInMemoryStore** (`crates/chess-training-pgn-import/src/storage.rs`): In-memory implementation of the importer’s `Storage` trait using B-tree collections. *Reason for including:* allows fast iteration on the importer without external storage dependencies.  
  - Similar: **InMemoryCardStore** serves the same purpose for review storage.
- **IoError** (`crates/chess-training-pgn-import/src/errors.rs`): Describes a file system failure that happened while loading importer configuration, including the path and underlying error. *Reason for including:* surfaces actionable diagnostics when configuration files cannot be read.  
  - Similar: **ParseError** reports issues when the file contents are invalid.
- **ParseError** (`crates/chess-training-pgn-import/src/errors.rs`): Wraps a TOML parsing failure encountered while loading importer configuration. *Reason for including:* distinguishes parsing issues from file access problems.  
  - Similar: **IoError** handles the complementary I/O failure scenario.
- **IngestConfig** (`crates/chess-training-pgn-import/src/config.rs`): Runtime switches controlling how PGN ingestion should behave, such as whether to extract tactics and maximum variation depth. *Reason for including:* lets operators tailor the importer’s behavior without code changes.  
  - Similar: **StorageConfig** gives the same flexibility to storage backends.
- **FileConfig** (`crates/chess-training-pgn-import/src/config.rs`): Internal struct for deserializing TOML configuration files before merging them into `IngestConfig`. *Reason for including:* isolates config-file parsing so CLI and file inputs can be combined cleanly.  
  - Similar: **CliArgs** captures the same settings when supplied via command line.
- **CliArgs** (`crates/chess-training-pgn-import/src/config.rs`): Representation of command-line arguments accepted by the importer, including input paths and feature toggles. *Reason for including:* provides a structured output from clap so the importer can build its configuration.  
  - Similar: **IngestConfig** is the final consolidated configuration these arguments influence.

## Testing and Tooling Helpers

- **IssuePayload** (`tests/run_with_error_logging.rs`): Minimal struct used in integration tests to deserialize the JSON payload the CI error-logging script would submit. *Reason for including:* lets the test assert that the GitHub issue body and title are populated correctly.  
  - Similar: **ImportMetrics** is another test-checked struct that reports what happened during importer runs.
- **RelearningFixture** (`crates/scheduler-core/tests/scheduler_sm2.rs`): Helper fixture bundling a scheduler instance, a prepared card, and a reference date for SM-2 tests. *Reason for including:* keeps repetitive setup code out of individual scheduler tests.  
  - Similar: **TimedStore** in opening scheduling tests provides a tailored test double for different scenarios.
- **TimedStore** (`crates/scheduler-core/tests/opening_scheduling.rs`): Custom in-memory `CardStore` used in tests to control card availability by day. *Reason for including:* enables deterministic tests around unlock timing and ordering.  
  - Similar: **InMemoryStore** is the production-ready in-memory store used outside these specialized tests.
