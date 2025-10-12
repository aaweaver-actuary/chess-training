# Chess Training Strategic Roadmap

## Document Inventory and Scope
| Document | Primary Focus | Scope Summary |
| --- | --- | --- |
| `docs/ui-separation-plan.md` | Define an application layer between React UI and backend gateways. | Targets presentation/application architecture, describing pain points in current UI components and proposing contracts for services, controllers, and view models.【F:docs/ui-separation-plan.md†L1-L75】 |
| `docs/ui-component-refactor-roadmap.md` | Step-by-step UI refactor initiatives. | Breaks the UI work into workstreams with detailed checklists for controllers, hooks, and services to reduce component responsibilities.【F:docs/ui-component-refactor-roadmap.md†L1-L169】 |
| `docs/chessboard_implementation_plan.md` | Reusable chessboard component extraction. | Focuses on creating a configurable board UI component with encapsulated state, styling options, and test guidance.【F:docs/chessboard_implementation_plan.md†L1-L33】 |
| `docs/review-domain-redesign-plan.md` | Harden review domain crate. | Introduces type-safe IDs, card aggregates, an opening graph, and unlock symmetry with implementation steps and testing guidance.【F:docs/review-domain-redesign-plan.md†L3-L74】 |
| `documentation/pgn-tutor-roadmap.md` | End-to-end PGN drill readiness. | Assesses the PGN import pipeline, scheduler, gateway, and UI gaps to reach a working drill tutor across services.【F:documentation/pgn-tutor-roadmap.md†L1-L93】 |

## Overlap and Dependency Analysis
- **Frontend architecture plans** (`ui-separation-plan`, `ui-component-refactor-roadmap`, `chessboard_implementation_plan`) all target the React UI’s overextended responsibilities. They share the goals of extracting controllers/services, codifying configuration contracts, and improving testability. The chessboard plan is effectively a detailed sub-track that fits inside the UI refactor roadmap.
- **Backend/domain initiatives** (`review-domain-redesign-plan`, portions of `pgn-tutor-roadmap`) focus on type safety, graph traversal, and scheduler integration. The PGN tutor roadmap depends on the review domain redesign to avoid rework when persisting cards and unlocks.
- **End-to-end readiness** (PGN tutor roadmap) acts as an integration umbrella that requires UI separation, domain redesign, and infrastructure services to land in sequence. Without the application layer refactors, the UI cannot consume real backend data; without the domain redesign, the scheduler integration risks schema churn.

## Simplified Unified Workstreams
The following three workstreams capture all existing plans while clarifying sequencing and ownership.

### 1. Application Layer & UI Readiness
**Goals:** Extract orchestration logic from React, deliver reusable components (including the chessboard), and expose testable contracts.

**Key Milestones:**
1. ✅ Establish `application/` layer scaffolding with services/controllers/view models as outlined in the separation plan. The scaffolding now ships symbol tokens and TypeScript contract definitions under `web-ui/src/application/`, giving follow-on work a stable import surface.【F:docs/ui-separation-plan.md†L39-L129】【F:web-ui/src/application/index.ts†L1-L26】
2. Execute UI refactor roadmap checklists, starting with PGN import and dashboard view models for quick wins, then session controller and command palette hooks.【F:docs/ui-component-refactor-roadmap.md†L63-L168】
3. Deliver the reusable chessboard base component with configuration API and documentation, integrating it into the new controllers/hooks.【F:docs/chessboard_implementation_plan.md†L1-L33】
4. Update UI tests to cover hooks/services headlessly and components via snapshots as prescribed across the UI plans.【F:docs/chessboard_implementation_plan.md†L22-L29】【F:docs/ui-component-refactor-roadmap.md†L19-L26】

**Dependencies:** Requires coordination with backend contracts (command palette, session APIs) as they stabilize through later workstreams.

### 2. Review Domain & Scheduler Foundation
**Goals:** Solidify core Rust crates so PGN imports and scheduling operate on safe, ergonomic abstractions.

**Key Milestones:**
1. Introduce type-safe identifiers and card aggregate constructors to remove `u64` ambiguity and encapsulate scheduling state.【F:docs/review-domain-redesign-plan.md†L11-L44】
2. Implement `OpeningGraph` adjacency structure and unlock symmetry while maintaining serializer compatibility.【F:docs/review-domain-redesign-plan.md†L45-L82】
3. Provide bridging adapters from importer outputs into the redesigned domain types, enabling importers to populate persistent stores without leaking invariants.【F:documentation/pgn-tutor-roadmap.md†L95-L142】
4. Expose scheduler HTTP service backed by the redesigned domain/store contracts, aligning API payloads with the gateway expectations.【F:documentation/pgn-tutor-roadmap.md†L143-L184】

**Dependencies:** Completes before full PGN drill integration to avoid mismatched schemas; informs UI contracts for session stats and unlock displays.

### 3. End-to-End PGN Drill Integration
**Goals:** Connect ingestion, scheduling, gateway, and UI into a working learner experience.

**Key Milestones:**
1. Build import-to-card pipeline using the redesigned domain types and persistent storage, producing learner-specific queues.【F:documentation/pgn-tutor-roadmap.md†L95-L142】
2. Align session gateway and web client contracts (session IDs, metrics), leveraging the new application layer hooks to consume backend data.【F:documentation/pgn-tutor-roadmap.md†L143-L184】
3. Wire the scheduler service into the gateway and UI flows, replacing demo data with live sessions managed by the `SessionController` abstraction.【F:docs/ui-separation-plan.md†L1-L94】【F:documentation/pgn-tutor-roadmap.md†L143-L184】
4. Deliver cross-stack smoke tests that ingest a PGN and validate the learner journey, ensuring all controllers/services collaborate correctly.【F:documentation/pgn-tutor-roadmap.md†L185-L193】

**Dependencies:** Depends on completion of workstream 1 (UI contracts) and workstream 2 (domain readiness).

## Implementation Guidance
- **Ticketing Strategy:** For each milestone, create linked issues referencing the originating documents so history remains traceable even as this unified roadmap becomes the source of truth.
- **Change Management:** When landing significant milestones (e.g., SessionController adoption, OpeningGraph introduction), update the legacy documents with completion notes or archive them after migrating actionable content here.
- **Communication:** Maintain this roadmap as the canonical status board—update milestone checkboxes, add links to merged PRs, and record deviations (e.g., if scheduler service ships before complete domain redesign, document the trade-offs).

## Maintenance Plan
1. Review this roadmap quarterly with maintainers; archive or compress underlying documents once their content is fully represented here.
2. Keep a changelog at the bottom of this file summarizing major updates and linking to relevant PRs for visibility.
3. Ensure onboarding materials (e.g., README) reference this roadmap so contributors know where to find the consolidated plan.

## Initial Changelog
- `v1.0` – Consolidated UI separation, UI refactor, chessboard, review domain redesign, and PGN tutor readiness plans into a single, sequenced roadmap (May 2024).
