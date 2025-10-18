# Chess Quiz Engine Design Brief

## Current State

### 1. Current state of our most basic quiz engine
- `QuizEngine::run` now drives fully hydrated `QuizSession` instances, looping until each `QuizStep` is graded and publishing a `QuizSummary` through the active `QuizPort`.【F:crates/quiz-core/src/engine.rs†L1-L117】
- Session hydration is sourced from PGN input via `QuizSource::from_pgn`, guaranteeing single-game main lines and producing SAN prompts with matching FEN board snapshots for every move.【F:crates/quiz-core/src/source.rs†L1-L81】【F:crates/quiz-core/src/state.rs†L1-L121】
- Adapter communication is mediated entirely through serialisable `PromptContext` and `FeedbackMessage` structures, and the terminal adapter implements the full prompt/feedback/summary contract behind the `cli` feature flag.【F:crates/quiz-core/src/ports.rs†L1-L108】【F:crates/quiz-core/src/cli.rs†L1-L114】
- Error handling is centralised in `QuizError`, with conversions from PGN parsing and I/O failures so adapter code can rely on `QuizResult` aliases without bespoke plumbing.【F:crates/quiz-core/src/errors.rs†L1-L72】

### 2. Work effort still required for an mvp
- Integrate annotations into hydration—`QuizStep::annotations` is currently empty because `QuizSource` normalises SAN but does not preserve instructional commentary; we need a data source for move-level notes.【F:crates/quiz-core/src/state.rs†L60-L78】【F:crates/quiz-core/src/source.rs†L47-L81】
- Replace the placeholder `cli::run` stub with wiring that loads PGN input, constructs a `QuizEngine`, and streams interaction through `TerminalPort` for manual playtesting.【F:crates/quiz-core/src/cli.rs†L116-L118】
- Deliver non-CLI adapters (API, WASM) or remove their feature flags until implementations land, and provide integration hooks for downstream scheduling/card systems (captured in Task 12 of the execution plan).【F:documentation/chess-quiz-engine-execution-plan.md†L63-L95】

### 3. Issues surfacing during implementation that do not appear to have been considered during planning
- Retry feedback currently reports the pre-attempt allowance because `grade_attempt` captures `remaining_retries` before incrementing `retries_used`, leading the terminal adapter to overstate how many chances remain; we need to revise the helper to reflect the post-attempt state.【F:crates/quiz-core/src/engine.rs†L80-L105】【F:crates/quiz-core/src/cli.rs†L69-L99】
- `san_matches` performs a case-insensitive comparison but otherwise expects exact SAN tokens, so equivalent notations that differ by suffixes (e.g., `Nf3+` vs `Nf3`) are graded as incorrect; we should revisit normalisation rules if annotations or PGN cleaning add symbols back in.【F:crates/quiz-core/src/engine.rs†L131-L160】

### 4. Any suggestions for adjustments based on those issues?
- Amend `FeedbackMessage::retry` or the call site to compute remaining retries after incrementing `retries_used`, ensuring adapters communicate accurate expectations before a learner's second attempt.【F:crates/quiz-core/src/engine.rs†L80-L105】
- Extend `san_matches` to tolerate optional suffix markers (e.g., strip trailing `+`, `#`, or match via `San::from_ascii`) so learners are not penalised for including check indicators that were pruned during PGN normalisation.【F:crates/quiz-core/src/engine.rs†L131-L160】
## Role We Are Supporting
We are acting as the core infrastructure team for the chess-training workspace. Our responsibility is to design a reusable quiz
engine that other product surfaces—CLI tools, web experiences, mobile apps, or background services—can embed without depending o
n any particular front end. That means our primary deliverable is a well-factored Rust crate that exposes a stable API, clearly
expressed boundaries, and adapters that can be swapped or extended as new surfaces appear.

Key expectations for the role:

- Champion strict TDD so that every behavior is documented by tests before implementation.
- Preserve the repository convention of feature-gated adapters to keep compilation fast and focused.
- Document structures, flows, and error handling so downstream teams can integrate confidently.

## Solution Overview
The workspace now ships with the `quiz-core` crate. The implementation focuses on three responsibilities:

1. **Parsing & validation** – Interpret a single PGN line, validate the format, and surface rich errors for misformatted data.
2. **Quiz orchestration** – Drive the move-by-move loop (board snapshot, prior move, prompt, scoring, annotations, retries).
3. **Adapter isolation** – Provide ports that drive user interaction or external notifications, leaving concrete I/O to feature-
   gated binaries (`cli`, `api`, `wasm`).

The resulting crate is deliberately lean: adapters can be compiled out entirely, and the engine can be embedded in tests or asyn
c contexts. A high-level flow is captured below:

```mermaid
flowchart TD
    A[Load PGN] --> B[Validate Format]
    B -->|ok| C[Initialize Session State]
    C --> D[Render Board & Prior Move]
    D --> E[Prompt User via Port]
    E --> F{Answer Correct?}
    F -->|Yes| G[Emit Feedback & Annotations]
    G --> H[Advance to Next Move]
    F -->|Retry| I[Give Second Chance]
    I -->|Incorrect| J[Reveal Correct Move]
    H -->|End of Line| K[Return Summary]
```

## Acceptance Criteria Checklist

Each acceptance criterion carries a stable identifier so red tests can reference the specific behavior they lock down.

- [x] **[AC1] Single-line PGN scope is enforced.** The engine only accepts PGN strings that describe a single game presented as one main line without comments, annotations, or nested variations. Inputs that include multiple games, line breaks with alternate lines, or unsupported metadata must surface targeted `QuizError` variants so adapters can relay actionable feedback. Normalisation work already available in `crates/chess-training-pgn-import` must be reused instead of re-implementing parsing logic.
- [x] **[AC2] Retry policy allows exactly one additional attempt per move.** When a learner submits an incorrect answer the engine must prompt the port for one—and only one—retry before revealing the correct SAN. Exhausted retries mark the step as incorrect, advance the session automatically, and increment retry counters captured in the session summary.
- [x] **[AC3] Feedback messaging captures correctness and annotations.** Each engine decision results in a `FeedbackMessage` delivered through the active `QuizPort`. Correct attempts report success alongside any annotations configured for the move. Incorrect answers must communicate failure reasons (wrong SAN, retry exhausted) and, after the final attempt, include the authoritative move so adapters can render the teaching moment.
- [x] **[AC4] Adapter isolation remains intact.** All user interaction flows through the `QuizPort` trait so adapters can live behind feature flags (`cli`, `api`, `wasm`). Engine code must stay free of direct `std::io` usage, expose deterministic error types for adapters to translate, and provide documentation hooks so downstream teams understand the boundary contract.

## Initial Red Test Backlog

1. **`[RT1] pgn_rejects_non_single_line_games`** – Covers [AC1]. Feed `QuizSource::from_pgn` examples containing multiple games, PGN comments, or variation markers and assert the precise `QuizError` variant. This validates the single-line scope before any engine orchestration work begins.
2. **`[RT2] engine_limits_retry_attempts`** – Covers [AC2]. Simulate a quiz session where the learner answers a move incorrectly twice and verify that the engine surfaces retry exhaustion, records the miss, and emits the reveal feedback. Establishing this guard ensures future features cannot regress the retry contract.
3. **`[RT3] summary_counts_correct_and_retry_totals`** – Supports [AC2]. Drive a short session with a mix of correct answers, single-retry saves, and final misses to assert the `QuizSummary` math (correct/incorrect counts, retry tally, move index progression). Locking the summary rules early protects downstream analytics integrations.
4. **`[RT4] feedback_messages_reflect_outcomes`** – Covers [AC3]. Exercise correct answers, retry saves, and exhausted attempts to assert the emitted `FeedbackMessage` contents (success flag, annotations, failure reasons, reveal move). Prevents regressions in learner-facing messaging.
5. **`[RT5] engine_remains_adapter_isolated`** – Covers [AC4]. Drive the engine against a fake port that records method invocations, asserting that all external interactions flow through the trait contract and that feature-gated adapters can be swapped without leaking `std::io` dependencies.

## Architecture
The architecture mirrors other workspace crates that separate pure logic from delivery concerns.

- `engine` – Stateless functions and structs that govern quiz progression, retries, scoring, and annotations.
- `state` – Data structures that track board state, progress, and cumulative summary results.
- `ports` – Traits describing how the engine communicates with the outside world (prompts, answers, telemetry, logging).
- `errors` – Error types powered by `thiserror` to model parsing failures, illegal formats, and unsupported PGN features.
- Feature-gated binaries under `src/bin/` that compile only when their respective feature flag is active.

Adapters depend on the engine via the `ports` traits, while the engine never touches `std::io` directly. The reference `TerminalPort` demonstrates how buffered handles can be swapped during tests to keep the engine ergonomic for async or embedded environments.【F:crates/quiz-core/src/ports.rs†L1-L108】【F:crates/quiz-core/src/cli.rs†L1-L115】

### Session State Model
The `state` module now codifies the data exchanged between the engine and adapters:

- `QuizSession` aggregates ordered `QuizStep` entries, tracks the active index, and mirrors
  `QuizSummary` totals so adapters can serialise a complete snapshot at any point.
- `QuizStep` pairs a FEN board snapshot with the SAN prompt and solution, embedding an
  `AttemptState` that records retries and learner responses alongside optional annotations.
- `AttemptState` and its `AttemptResult` enum capture retry allowances, responses, and the final
  outcome so retry policies remain enforceable without leaking implementation details to adapters.
- `QuizSummary` tallies correct, incorrect, and retry counts, giving the orchestration layer a
  single structure to update as the session advances.
- `QuizSession::from_source` and `QuizSession::from_pgn` hydrate session steps from a validated
  PGN source, ensuring each `QuizStep` carries the legal-board FEN snapshot and canonical SAN
  prompt while bubbling up any parsing errors for adapters to display.

### Engine Run Loop

`QuizEngine::run` iterates through each `QuizStep`, building a `PromptContext` for the active move and delegating user interaction to the injected `QuizPort`. Responses are graded through `grade_attempt`, which enforces the single-retry policy, records attempt history, and produces the appropriate `FeedbackMessage`. Once a step resolves as correct or incorrect the engine advances the session, updates `QuizSummary` totals (including retries consumed), and continues until every move is processed before presenting the final summary through the adapter.【F:crates/quiz-core/src/engine.rs†L12-L117】

### Adapter Failure Handling

The fake-port test harness now exercises error propagation for every adapter touchpoint. When
`present_prompt`, `publish_feedback`, or `present_summary` return `QuizError::Io`, the engine bubbles
the error immediately so adapters can recover, while leaving session and summary state in the last
consistent position (e.g., completed steps remain counted even if summary presentation fails).
Additional assertions cover attempt history capture, ensuring trimmed SAN responses are recorded in
order even when learners take a retry before submitting the correct move.

Task 9 extends this surface by giving adapters dedicated result aliases (`QuizResult` and
`AdapterResult`) and `From` conversions from `std::io::Error`, `shakmaty::san::ParseSanError`, and
`shakmaty::san::SanError`. The CLI adapter now leans on these conversions directly via the `?`
operator, with tests that inject failing writers to prove `QuizError::Io` is surfaced without
advancing quiz state. Parsing helpers annotate PGN failures with both the offending SAN token and the
underlying `shakmaty` message so adapters can display actionable diagnostics.

### End-to-End Integration Tests

Task 10 adds integration coverage in `crates/quiz-core/tests/end_to_end.rs`, pairing the engine with a deterministic fake port to drive complete quiz sessions. These tests exercise the full loop mandated by the acceptance criteria: a perfect run with zero retries, a single-retry recovery, retry exhaustion leading to a recorded miss, and PGN parsing rejection. The port fixture records prompts, feedback, and the delivered summary, giving confidence that adapter interactions and summary totals stay coherent across the entire session.【F:crates/quiz-core/tests/end_to_end.rs†L1-L213】

## Implementation Roadmap
The roadmap breaks implementation into four atomic streams. Each subsection describes candidate approaches, the trade-offs we e
valuated, and the decision we committed to.

### 1. Scaffold `quiz-core` Crate With Feature-Gated Adapters
**Objective:** Bootstrap the crate structure, declare feature flags, and reserve space for adapters without coupling them to the
 core logic.

```bash
cargo new crates/quiz-core --lib
```

```toml
# crates/quiz-core/Cargo.toml
[package]
name = "quiz-core"
version = "0.1.0"
edition = "2024"

[features]
default = []
cli = []
api = []
wasm = []

[dependencies]
```

**Alternatives considered:**

- *Single monolithic crate with optional binaries but no features.* This keeps configuration simple but forces every consumer to
  compile adapters they may not need, slowing builds in constrained environments.
- *Separate crates per adapter.* This yields maximal isolation but duplicates shared types and complicates dependency managemen
  t when the engine evolves.

**Decision:** Adopt a single crate with optional features. This approach matches the repository norm (e.g., `pgn-import` crates),
 keeps dependency graphs shallow, and lets each consumer opt in only to the adapters they require. Stub binaries in `src/bin/`
 will remain minimal placeholders compiled behind `#![cfg(feature = "...")]` guards until their dedicated tasks flesh them out.

### 2. Define Quiz Interaction Ports and CLI Adapter
**Objective:** Specify the interface through which the engine communicates with presentation layers, and provide a terminal-bac
ked reference implementation.

```rust
// src/ports.rs
pub trait QuizPort {
    fn present_prompt(&mut self, context: PromptContext) -> Result<String, QuizError>;
    fn publish_feedback(&mut self, feedback: FeedbackMessage) -> Result<(), QuizError>;
    fn present_summary(&mut self, summary: &QuizSummary) -> Result<(), QuizError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptContext {
    pub step_index: usize,
    pub total_steps: usize,
    pub board_fen: String,
    pub prompt_san: String,
    pub previous_move_san: Option<String>,
    pub remaining_retries: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackMessage {
    pub step_index: usize,
    pub result: AttemptResult,
    pub learner_response: Option<String>,
    pub solution_san: String,
    pub annotations: Vec<String>,
    pub remaining_retries: u8,
}
```

```rust
// src/cli.rs (cfg(feature = "cli"))
pub struct TerminalPort<R, W> {
    reader: R,
    writer: W,
}

impl TerminalPort<BufReader<io::Stdin>, io::Stdout> {
    pub fn new() -> Self {
        Self::with_io(BufReader::new(io::stdin()), io::stdout())
    }
}

impl<R, W> TerminalPort<R, W> {
    pub fn with_io(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }
}

impl<R, W> QuizPort for TerminalPort<R, W>
where
    R: BufRead,
    W: Write,
{
    // writes prompts, reads SAN responses, prints feedback and summary text,
    // mapping any I/O failure into QuizError::Io
}
```

**Alternatives considered:**

- *Hard-wire std::io in the engine.* Simpler short-term but impossible to reuse in async contexts or unit tests without mockin
  g the entire terminal.
- *Adopt an event-sourcing abstraction up front.* Extremely flexible but adds significant ceremony before the basic quiz loop is
  working.

**Decision:** Define a lightweight synchronous trait. We can layer async adapters later by having them spawn the engine on a separate task or by providing a second trait if needed. The CLI adapter remains opt-in under the `cli` feature, while future tasks will implement `ApiPort` and `WasmPort` modules behind their own flags. Companion unit tests live immediately beneath the implementation in `ports.rs`, ensuring optional previous moves, retry exhaustion messaging, and helper constructors all stay covered.

### 3. Implement PGN Parsing Error Handling
**Objective:** Capture all validation failures distinctly so adapters can display actionable errors to the user.

```rust
// src/errors.rs
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuizError {
    #[error("failed to parse PGN: {0}")]
    UnreadablePgn(String),
    #[error("PGN must contain exactly one game")]
    MultipleGames,
    #[error("variations are not supported in quiz mode")]
    VariationsUnsupported,
    #[error("expected a single main line of moves")]
    WrongFormat,
    #[error("PGN did not contain any moves")]
    NoMoves,
    #[error("I/O error")] // used by adapters
    Io,
}
```

**Alternatives considered:**

- *Map every failure to a generic `InvalidPgn` message.* Easier to implement but leaves end users guessing which part of the inp
  ut failed.
- *Return `shakmaty::Error` directly.* Provides low-level detail but couples us tightly to the upstream crate’s error structure,
  making it harder to provide consistent messaging.

**Decision:** Wrap `shakmaty` errors in our own enum. This maintains human-friendly language while letting the engine branch on
specific cases. Unit tests will construct representative PGN snippets to ensure each variant is reachable.

To consume those errors, `QuizSource::from_pgn` has been implemented as the sole entry point for parsing quiz content. The helper
strips move numbers, results, and annotation glyphs, rejects comments or nested variations outright, and converts each SAN token
into a `shakmaty::san::San` while advancing a `shakmaty::Chess` board. The function returns the starting position together with
the canonical move list so later tasks can hydrate `QuizSession` state without re-parsing PGN. Dedicated unit tests cover
malformed strings (empty inputs, embedded comments, multiple games) to assert that the correct `QuizError` variant surfaces.

### 4. Build Red–Green Test Suite and Documentation
**Objective:** Follow strict TDD to grow the engine, and document every public-facing type and behavior.

```rust
// tests/engine_loop.rs (excerpt)
#[test]
fn quiz_advances_after_correct_answer() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5";
    let mut port = RecordingPort::default();
    let mut engine = QuizEngine::from_pgn(pgn, PlayerColor::White).unwrap();

    port.queue_answers(["e4", "Nf3", "Bb5"]);
    let summary = engine.run(&mut port).unwrap();

    assert_eq!(summary.correct, 3);
    assert_eq!(summary.incorrect, 0);
    assert!(port.messages.contains(&"Correct!".into()));
}
```

Documentation artifacts to update:

- `documentation/chess-quiz-engine.md` (this file) with design reasoning and alternatives.
- `docs/rust-structs-glossary.md` with any new structs/enums (`QuizEngine`, `QuizError`, `FeedbackMessage`).
- Crate-level `README.md` with flow diagrams and adapter notes.

**Alternatives considered:**

- *Lean entirely on integration tests without unit coverage.* Quicker to write initially but hinders pinpointing failures as the
  engine grows.
- *Skip documentation until the engine is functional.* Saves time now but conflicts with repository expectations and slows future
  contributors.

**Decision:** Continue the strict red–green-refactor cadence. Each behavior begins with a failing test, followed by minimal code
to pass, and a refactor phase. Documentation updates accompany each new public type to keep shared knowledge synchronized.

---

These notes should remain the canonical reference while we execute the roadmap. Each task can be tackled independently, enabling
parallel work without design ambiguity.

## Detailed Execution Plan
To convert the roadmap into day-to-day work, we decomposed the effort into ten concrete tasks. Each task lists the inputs we rely on and the tangible outputs that signal completion so parallel contributors can coordinate efficiently.

1. **Confirm scope & acceptance tests**
   - *Inputs:* Expectations from this brief around adapter isolation, retry allowances, and documentation guarantees.
   - *Outputs:* Written acceptance criteria describing the single-line PGN quiz flow, retry behavior, error taxonomy, and the documentation commitments that gate sign-off. Identify the initial red tests we will author before writing engine code.

2. **Scaffold the `quiz-core` crate and workspace wiring**
   - *Inputs:* Workspace manifests, design brief instructions, and dependency choices (`thiserror`, `shakmaty`).
   - *Outputs:* A new `crates/quiz-core` library containing module stubs (`engine`, `state`, `ports`, `errors`), feature flags for adapters (`cli`, `api`, `wasm`), placeholder binaries behind those flags, and updated workspace manifests referencing the crate.

3. **Design quiz state and summary data structures**
   - *Inputs:* The architecture goals separating orchestration logic from persisted state.
   - *Outputs:* Types such as `QuizSession`, `QuizStep`, `AttemptState`, and `QuizSummary` that track FEN snapshots, SAN prompts, retry counters, and scoring totals. Ensure they derive serde-friendly traits where useful and are documented in the structs glossary.

4. **Implement PGN parsing & validation layer**
   - *Inputs:* The parsing responsibilities outlined above and the `shakmaty` API surface.
   - *Outputs:* A constructor like `QuizSource::from_pgn` that enforces single-main-line constraints, converts parser failures into `QuizError` variants, and yields a normalized sequence of moves with the initial board state.

5. **Build the quiz orchestration engine**
   - *Inputs:* The flow diagram in this brief that covers board rendering, prompting, retries, and summary emission.
   - *Outputs:* `QuizEngine` methods (`from_pgn`, `advance`, `run`) that iterate through parsed moves, compare SAN answers, respect the one-retry policy, accumulate annotations, and return a `QuizSummary`.

6. **Define interaction ports and CLI reference adapter**
   - *Inputs:* Port trait specifications and expectations for feature-gated adapters.
   - *Outputs:* A `ports::QuizPort` trait with supporting message structs plus a `TerminalPort` implementation behind the `cli` feature that exercises the end-to-end flow for manual smoke tests.

7. **Harden error handling and adapter-safe boundaries**
   - *Inputs:* The error enum defined earlier and the requirement that adapters remain decoupled from engine internals.
   - *Outputs:* Exhaustive `QuizError` conversions, helper utilities (result aliases, adapter-safe wrappers), and tests that cover malformed PGN inputs, I/O failures, and retry exhaustion scenarios.

8. **Deliver red–green test suite**
   - *Inputs:* TDD mandate and the sample integration test sketches.
   - *Outputs:* Unit and integration tests spanning PGN parsing, retry rules, summary math, and port interactions (mock-based). Include fixtures or snapshots for board states to simplify regression analysis.

9. **Document public API & glossary updates**
   - *Inputs:* Documentation obligations from the design brief and existing repo conventions.
   - *Outputs:* Crate-level `README`, updates to this brief with implementation notes, and entries in `docs/rust-structs-glossary.md` for each public struct or enum we introduce.

10. **Plan follow-on integration work**
    - *Inputs:* Roadmap dependencies on PGN importer normalization, scheduling pipelines, and adapter expansion.
    - *Outputs:* Backlog items or ADR references describing how `quiz-core` will integrate with upstream PGN ingestion, emit telemetry for the scheduler, and expose API/wasm adapters when supporting workstreams mature.

Together, these tasks provide a step-by-step recipe for realizing the quiz engine while honoring our TDD discipline and documentation commitments.

