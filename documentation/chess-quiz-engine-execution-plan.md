# Atomic Chess Quiz Engine Execution Plan

This plan translates the chess quiz engine design brief and the surrounding repository conventions into a concrete sequence of deliverables. Each task documents the primary inputs we depend upon and the tangible outputs that signal completion. The tasks are ordered to support strict red–green-refactor development and to keep parallel contributors coordinated.

## 1. Finalise acceptance criteria and red tests ✅
- **Inputs:** `documentation/chess-quiz-engine.md` solution overview, repository TDD policy, existing PGN parsing behaviors in `crates/chess-training-pgn-import`.
- **Outputs:** A living checklist of acceptance criteria labelled AC1–AC4 (single-line PGN scope, retry policy, feedback messaging, adapter isolation) plus an ordered backlog of failing tests RT1–RT5 that map one-to-one to those behaviors (parser errors, retry exhaustion, summary math, feedback messaging coverage, adapter isolation guardrails). Published in `documentation/chess-quiz-engine.md` under “Acceptance Criteria Checklist” and “Initial Red Test Backlog”.

## 2. Scaffold the `quiz-core` crate and workspace wiring ✅
- **Inputs:** Workspace `Cargo.toml`, Makefile conventions, design decision to host adapters behind feature flags.
- **Outputs:** `crates/quiz-core` library with `engine`, `state`, `ports`, and `errors` modules stubbed; feature declarations for `cli`, `api`, and `wasm` with an empty default feature set; placeholder binaries under `src/bin/` compiled only when their feature is enabled via `#![cfg(feature = "...")]`; workspace manifests already glob `crates/*`, so no additional wiring was required.

## 3. Establish crate-level documentation and glossary placeholders ✅
- **Inputs:** Repository documentation standards (`README.md`, `docs/rust-structs-glossary.md`).
- **Outputs:** Added `crates/quiz-core/README.md` to describe architecture boundaries and feature gating; appended placeholder glossary entries for forthcoming types (`QuizEngine`, `QuizSession`, `QuizError`, `FeedbackMessage`) in `docs/rust-structs-glossary.md`, each marked “implementation pending” so downstream writers know what to expect.

## 4. Model quiz session state structures ✅
- **Inputs:** Design brief architecture section, existing domain patterns for immutable state.
- **Outputs:** Data structures such as `QuizSession`, `QuizStep`, `AttemptState`, and `QuizSummary` with documented fields for FEN snapshots, SAN prompts, retry counters, and cumulative scoring. Include serde derives where useful and unit tests that assert default/constructor invariants (failing first per TDD).

## 5. Implement PGN parsing and validation primitives ✅
- **Inputs:** `shakmaty` APIs, existing PGN normalisation strategies documented in `documentation/chess-quiz-engine.md`, parser error taxonomy requirement.
- **Outputs:** `source::QuizSource::from_pgn` normalises a single-game PGN string into SAN moves and a starting `Chess` position, backed by a richer `QuizError` enum (`UnreadablePgn`, `MultipleGames`, `VariationsUnsupported`, `WrongFormat`, `NoMoves`). Unit tests assert rejection of comments, variations, multiple games, and empty inputs while confirming successful parsing of well-formed PGN samples.

## 6. Wire quiz state initialisation and step hydration ✅
- **Inputs:** Outputs from Tasks 4 and 5.
- **Outputs:** `QuizSession::from_source` and `QuizSession::from_pgn` hydrate the session from parsed PGN data, producing ordered `QuizStep` entries with legal-board FEN snapshots and SAN prompts. Unit tests confirm FEN/SAN alignment for multi-move sequences and verify that unsupported features (variations, comments) surface the explicit parsing errors introduced earlier.

## 7. Define interaction ports and reference terminal adapter ✅
- **Inputs:** Port trait sketch in the design brief, repository feature-flagging conventions.
- **Outputs:** `ports::QuizPort` trait with prompt, feedback, and summary hooks alongside serialisable `PromptContext` and `FeedbackMessage` structs; a generically testable `TerminalPort` adapter behind the `cli` feature that wraps arbitrary `BufRead`/`Write` handles; and adapter-focused unit tests capturing stdout to verify prompt rendering, feedback wording, summary formatting, and the helper constructors so every branch of the module is exercised.

## 8. Build the quiz orchestration engine ✅
- **Inputs:** Session state types, port trait, retry policy (single retry) from acceptance criteria.
- **Outputs:** Implemented `QuizEngine` with constructors (`new`, `from_source`, `from_pgn`), the execution loop (`run`/`process_current_step`), and grading helpers that update attempts and summaries. Augmented unit tests with fake ports covering perfect runs, retry saves, exhausted retries, prompt context metadata, adapter summary publication, adapter failure propagation (prompt, feedback, summary), and attempt history capture for trimmed SAN submissions.

## 9. Harden error handling boundaries for adapters ✅
- **Inputs:** `QuizError` enum, adapter isolation requirement, prior error-handling tests.
- **Outputs:** Exhaustive conversions from lower-level errors (`shakmaty`, `std::io`) into `QuizError`; adapter-facing result aliases used across ports and the CLI implementation; regression tests for CLI I/O failures alongside existing retry-exhaustion and summary guard rails. Documentation updated to describe adapter-safe failure modes and the new error-conversion helpers.

## 10. Assemble integration tests for end-to-end quiz runs ✅
- **Inputs:** Engine implementation, terminal adapter, acceptance criteria backlog from Task 1.
- **Outputs:** Integration tests under `crates/quiz-core/tests/end_to_end.rs` orchestrate full quiz sessions with deterministic ports. Scenarios cover perfect runs, retries leading to success, failures after retries, and PGN parsing rejection, verifying adapter prompts, feedback, and summary delivery in one flow.

## 11. Update documentation and knowledge artifacts ✅
- **Inputs:** Implemented API surface, glossary placeholders, documentation obligations described in the brief.
- **Outputs:** Revised `documentation/chess-quiz-engine.md` capturing key decisions, current-state analysis, and implementation notes (see the "Current State" dashboard and refreshed architecture notes); updated glossary entries with full definitions and usage guidance; crate-level README tables illustrating adapter usage via the "Adapter quick reference"; changelog entry if the repository maintains one.

## 12. Plan follow-on integration work and backlog items ✅
- **Inputs:** Engine deliverables, dependencies on PGN importer, scheduler, and UI adapters noted in repository docs.
- **Outputs:** Documented backlog stories (CLI UX polish, API adapter, WASM embedding, telemetry hooks) and integration guidance captured in `documentation/chess-quiz-engine.md`. These notes align downstream teams on the adapter contracts exposed by `quiz-core` and the additional data the scheduler/card-store stacks require to consume quiz outcomes.

### Backlog stories queued for follow-up delivery
1. **CLI session runner.** Replace the placeholder `cli::run` entry point with PGN loading, engine construction, and terminal orchestration so product and pedagogy teams can perform manual smoke tests before other adapters land.
2. **HTTP API adapter.** Stand up an `api` feature that exposes the quiz engine over HTTP (likely Axum), translating the `QuizPort` trait into request/response handlers and wiring structured errors for the gateway to consume.
3. **WASM adapter for the web client.** Implement the `wasm` feature to surface quiz prompts, accept SAN submissions, and emit structured `FeedbackMessage` payloads that the existing web UI can render.
4. **Telemetry and analytics hooks.** Instrument the engine to emit attempt/summary events so the scheduler and analytics pipelines can track retry utilisation, streaks, and completion rates without scraping adapter logs.
5. **Card-store bridge.** Provide helpers that map `QuizStep` metadata into card-store DTOs, allowing curated openings to populate the scheduler queue with engine-authored quizzes instead of static flashcards.

### Integration guidance for scheduler and card-store teams
- **Scheduler expectations.** The scheduler requires quiz summaries that differentiate correct, incorrect, and retried attempts; downstream consumers should depend on `QuizSummary`'s `completed_steps`, `correct_answers`, `incorrect_answers`, and `retries_consumed` fields when computing unlock policies. Maintain stable field names so HTTP adapters can serialise them without ad hoc mapping.
- **Card-store normalisation.** Persist FEN positions, SAN prompts, and revealed solutions exactly as emitted by `QuizStep`. When enriching card-store records, attach a canonical identifier (e.g., `card_id` or PGN hash) so subsequent quiz runs can correlate learner history with scheduler unlocks.
- **Adapter contracts.** Any service embedding the engine must implement the `QuizPort` trait, honouring the prompt/feedback/summary flow and propagating `QuizError::Io` failures. Document how each adapter surface maps `PromptContext` and `FeedbackMessage` fields to its transport so new clients stay aligned.
- **Telemetry shape.** Emit events capturing `step_index`, learner responses, and retry consumption so scheduling algorithms and review dashboards can model difficulty. Prefer structured logs (JSON) or dedicated channels rather than parsing terminal output.

These tasks provide a clear pathway from design to a fully functioning `quiz-core` crate while keeping documentation, testing, and adapter isolation aligned with repository standards.
