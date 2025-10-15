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

## 10. Assemble integration tests for end-to-end quiz runs
- **Inputs:** Engine implementation, terminal adapter, acceptance criteria backlog from Task 1.
- **Outputs:** Integration tests under `crates/quiz-core/tests/` that orchestrate full quiz sessions with deterministic ports. Scenarios cover perfect runs, retries leading to success, failures after retries, and PGN parsing rejection. Tests start as red cases before the relevant code is implemented.

## 11. Update documentation and knowledge artifacts
- **Inputs:** Implemented API surface, glossary placeholders, documentation obligations described in the brief.
- **Outputs:** Revised `documentation/chess-quiz-engine.md` capturing key decisions and implementation notes; updated glossary entries with full definitions and code snippets; crate-level README diagrams or tables illustrating adapter usage; changelog entry if the repository maintains one.

## 12. Plan follow-on integration work and backlog items
- **Inputs:** Engine deliverables, dependencies on PGN importer, scheduler, and UI adapters noted in repository docs.
- **Outputs:** Documented backlog stories (e.g., CLI UX polish, API adapter, WASM embedding, telemetry hooks) captured in `docs/` or project management tooling. Include integration guidelines for the scheduler and card-store teams, highlighting any API contracts or data normalisation requirements identified during development.

These tasks provide a clear pathway from design to a fully functioning `quiz-core` crate while keeping documentation, testing, and adapter isolation aligned with repository standards.
