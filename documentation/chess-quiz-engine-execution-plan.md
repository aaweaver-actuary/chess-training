# Chess Quiz Engine MVP Execution Plan

This plan converts the current implementation described in
`documentation/chess-quiz-engine.md` into a concrete, test-first backlog that
carries the crate to a fully usable MVP. Each task is intentionally atomic: it
can be implemented, reviewed, and merged without blocking on the others. Tasks
reference the modules that will change and call out the verifications required
before they are considered complete.

## Stabilise the core feedback loop

### [T1] Align retry messaging with consumed allowances
- **Objective:** Ensure `FeedbackMessage::retry` and terminal output report the
  number of retries remaining *after* the current miss so learners receive
  accurate guidance.
- **Primary inputs:** `crates/quiz-core/src/engine.rs`
  (`grade_attempt`), `crates/quiz-core/src/cli.rs` (`TerminalPort::publish_feedback`).
- **Deliverables:** Update retry bookkeeping so the attempt state increments
  before generating retry feedback. Adjust `FeedbackMessage` constructors if
  required and extend unit tests to assert the new count. Confirm the terminal
  adapter prints the corrected allowance.
- **Verification:** Red tests in `engine` module covering exhausted retries and
  terminal adapter tests confirming the displayed counts. No dependencies on
  other tasks.

### [T2] Accept equivalent SAN notations during grading
- **Objective:** Prevent false negatives when learners include optional suffixes
  like `+`, `#`, or annotation glyphs by normalising SAN comparison.
- **Primary inputs:** `crates/quiz-core/src/engine.rs` (`san_matches` helper)
  and associated tests.
- **Deliverables:** Normalise learner responses using `shakmaty::San` parsing or
  by stripping optional suffix markers prior to comparison. Document the
  behaviour on `FeedbackMessage` or engine docs.
- **Verification:** Unit tests that prove `Nf3+` and `Nf3` (and other annotated
  variants) are treated as equivalent. Independent from other tasks.

## Expand session context and fidelity

### [T3] Introduce durable step identifiers and metadata ✅
- **Status:** Completed via `feat(quiz-core): add durable step metadata surfaces`.
- **Outcome:** `QuizStep` exposes a `StepMetadata` payload (step IDs, theme tags,
  spaced-repetition card IDs) that now propagates into `PromptContext` and
  `FeedbackMessage`. The terminal adapter renders the metadata and tests assert
  fake ports capture it.
- **Primary inputs:** `crates/quiz-core/src/state.rs` (`QuizStep`, `StepMetadata`),
  `crates/quiz-core/src/ports.rs`, `crates/quiz-core/src/engine.rs`,
  `crates/quiz-core/src/cli.rs`.
- **Verification:** Unit tests in `state`, `engine`, and `ports` modules confirm
  metadata hydration, propagation, and adapter output.

### [T4] Preserve PGN annotations and surface them in feedback
- **Objective:** Carry commentary and move-level annotations from the source PGN
  into `QuizStep::annotations` so adapters can display coaching notes alongside
  feedback.
- **Primary inputs:** `crates/quiz-core/src/source.rs`, `crates/quiz-core/src/state.rs`,
  `crates/quiz-core/src/ports.rs`.
- **Deliverables:** Enhance PGN parsing to capture supported annotations (e.g.,
  `{}` comments or glyphs) without reintroducing unsupported variation lines.
  Populate `QuizStep::annotations` during hydration and ensure `FeedbackMessage`
  retains them. Update unit and integration tests to cover annotated PGN cases.
- **Verification:** Red tests in `source` and `state` modules proving annotated
  PGNs hydrate correctly, plus adapter tests asserting notes are displayed.
  Independent of [T3] but complementary.

## Ship a runnable terminal MVP

### [T5] Replace the CLI stub with an interactive runner
- **Objective:** Turn `cli::run` into a thin binary that reads PGN input,
  constructs a `QuizEngine`, and streams prompts/feedback through the
  `TerminalPort`.
- **Primary inputs:** `crates/quiz-core/src/cli.rs`, `src/bin/` entry point if
  required, and repository CLI conventions.
- **Deliverables:** Implement argument parsing or stdin ingestion for PGN text,
  wire `QuizEngine::from_pgn`, and expose an executable compiled behind the
  `cli` feature. Include smoke tests using buffered handles to assert a full
  session run succeeds.
- **Verification:** Integration-style tests in `crates/quiz-core/tests` that run
  the CLI harness with deterministic input/output transcripts. Task is
  independent of [T1]–[T4].

## Provide service adapters

### [T6] Deliver the HTTP API adapter
- **Objective:** Fulfil the promised `api` feature by exposing the quiz engine
  over HTTP for other services to consume.
- **Primary inputs:** `crates/quiz-core/src/api.rs`, `Cargo.toml` feature flags,
  workspace API conventions (e.g., Axum or similar frameworks).
- **Deliverables:** Implement request/response structs that wrap the
  `QuizPort` contract, provide a reference Axum (or equivalent) router, and add
  adapter-focused tests mocking the port to verify error propagation and JSON
  payloads. Ensure the feature compiles cleanly when disabled.
- **Verification:** Unit tests within the API module and, if applicable,
  superstructure tests that spin up the router with in-memory state. Runs
  independently once [T3]–[T4] have established the metadata it returns (no hard
  dependency, but align payloads if those tasks are complete).

### [T7] Implement the WASM adapter for the web client
- **Objective:** Provide a browser-friendly adapter under the `wasm` feature so
  the existing web UI can embed the engine without server round-trips.
- **Primary inputs:** `crates/quiz-core/src/wasm.rs`, WASM build tooling in the
  repository.
- **Deliverables:** Expose a minimal WASM API that feeds prompts, collects SAN
  answers, and publishes feedback/summary messages as serialisable structs.
  Include JS glue examples and WASM-targeted tests (wasm-bindgen test harness or
  equivalent) to validate the bindings.
- **Verification:** WASM tests/build checks executed via the repository's
  toolchain. Task operates independently of [T6].

## Add operational instrumentation

### [T8] Emit structured telemetry from the engine
- **Objective:** Capture learner attempts, retry consumption, and summary events
  so downstream analytics can consume them without scraping adapter logs.
- **Primary inputs:** `crates/quiz-core/src/engine.rs`, potential telemetry sink
  traits in `crates/quiz-core/src/ports.rs` or a new module.
- **Deliverables:** Introduce a lightweight telemetry trait or callback invoked
  during `process_current_step`, emit structured events for prompt, attempt, and
  summary transitions, and provide default no-op implementations for adapters
  that opt out. Extend tests to assert events fire in the expected order.
- **Verification:** Unit tests around the engine and fake telemetry sink plus
  documentation of the event schema. Task can proceed in parallel with adapter
  work.

## Documentation and enablement

### [T9] Update documentation and glossary as features land
- **Objective:** Keep reference materials aligned with the MVP feature set so
  downstream teams have accurate integration guidance.
- **Primary inputs:** `documentation/chess-quiz-engine.md`, `crates/quiz-core/README.md`,
  `docs/rust-structs-glossary.md`.
- **Deliverables:** For each completed task above, refresh the design brief,
  adapter README sections, and glossary entries to reflect new fields, adapters,
  and telemetry. Capture CLI/API usage examples as they stabilise.
- **Verification:** Documentation diffs reviewed alongside feature PRs; no code
  dependencies.

These tasks collectively bridge the gap from the current crate capabilities to a
fully featured MVP while preserving the repository's red–green–refactor cadence
and adapter isolation guarantees.
