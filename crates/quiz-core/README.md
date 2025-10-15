# `quiz-core`

The `quiz-core` crate owns the domain logic for running interactive chess quizzes. It is designed
so the core engine can execute without being tied to any particular delivery mechanism. Adapters
(such as terminal, HTTP API, or WASM front-ends) integrate through narrow port traits while feature
flags ensure only the code relevant to a build target is compiled.

## Architecture boundaries

The crate is organised around a small set of modules that mirror the execution plan:

- `engine`: will house the orchestration loop that coordinates prompts, attempts, retries, and
  session summaries.
- `state`: will model immutable quiz session snapshots, including per-move prompts, attempt
  tracking, and scoring details.
- `ports`: will define the adapter-facing traits and message types used by the engine to exchange
  prompts and feedback with external systems.
- `errors`: will collect the error taxonomy shared across the engine, state builders, and adapters.

Each module currently exposes placeholders and documentation so downstream contributors understand
where future implementations belong. As functionality lands, this README should be updated with
links to concrete types and diagrams that clarify responsibilities between modules.

## Feature gating strategy

The crate ships without default features enabled. Consumers explicitly opt into the adapters they
need via the following feature flags:

- `cli`: compiles the reference terminal adapter and its supporting binary at `src/bin/cli.rs`.
- `api`: enables the HTTP/API adapter surface together with the `src/bin/api.rs` binary.
- `wasm`: enables the WebAssembly adapter surface together with the `src/bin/wasm.rs` binary.

This layout allows lightweight builds (e.g., for server-side batch processing) while keeping adapter
code isolated. Additional features should follow the same pattern: guard adapter-specific code with
a named feature flag and add a corresponding binary target only when the feature is enabled.

## Documentation roadmap

- Update this README with concrete examples once the engine APIs are implemented.
- Cross-link the module documentation with the glossary entries in `docs/rust-structs-glossary.md`
  so contributors can quickly discover type definitions and invariants.
- Record adapter-specific considerations (I/O, threading, async boundaries) as those surfaces solidify.
