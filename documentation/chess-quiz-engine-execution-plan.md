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

## 6. Wire quiz state initialisation and step hydration
- **Inputs:** Outputs from Tasks 4 and 5.
- **Outputs:** Logic that consumes parsed PGN data to populate the session state sequence (initial board plus per-move prompts). Includes tests ensuring correct SAN/FEN generation and that unsupported features (variations, comments) surface explicit errors.

## 7. Define interaction ports and reference terminal adapter
- **Inputs:** Port trait sketch in the design brief, repository feature-flagging conventions.
- **Outputs:** `ports::QuizPort` trait and companion message types (e.g., `FeedbackMessage`, `PromptContext`), plus a `TerminalPort` implementation behind the `cli` feature that can drive manual smoke tests. Provide adapter-focused unit tests using mock/stdout capturing to exercise the trait contract.

## 8. Build the quiz orchestration engine
- **Inputs:** Session state types, port trait, retry policy (single retry) from acceptance criteria.
- **Outputs:** `QuizEngine` implementation with constructors (`from_pgn`, `from_source`), the main execution loop (`run`), and helper methods (`advance`, `grade_attempt`). Tests simulate correct/incorrect answers, retry flows, and summary aggregation using fake ports.

## 9. Harden error handling boundaries for adapters
- **Inputs:** `QuizError` enum, adapter isolation requirement, prior error-handling tests.
- **Outputs:** Exhaustive conversions from lower-level errors (`shakmaty`, `std::io`) into `QuizError`; result aliases for adapter ergonomics; tests covering I/O failures, retry exhaustion, and summary edge cases. Update documentation to describe adapter-safe failure modes.

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
