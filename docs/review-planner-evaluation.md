# Review Planner Calculation Placement

This note tracks the migration of the review planner calculations from the front-end TypeScript
service into the Rust scheduler so the backend owns the review schedule.

## Current architecture

- The core recommendation and backlog logic now lives in `scheduler-core` within the
  `ReviewPlanner` module, which codifies the thresholds for backlog pressure, accuracy risk, and the
  associated guidance strings.【F:crates/scheduler-core/src/review_planner.rs†L1-L215】
- A wasm binding exposes the Rust planner through the `buildReviewOverview` helper so JavaScript
  callers can request an overview without reimplementing the rules in TypeScript.【F:crates/scheduler-wasm/src/bindings.rs†L1-L74】
- The web UI consumes a baseline overview emitted by the Rust planner and only recomputes the
  progress section when fresh session stats arrive, leaving the recommendation logic to the backend.
  This workflow is orchestrated by `dashboardOverview.ts` together with the
  `baselineOverview.ts` fixture.【F:web-ui/src/utils/dashboardOverview.ts†L1-L45】【F:web-ui/src/fixtures/baselineOverview.ts†L1-L30】

## Front-end responsibilities

- `SessionRoutes.tsx` hydrates the dashboard with the Rust-provided baseline overview and merges in
  session stats from the gateway before rendering React components.【F:web-ui/src/components/SessionRoutes.tsx†L5-L88】
- Presentation components such as `ReviewDashboard.tsx` remain focused on rendering metrics and do
  not perform domain calculations.【F:web-ui/src/components/ReviewDashboard.tsx†L1-L78】
- Type definitions for the overview shape live alongside other shared types so the UI mirrors the
  Rust payload exactly.【F:web-ui/src/types/reviewOverview.ts†L1-L30】

## Gateway implications

- The session gateway still exposes only review counts, accuracy, and latency metrics; it does not
  yet persist the full overview payload. Until those endpoints are extended, the UI relies on the
  precomputed baseline overview and local stat composition.【F:apps/session-gateway/src/sessionService.ts†L1-L123】
- When the gateway begins streaming overview data, the new wasm helper can be replaced with direct
  API calls because the server and client now agree on the review planner contract.

## Next steps

1. **Extend gateway telemetry.** Persist due and completed counts so the API can deliver the same
   progress information the UI currently derives locally.
2. **Surface overview endpoints.** Add REST or WebSocket messages that return the Rust
   `ReviewOverview` structure to clients.
3. **Replace the fixture.** Once the backend returns live overviews, remove the baseline fixture and
   hydrate the dashboard directly from the API response, keeping the UI logic presentation-only.
