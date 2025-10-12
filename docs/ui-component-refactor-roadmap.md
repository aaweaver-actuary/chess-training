# UI Component Refactor Roadmap

> **Status update (May 2024):** This roadmap is represented within Workstream 1 of the
> consolidated [`docs/strategic-roadmap.md`](./strategic-roadmap.md). Use that file for
> sequencing, dependencies, and progress tracking while retaining this document for the
> detailed checklists and component-specific guidance.

## Purpose
The current web UI concentrates application logic, derived view state, and imperative browser hooks directly inside React components. This document captures concrete opportunities to extract reusable, testable units so the UI tree can focus on presentation. Each workstream below links to the source responsible today and proposes new boundaries (controllers, hooks, services, or pure components) with a checklist to guide incremental delivery.

## Summary of Opportunities
| Surface | Observed Problem | Suggested Abstraction | Primary Benefit |
| --- | --- | --- | --- |
| Opening review board | Board component owns chess engine state, latency tracking, grading, overlay rendering, and DOM mutations.【F:web-ui/src/components/OpeningReviewBoard.tsx†L36-L352】 | `OpeningReviewController` hook/service exposing board snapshot + intent handlers; dedicated `OpeningReviewOverlay` presentational component. | Separates complex chess orchestration from rendering so both board logic and overlay visuals can be tested independently. |
| PGN import tools | Pane intermixes parsing, heuristics, scheduling copy, focus traps, and DOM event lifecycles.【F:web-ui/src/components/PgnImportPane.tsx†L26-L412】 | `usePgnImport` hook backed by `PgnImportService` for detection + messaging, plus light-weight `PgnImportPane` composed of subcomponents (`ModeToggle`, `DetectionSummary`). | Allows pure testing of parsing/scheduling rules and unlocks reuse of import feedback in future flows. |
| Session routing shell | Routes perform session lifecycle orchestration, subscribe directly to store, compute overview, and translate timing data.【F:web-ui/src/components/SessionRoutes.tsx†L20-L105】 | `SessionController` abstraction that exposes `snapshot` + `subscribe`, `DashboardViewModel` for derived overview strings, and route-level wrapper that only renders components. | Centralizes session orchestration while letting UI consume ready-to-render data. |
| App root | App owns command dispatcher wiring, keyboard listeners, and import scheduling side effects in state reducers.【F:web-ui/src/App.tsx†L13-L137】 | `useCommandPalette` hook (delegating to `CommandPaletteService`) and `useImportPlanner` hook/service for scheduling with deterministic IDs/dates. | Enables reusing command palette + import planner outside React and simplifies App to wiring providers. |
| Dashboard metrics | Presentational component calculates percentages and badge labels on every render.【F:web-ui/src/components/ReviewDashboard.tsx†L24-L83】 | `DashboardViewModel` (shared with SessionRoutes above) that supplies formatted strings and badge variants. | Prevents duplication of formatting logic and ensures consistent copy across future dashboards. |
| Imported line utilities | Scheduler produces IDs/dates inside UI state updates; overview helper concatenates unlock rows.【F:web-ui/src/utils/importedLines.ts†L5-L42】【F:web-ui/src/utils/dashboardOverview.ts†L5-L56】 | Promote to application-layer services reused by both controllers and view models. | Eliminates duplicated transformation logic and clarifies separation between data planning and UI binding. |
| Session store | Store speaks to gateway, mutates global state, and exposes imperative API consumed directly by UI components.【F:web-ui/src/state/sessionStore.ts†L1-L73】 | Wrap existing gateway inside `SessionController` mentioned above with immutable snapshot + typed events. | Unlocks deterministic tests for session transitions and clarifies ownership of async gateway calls. |
| Command console | Component handles animation lifecycle, command submission policy, and integrates directly with dispatcher side effects.【F:web-ui/src/components/CommandConsole.tsx†L21-L189】 | Split UI (`CommandConsole`) from behaviorful `useCommandConsole` hook that owns animation timers and submission semantics provided by `CommandPaletteService`. | Simplifies UI into declarative markup and allows headless testing of command submission rules. |

## Workstreams & Checklists

### 1. Opening Review Experience
**Current issues**
- Chess rules, grading, latency, and DOM overlay management live inside `OpeningReviewBoard`, forcing comprehensive React tests to validate every flow.【F:web-ui/src/components/OpeningReviewBoard.tsx†L36-L276】
- Teaching arrow extraction and error highlighting logic are hidden helpers in the same module, coupling board rendering with metadata interpretation.【F:web-ui/src/components/OpeningReviewBoard.tsx†L317-L352】

**Refactor approach**
- Create an `OpeningReviewController` (class or hook) that wraps `chess.js`, manages expected move queues, and emits a serializable board state (`fen`, `selectedSquare`, `legalTargets`, `errorSquare`, `teachingArrow`, `lichessUrl`, `status`).
- Expose intent methods (`selectSquare`, `dropPiece`, `submitMove`) returning structured results (`grade`, `latencyMs`, `nextState`).
- Reduce `OpeningReviewBoard` to a presentational component that consumes controller state via props/context and delegates overlay rendering to a new `OpeningReviewOverlay` pure component.
- Relocate error highlight timing and teaching-arrow toggles into the controller so React components observe state changes instead of manipulating DOM attributes.

**Checklist**
- [ ] Introduce controller contract (`OpeningReviewController`) with unit tests covering move validation, grading, and latency rules.
- [ ] Add `useOpeningReviewController(card, onResult)` hook that binds the controller to React lifecycle.
- [ ] Extract `OpeningReviewOverlay` component that renders overlay squares purely from props.
- [ ] Update `OpeningReviewBoard` to consume hook state + overlay component, removing direct DOM mutations.
- [ ] Document controller lifecycle and testing strategy in `docs/`.

### 2. PGN Import Tools
**Current issues**
- `PgnImportPane` contains parsing heuristics (`sanitizeMoves`, `detectOpening`), message formatting, state machine for mode switching, and command palette bindings.【F:web-ui/src/components/PgnImportPane.tsx†L39-L310】
- File reading, feedback copy, and scheduling logic are tightly coupled to component state, complicating reuse or testing without DOM involvement.【F:web-ui/src/components/PgnImportPane.tsx†L213-L409】

**Refactor approach**
- Move parsing and detection into a pure `PgnImportService` returning discriminated unions for `UnknownOpening`, `DetectedOpening`, and `ScheduledResult` messages.
- Provide a `usePgnImport` hook that orchestrates service calls, mode transitions, and command dispatcher integration, returning a simple state machine for the pane.
- Break UI into smaller components: `PgnImportHandle`, `PgnImportModeToggle`, `PgnImportForm`, `DetectedLinePreview`, and `FeedbackBanner` so each part can be snapshot tested independently.

**Checklist**
- [ ] Implement `PgnImportService` with exhaustive unit tests for parsing/detection edge cases.
- [ ] Build `usePgnImport({ onImportLine, commandDispatcher })` hook managing mode, text, detection, and feedback state.
- [ ] Refactor `PgnImportPane` to consume the hook and compose new presentational subcomponents.
- [ ] Extract command registration/unregistration into the hook (with tests ensuring cleanup).
- [ ] Document supported import flows and failure cases.

### 3. Session Shell & Dashboard Data
**Current issues**
- `SessionRoutes` simultaneously subscribes to global store, triggers session start side effects, measures latency, and builds overview data.【F:web-ui/src/components/SessionRoutes.tsx†L20-L105】
- Overview computations and imported line extensions are spread across utils and components, redoing formatting work already needed elsewhere.【F:web-ui/src/components/ReviewDashboard.tsx†L24-L83】【F:web-ui/src/utils/dashboardOverview.ts†L5-L56】
- `sessionStore` mixes gateway calls with state mutations and exposes imperative methods without deterministic snapshots.【F:web-ui/src/state/sessionStore.ts†L1-L73】

**Refactor approach**
- Encapsulate session lifecycle in a `SessionController` responsible for starting sessions, grading cards, and emitting immutable snapshots (`currentCard`, `queue`, `stats`, `status`). Provide `subscribe`/`getSnapshot` to integrate with React or other clients.
- Create a `DashboardViewModel` that accepts `SessionSnapshot`, `ReviewPlanner`, and planned imports to produce formatted labels, badge variants, and unlock lists (strings only).
- Introduce `useSessionController` hook wrapping the controller for React components, and update routing layer to purely render `<Routes>` using provided data.

**Checklist**
- [ ] Extract `SessionController` from `sessionStore`, separating gateway access and state emission.
- [ ] Provide `useSessionController()` hook (or context provider) for React integration with tests covering subscription/unsubscription.
- [ ] Implement `DashboardViewModel` returning view-ready data; update `ReviewDashboard` to consume props only (no local calculations).
- [ ] Replace `composeOverview`/`extendOverviewWithImports` with methods on the view model or controller.
- [ ] Update `SessionRoutes` to consume controller snapshot + view model outputs without direct gateway calls.

### 4. Command Palette & App Shell
**Current issues**
- App root listens for keyboard shortcuts, manages console open state, maps commands to navigation, and schedules imported lines within React state updates.【F:web-ui/src/App.tsx†L31-L137】
- `CommandConsole` component owns animation timing state and submission semantics that would be easier to test outside JSX.【F:web-ui/src/components/CommandConsole.tsx†L21-L189】
- Import scheduling uses ad-hoc utilities (`createOpeningLineScheduler`, `linesMatch`) directly in React state, limiting reuse across contexts.【F:web-ui/src/utils/importedLines.ts†L5-L42】

**Refactor approach**
- Create a `CommandPaletteService` that registers commands, executes handlers, and reports unknown commands. Wrap it in a `useCommandPalette` hook to manage keyboard listeners and dispatcher wiring.
- Extract `useCommandConsole` hook that handles animation timers, focus reset, and form submission, leaving `CommandConsole` purely presentational.
- Promote `createOpeningLineScheduler` into an application-layer `ImportPlanner` service and expose a `useImportPlanner` hook that App (and future contexts) can call to schedule lines.

**Checklist**
- [ ] Implement `CommandPaletteService` with unit tests covering registration, execution, and unknown command handling.
- [ ] Build `useCommandPalette` to integrate keyboard shortcuts and provide dispatcher functions to components.
- [ ] Extract `useCommandConsole` hook for open/close/submit logic; refactor component to consume the hook outputs (class names, handlers, state).
- [ ] Introduce `ImportPlanner` service for scheduling + deduplication with deterministic tests; replace direct `linesMatch` usage in App.
- [ ] Ensure App root merely composes providers/hooks and renders routes + console.

### 5. Supporting Documentation & Testing
- [ ] Record architecture of the new application layer (controllers, services, hooks) alongside existing `ui-separation-plan` for future contributors.【F:docs/ui-separation-plan.md†L1-L104】
- [ ] Add testing guidelines describing which units require React Testing Library vs. headless unit tests.
- [ ] Track incremental migration progress in this roadmap (check off items, link PRs) to maintain visibility.

## Next Steps
1. Socialize this roadmap with maintainers and prioritize the workstreams (suggest starting with PGN import + dashboard view model for quick wins).
2. For each workstream, create tickets referencing the checklist items and link back to this document for traceability.
3. Update the document as tasks complete to keep it the authoritative guide for UI refactoring efforts.
