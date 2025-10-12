# UI–Backend Separation Plan

## Goal
Establish a dedicated presentation/application layer that encapsulates all domain and orchestration logic between the React UI and the backend gateways. This layer will expose explicit contracts that the UI consumes, ensuring the UI focuses solely on rendering and interaction while the backend focuses on persistence and core domain operations.

## Current Pain Points
A review of the existing UI uncovered several places where components perform orchestration or business logic that should live in an intermediate layer.

| Area | Current Responsibility in UI | Why It Should Move |
| --- | --- | --- |
| PGN import pane | Normalizes PGN text, matches openings, formats move previews, and builds scheduled feedback messages in the component itself.【F:web-ui/src/components/PgnImportPane.tsx†L26-L214】 | Parsing, pattern matching, and scheduling copy should be owned by a reusable service so the UI only renders results and surface-level validation.
| Opening review board | Manages a `chess.js` instance, grades moves, controls latency tracking, and toggles teaching arrows directly from the component.【F:web-ui/src/components/OpeningReviewBoard.tsx†L36-L216】 | Move validation, grading, and timing are domain concerns; extracting them will make the component a thin view over a board controller.
| Dashboard metrics | Computes completion/accuracy percentages and renders status badges, duplicating logic that should be centralized.【F:web-ui/src/components/ReviewDashboard.tsx†L24-L82】 | Percentages, label text, and badge selection should be provided by a view model to ensure consistency across surfaces.
| Session routes | Bootstraps demo sessions, derives overview data, and manages grading latencies directly inside routing logic.【F:web-ui/src/components/SessionRoutes.tsx†L17-L105】 | Route-level coordination should call into a session controller that exposes start/grade commands and derived overview data.
| App root | Owns command dispatcher wiring and schedules imported lines by generating IDs and future dates in React state.【F:web-ui/src/App.tsx†L31-L137】 | Application-level command mapping and scheduling should sit in the presentation layer so alternative UIs can reuse them.
| Dashboard overview helpers | Calculate progress metrics and append imported lines, effectively duplicating planner responsibilities.【F:web-ui/src/utils/dashboardOverview.ts†L5-L56】 | These transformations belong in a dedicated view model that can be unit-tested without React.
| Session store | Talks to the gateway, maintains queue state, and manages grade submissions entirely inside the UI tree.【F:web-ui/src/state/sessionStore.ts†L1-L73】 | A presentation service should own session lifecycles and expose a simple read-only state contract to the UI.
| Imported line utilities | Generate IDs, compute schedule dates, and perform move equality checks invoked directly from React state updaters.【F:web-ui/src/utils/importedLines.ts†L3-L42】 | Scheduling logic should move behind an import service so the UI just calls `planImport(line)` and renders the result.

## Target Architecture
```
Backend APIs ──> Gateway clients ──> Application Layer (new) ──> React UI
```

* **Gateway clients** remain thin fetch wrappers.
* **Application layer** encapsulates orchestration, planning, and state transitions. It exposes:
  * Stateless services (e.g., `PgnImportService`, `OpeningReviewService`).
  * Stateful controllers/stores (e.g., `SessionController`) with immutable snapshots or observable streams.
  * Typed view models for pages/components (e.g., `DashboardViewModel`, `OpeningReviewViewModel`).
* **React UI** consumes contracts from the application layer, subscribes to controller snapshots, and renders.

## Proposed Contracts
| Contract | Responsibility | Consumed By |
| --- | --- | --- |
| `PgnImportService` | Accepts raw PGN input or files, returns detection results, validation errors, and the formatted preview. Schedules friendly feedback messages based on import outcomes. | `PgnImportPane` |
| `OpeningReviewController` | Wraps `chess.js`, tracks latency, validates moves, and emits board state plus grade results. Provides methods `selectSquare`, `dropPiece`, `submitGrade`. | `OpeningReviewBoard` |
| `DashboardViewModel` | Produces derived metrics, badge labels, and upcoming unlock entries from session stats and planned imports. | `ReviewDashboard`, `DashboardPage` |
| `SessionController` | Starts sessions, exposes read-only state (`currentCard`, `stats`, `status`), and handles grade submissions with latency measurement. | `SessionRoutes`, other session-aware pages |
| `CommandPaletteService` | Registers navigation/utility commands, exposes `execute`, and emits results or errors. | `CommandConsole`, `App` |
| `ImportPlanner` | Determines schedule dates and IDs for new repertoire lines and persists them via backend APIs when available. | `SessionController`, `DashboardViewModel` |

## Migration Plan
1. **Define application layer structure**
   * Create `web-ui/src/application/` with subfolders for `services/`, `controllers/`, and `viewModels/`.
   * Re-export public contracts via an index to keep imports stable.

2. **Extract stateless services**
   * Move PGN parsing/matching, imported line scheduling, and command dispatcher initialization into dedicated services.
   * Provide pure functions with exhaustive unit tests.

3. **Introduce controllers for stateful flows**
   * Replace `sessionStore` with a `SessionController` that wraps the gateway and exposes an immutable snapshot plus a subscribe mechanism.
   * Add an `OpeningReviewController` that encapsulates the `chess.js` interactions and emits board state + feedback events.

4. **Create page-level view models**
   * Build `DashboardViewModel` that combines `ReviewPlanner`, session stats, and import plans to produce the full dashboard payload.
   * Ensure view models deliver ready-to-render data (strings, labels) so components stop computing percentages or badges themselves.

5. **Refactor UI components**
   * Update components to depend on contracts (props + hooks) from the application layer.
   * Remove in-component computations, leaving only rendering and event binding.

6. **Gradual integration strategy**
   * Start with PGN import and dashboard view model since they have the smallest surface area.
   * Follow with session/board controllers to reduce coupling to `chess.js` and backend calls.
   * Use feature flags or adapter hooks to bridge old and new implementations during migration if needed.

7. **Documentation & testing**
   * Document each application-layer contract and lifecycle in `docs/`.
   * Expand unit/integration tests to cover services and controllers independently of React.

## Expected Outcomes
* **Improved testability**: services and controllers can be tested without rendering components.
* **UI portability**: alternative clients (e.g., mobile) can reuse application layer contracts.
* **Clear ownership**: backend logic stays server-side, application logic sits in the middle layer, and UI remains presentational.
* **Simpler maintenance**: changes to business rules (scheduling, grading, recommendations) occur in one place instead of scattered across components.
