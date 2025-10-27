# Architecture Cleanup Roadmap

This document consolidates the full architecture review for the `chess-training` workspace and translates it into an actionable cleanup program. It confirms that all major subsystems are captured, clarifies how they collaborate, highlights naming-alignment opportunities, and provides a step-by-step checklist to deliver a modular, SOLID-aligned design.

---

## 1. Final Inventory Check
The repository organises its runtime functionality into the following major components. Each entry lists its purpose, the supporting submodules, and the primary collaboration points. This pass re-validates that no critical subsystem is overlooked before executing the cleanup roadmap.

| Major Component | Location | Purpose & Key Responsibilities | Primary Collaborators | Supporting Submodules / Files |
| --- | --- | --- | --- | --- |
| **Shared Domain Model** | `crates/review-domain` | Defines review cards, unlock rules, deterministic IDs, grading types, and tactical/opening descriptors shared across all services. | Consumed by card store, scheduler core & WASM, PGN importer, quiz engine, session gateway integrations, and any analytics tooling. | `card.rs`, `grade/`, `opening/`, `tactic.rs`, `unlock.rs`, `hash.rs` |
| **Persistence Contracts & Reference Backend** | `crates/card-store` | Exposes `ReviewCardStore` trait, DTOs, validation, and an in-memory implementation that backs tests and WASM bindings. | Scheduler (reads queues), PGN importer (writes), potential analytics/reporting jobs. | `memory/` (cards, edges, reviews, unlocks), `config.rs`, `error.rs`, `validation` helpers |
| **Scheduling Engine** | `crates/scheduler-core`, `crates/scheduler-wasm` | Builds review queues, enforces SM-2 cadence, manages unlock policies, and offers WASM bindings for browser usage. | Depends on card store & domain; feeds session gateway; WASM facade serves web clients and demos. | Queue builders, unlock evaluators, SM-2 math, configuration modules, `SchedulerFacade`, DTOs |
| **PGN Ingestion Pipeline** | `crates/chess-training-pgn-import` | CLI tool that parses PGN files, normalises them into domain entities, and persists via the card store. | Card store (writes), domain crate (types), future streaming adapters. | `config.rs`, `errors.rs`, `importer.rs`, `model.rs`, `storage.rs`, `main.rs` |
| **Quiz Engine** | `crates/quiz-core` | Generates and runs PGN-based quizzes as a sibling training flow to reviews. | Shares domain types; can persist via card store or bespoke stores; potential UI/CLI adapters. | `engine/`, `state/`, `source/`, `ports/`, `errors/`, optional `cli/`, `api/`, `wasm/` |
| **Service Gateway** | `apps/session-gateway` | Node/TypeScript service bridging the scheduler to browser clients via REST and WebSockets, coordinating session state and grading. | Scheduler core (via HTTP/gRPC/WASM), Web UI, telemetry pipelines. | `index.ts`, `config.ts`, `server.ts`, `sessionService.ts`, `broadcaster.ts`, `clients/`, `stores/`, `types.ts` |
| **Web Client** | `web-ui/` | Vite + React UI delivering review sessions, dashboards, and playground tooling. | Talks to session gateway; consumes scheduler DTOs; may replay quiz sessions. | `application/`, `clients/`, `components/`, `fixtures/`, `pages/`, `services/`, `state/`, `styles/`, `types/`, `utils/` |
| **Workspace Binary & Harness** | `src/main.rs` | Lightweight integration harness validating cross-crate wiring; can evolve into smoke-test binary. | Links to domain, scheduler, store crates as integration exercises. | Root binary code (minimal today) |
| **Documentation & Standards** | `documentation/`, `docs/`, `repo-naming-standards.md` | Guides architecture, naming, and roadmap expectations. | Referenced by every subsystem during refactors. | Existing roadmaps, execution plans, naming standards |

This inventory confirms the review spans every code-hosting directory and all runtime-critical flows.

---

## 2. Architecture Overview & Interactions

### 2.1 Shared Domain Model (`crates/review-domain`)
- **Role:** Source of truth for review entities, unlock ledgers, tactics/openings, and deterministic identifiers.
- **Key Interactions:** Provides typed contracts and stable hashes to storage, scheduler, importer, quiz engine, and gateway integrations.
- **Internal Structure:** Modules segregated by domain slice (cards, grading, unlock, opening, tactic, hashing).
- **Refinement Opportunities:**
  - Split into focused crates (e.g., `review-domain-core`, `review-domain-openings`, `review-domain-tactics`, `review-identity`) so consumers pull only relevant slices.
  - Publish conversion traits (`IntoCardRecord`, `UnlockLedgerAccess`) to reduce direct struct coupling.
  - Consolidate hashing logic into a shared infra crate consumed by all domain slices.

### 2.2 Persistence Layer (`crates/card-store`)
- **Role:** Defines persistence contracts plus the reference in-memory backend.
- **Key Interactions:** Scheduler reads data via the trait; importer writes; WASM façade bundles the in-memory store.
- **Internal Structure:** Trait definitions with supporting config/error modules and `memory/` backend.
- **Refinement Opportunities:**
  - Separate traits/config/errors into `review-store-core` and relocate the memory backend into `review-store-memory`.
  - Provide dedicated read/write traits for finer substitution (e.g., analytics worker may be read-only).
  - Extract validation helpers into reusable modules consumed by all implementations.

### 2.3 Scheduling Engine (`crates/scheduler-core`, `crates/scheduler-wasm`)
- **Role:** Builds review queues, enforces unlock logic, and exposes configuration APIs and WASM bindings.
- **Key Interactions:** Consumes domain + store; session gateway invokes scheduling APIs; WASM harness powers browser/offline clients.
- **Internal Structure:** Queue builders, SM-2 math, unlock policy evaluators, configuration modules, WASM DTOs (`SchedulerConfigDto`, `SchedulerFacade`).
- **Refinement Opportunities:**
  - Decompose into `review-scheduler-core` (trait + orchestration), `review-scheduler-queue`, `review-scheduler-unlock`, and `review-scheduler-math` crates.
  - Expose a `SchedulerEngine` trait, enabling alternative adapters and easier WASM integration.
  - Allow the WASM package (renamed `review-scheduler-wasm`) to accept any store implementation conforming to the trait.

### 2.4 PGN Ingestion (`crates/chess-training-pgn-import`)
- **Role:** CLI import pipeline translating PGN into review domain entities.
- **Key Interactions:** Writes through the card-store traits; shares models with the domain crate.
- **Internal Structure:** Configuration, storage trait adapter, importer engine, CLI entry point.
- **Refinement Opportunities:**
  - Split into `pgn-import-core` (logic) and `pgn-import-cli` (argument parsing/execution).
  - Create separate modules for openings vs. tactics ingestion.
  - Define a storage adapter trait decoupled from the card store, then implement a bridge adapter for `review-store-core`.

### 2.5 Quiz Engine (`crates/quiz-core`)
- **Role:** Provides quiz-based training flows using PGN sources as complements to reviews.
- **Key Interactions:** Shares domain types; can integrate with store or bespoke persistence; future UI/CLI/WASM adapters.
- **Internal Structure:** Engine state machine, source abstractions, ports for messaging, optional adapters for CLI/API/WASM.
- **Refinement Opportunities:**
  - Promote core engine to `quiz-engine-core` and move adapters into `quiz-engine-cli`, `quiz-engine-api`, `quiz-engine-wasm`.
  - Standardise port traits so adapters compile with minimal dependencies.
  - Align naming and grade reporting with review domain for cross-feature analytics.

### 2.6 Session Gateway (`apps/session-gateway`)
- **Role:** Node/TypeScript gateway orchestrating review sessions over REST/WebSockets.
- **Key Interactions:** Calls scheduler service, broadcasts to web clients, records grading/analytics.
- **Internal Structure:** Express server bootstrap, session service orchestrator, broadcaster, typed clients/stores.
- **Refinement Opportunities:**
  - Rename to `apps/review-session-gateway` to mirror responsibility.
  - Break down `createSessionService` into collaborators (queue bootstrapper, grade processor, stats aggregator, event emitter).
  - Introduce interfaces for analytics/reporting emitters to ease swapping implementations.

### 2.7 Web UI (`web-ui/`)
- **Role:** React front end delivering review experiences and dashboards.
- **Key Interactions:** Consumes session gateway APIs, scheduler DTOs, and possibly quiz adapters.
- **Internal Structure:** Feature code under `application/`, `pages/`, `components/`, fixtures, state/services utilities.
- **Refinement Opportunities:**
  - Restructure into `web-ui/features/{review,quiz,dashboard}` with dedicated state/services.
  - Isolate demo fixtures under `web-ui/playground/` or Storybook-specific directories.
  - Extract shared components into a `web-ui/design-system` package for consistency.

### 2.8 Workspace Binary (`src/main.rs`)
- **Role:** Minimal integration harness; can orchestrate cross-crate smoke tests.
- **Refinement Opportunities:**
  - Expand into `tools/review-integration-runner` binary crate to host integration tests.
  - Use as a staging ground for verifying trait boundaries after modularisation.

### 2.9 Documentation & Standards (`documentation/`, `docs/`)
- **Role:** Houses architectural guides, execution plans, and naming standards.
- **Refinement Opportunities:**
  - Consolidate architecture guides under a consistent prefix (`documentation/architecture-*`).
  - Maintain naming playbooks in sync with code refactors.

---

## 3. Cross-Cutting Interaction Map
1. **Data Flow:** PGN importer converts raw games into domain entities and persists them via the store; scheduler reads the store to create queues; session gateway surfaces queues and collects grading; web UI interacts with the gateway; quiz engine offers an alternative training route using the same domain types.
2. **Shared Contracts:** `review-domain` (and its proposed sub-crates) anchors data models; consistent IDs and hashes ensure interoperability.
3. **Adapters & Ports:** WASM packages, CLI binaries, and Node services function as adapters around the Rust cores, each requiring clear trait-driven contracts to swap implementations without ripple effects.
4. **Observability & Analytics:** Gateway grading and quiz outcomes should share reporting interfaces, enabling central analytics or telemetry services in future phases.

---

## 4. Naming Alignment Recommendations
To support consistent naming across the workspace and align with `repo-naming-standards.md`, adopt the following conventions:

### 4.1 Top-Level Directories
| Current Name | Proposed Alignment | Rationale |
| --- | --- | --- |
| `crates/review-domain` | `crates/review-domain-core` (parent workspace), plus child crates `crates/review-domain-openings`, `crates/review-domain-tactics`, `crates/review-identity` | Clarifies the crate’s scope and keeps specialised features optional. |
| `crates/card-store` | `crates/review-store-core`, `crates/review-store-memory` | Distinguishes trait contracts from the reference backend. |
| `crates/scheduler-core`, `crates/scheduler-wasm` | `crates/review-scheduler-core`, `crates/review-scheduler-queue`, `crates/review-scheduler-unlock`, `crates/review-scheduler-math`, `crates/review-scheduler-wasm` | Aligns scheduler naming with the review domain and separates responsibilities. |
| `crates/chess-training-pgn-import` | `crates/pgn-import-core`, `crates/pgn-import-cli`, potential `crates/pgn-import-http` | Shortens and standardises crate names while flagging interface-specific adapters. |
| `crates/quiz-core` | `crates/quiz-engine-core`, with optional adapters `crates/quiz-engine-cli`, `crates/quiz-engine-wasm`, `crates/quiz-engine-api` | Mirrors the adapter pattern used elsewhere. |
| `apps/session-gateway` | `apps/review-session-gateway` | Highlights its focus on review sessions versus quiz tooling. |
| `web-ui/` | `apps/review-web-ui` (root) with nested packages `apps/review-web-ui/design-system`, `apps/review-web-ui/features/*` | Unifies application naming and clarifies that this UI targets review flows. |
| `src/main.rs` | Move into `tools/review-integration-runner` | Keeps workspace binaries in a dedicated tools namespace. |

### 4.2 Cross-Crate Type & Module Conventions
- Adopt consistent suffixes: `*Store`, `*Facade`, `*Config`, `*Record`, `*Queue`.
- Synchronise verb usage per `repo-naming-standards.md` (e.g., `build_queue`, `record_unlock`, `upsert_card`).
- Reserve `memory` namespace exclusively for in-memory adapters (e.g., `memory::InMemoryReviewStore`).
- Ensure WASM/CLI/API packages expose DTOs under `dto/` modules and trait adapters under `adapter/` modules for discoverability.

---

## 5. Step-by-Step Modularisation Checklist
The following checklist enumerates independent, actionable tasks required to deliver the modular architecture. Each task is scoped to minimise cross-cutting risk while building toward the overall goal.

### Phase 0 – Preparation & Analysis
- [ ] **Capture Dependency Graphs:** Generate Cargo dependency graph and service diagrams to validate refactor ordering (e.g., `cargo tree`, architecture diagrams in `documentation/`).
- [ ] **Stabilise Naming Playbook:** Update `repo-naming-standards.md` with any new suffix/prefix decisions before renaming crates.
- [ ] **Introduce Integration Tests:** Expand `src/main.rs` (or future `tools/review-integration-runner`) to exercise domain → store → scheduler wiring as a safety net.

### Phase 1 – Domain Decomposition
- [ ] **Create `review-identity` crate:** Move deterministic hash/ID helpers from `review-domain` into a dedicated crate consumed by all others.
- [ ] **Extract `review-domain-openings` and `review-domain-tactics`:** Relocate specialised modules (`opening/`, `tactic.rs`) and expose feature flags in the parent `review-domain-core` crate.
- [ ] **Publish conversion traits:** Define traits for translating importer and quiz data into domain records to decouple direct struct usage.
- [ ] **Update dependents:** Point scheduler, store, importer, and quiz crates to the new domain packages and ensure tests compile.

### Phase 2 – Persistence Layer Split
- [ ] **Create `review-store-core` crate:** Move traits, config, errors, and validators out of `card-store`.
- [ ] **Create `review-store-memory` crate:** Host the in-memory backend with explicit dependency on the core crate.
- [ ] **Introduce specialised traits:** Define read-only (`ReviewStoreReader`) and write-only (`ReviewStoreWriter`) traits to support selective implementations.
- [ ] **Refactor dependents:** Update scheduler, importer, WASM bindings, and tests to rely on the core crate plus chosen adapters.

### Phase 3 – Scheduler Modularisation
- [ ] **Define `SchedulerEngine` trait:** Abstract queue generation and unlock evaluation behind a trait in `review-scheduler-core`.
- [ ] **Split queue/unlock/math logic:** Move existing modules into dedicated crates (`review-scheduler-queue`, `review-scheduler-unlock`, `review-scheduler-math`).
- [ ] **Refactor WASM package:** Rename to `review-scheduler-wasm` and ensure it depends solely on the trait, accepting store implementations as generics.
- [ ] **Add adapter tests:** Cover queue generation against in-memory and (future) remote stores using the new trait boundaries.

### Phase 4 – Ingestion Pipeline Refinement
- [ ] **Split CLI and core crates:** Create `pgn-import-core` (logic) and `pgn-import-cli` (argument parsing & binary entry).
- [ ] **Implement storage adapter trait:** Introduce `ImportSink` trait with an adapter for `review-store-core`.
- [ ] **Segment content pipelines:** Separate openings vs. tactics ingestion modules with explicit orchestration APIs.
- [ ] **Add integration scenarios:** Test CLI and programmatic flows against both in-memory and mocked remote stores.

### Phase 5 – Quiz Engine Isolation
- [ ] **Rename to `quiz-engine-core`:** Align crate naming with the adapter pattern.
- [ ] **Extract adapters:** Move CLI/API/WASM integrations into dedicated crates with minimal dependencies.
- [ ] **Align naming with review domain:** Harmonise grading/queue terminology with `review-domain` for analytics parity.
- [ ] **Formalise port interfaces:** Publish clearly documented traits for message buses, persistence, and telemetry.

### Phase 6 – Service Gateway Refactor
- [ ] **Rename directory to `apps/review-session-gateway`:** Update package metadata and deployment scripts.
- [ ] **Decompose `createSessionService`:** Introduce modules for queue bootstrapper, grade processor, stats aggregator, event broadcaster, analytics reporter.
- [ ] **Isolate REST vs. WebSocket orchestration:** Provide separate initialisers and shared session context interfaces.
- [ ] **Harmonise DTO naming:** Ensure gateway DTOs align with scheduler/store naming conventions.

### Phase 7 – Web UI Restructuring
- [ ] **Adopt feature module layout:** Move review, quiz, and dashboard flows under `apps/review-web-ui/features/*` with co-located state/services.
- [ ] **Extract design system:** Create `apps/review-web-ui/design-system` for shared components and styling primitives.
- [ ] **Relocate fixtures/playgrounds:** Move prototype data into a dedicated `playground/` directory or Storybook stories.
- [ ] **Synchronise API clients:** Regenerate clients (OpenAPI or typed clients) once gateway DTOs stabilise.

### Phase 8 – Tooling & Documentation Consolidation
- [ ] **Create `tools/review-integration-runner`:** Relocate `src/main.rs` into a dedicated binary crate for integration smoke tests.
- [ ] **Standardise documentation naming:** Prefix architecture docs with `architecture-` and cross-link them in a table of contents.
- [ ] **Update glossaries and standards:** Reflect renamed crates, traits, and modules in `repo-naming-standards.md` and `docs/rust-structs-glossary.md`.
- [ ] **Publish architecture diagrams:** Include updated dependency charts and interaction maps in `documentation/`.

### Phase 9 – Validation & Cleanup
- [ ] **Run full workspace tests (`make test`):** Verify all crates and apps pass after restructuring.
- [ ] **Audit dependency boundaries:** Use `cargo deny` or similar tooling to ensure new crates respect intended dependency directions.
- [ ] **Review documentation accuracy:** Ensure READMEs and architecture guides reflect new module boundaries.
- [ ] **Plan future iterations:** Capture follow-up enhancements (analytics pipelines, additional adapters) based on lessons learned.

---

## 6. Implementation Notes & Guardrails
- **Incremental Delivery:** Execute phases in order, but keep each task reviewable on its own to avoid monolithic merges.
- **Backward Compatibility:** Maintain compatibility layers (e.g., re-export old module paths) during transitions to limit downstream disruption.
- **Testing Strategy:** Prioritise integration tests at each phase to catch contract drift early.
- **Documentation Discipline:** Update architecture and naming docs alongside code changes to prevent future drift.

This roadmap positions the project for a modular, maintainable architecture while aligning naming conventions and SOLID principles across the entire workspace.
