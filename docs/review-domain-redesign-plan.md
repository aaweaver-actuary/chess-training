# Review Domain Redesign Plan

## Purpose
This document captures the implementation outline for improving the review domain crate. It summarizes the current pain points, proposes concrete actions, and provides a recommended delivery roadmap. The goals are to strengthen invariants, improve traversal and reuse of opening data, and make card scheduling safer and more ergonomic.

## Current Pain Points
- **Identifier ambiguity.** Many structs expose raw `u64` identifiers. Callers must remember which kind of ID they are manipulating, raising the risk of mixing positions, edges, cards, and learners.
- **Leaky scheduling state.** `StoredCardState` exposes every scheduling field for mutation. External services can accidentally violate SM-2 invariants when updating intervals, ease factor, streak, or review timestamps.
- **Bag-of-moves repertoire.** `Repertoire` stores a flat `Vec<RepertoireMove>` without adjacency structure, so downstream callers have to rebuild graphs to answer "what moves follow from this position?" or to detect duplicate prefixes.
- **Generic-overload for cards.** `Card<Id, Owner, Kind, State>` offers maximum flexibility but makes the default card shape noisy to work with. Each service reconstructs the same type aliases and constructors.
- **Unlock symmetry gaps.** Unlock records do not mirror the card kind variants, forcing additional branching when a workflow supports both openings and tactics.

## Proposed Core Abstractions
1. **CardAggregate**
   - Replace ad-hoc `Card<...>` usage with a concrete struct that fixes the default parameters (e.g., `CardId`, `LearnerId`, `CardKind`, `StoredCardState`).
   - Expose constructor helpers (`CardAggregate::new_opening`, `CardAggregate::new_tactic`) that validate inputs and hide hashing details.
   - Provide an `apply_review(grade, reviewed_at)` method that delegates to the SM-2 update logic internally, keeping scheduling invariants encapsulated.

2. **OpeningGraph**
   - Introduce `PositionId`, `EdgeId`, `MoveId` newtypes to wrap the existing deterministic hashes.
   - Model repertoire data as a DAG: `OpeningGraph { positions: HashMap<PositionId, PositionNode> }` where each node tracks outgoing `OpeningEdge` children.
   - Supply navigation helpers (`children(position_id)`, `parents(position_id)`, `path_to(position_id)`) so consumers can answer prefix questions directly.
   - Maintain serialization support (serde + Avro) by flattening the graph to the existing edge list shape when needed.

3. **ContentUnlocks**
   - Align unlock transport types with `CardKind` by introducing enums like `UnlockDetail::Opening(OpeningUnlock)` and `UnlockDetail::Tactic(TacticUnlock)`.
   - Provide conversion helpers from card aggregates to unlock records to reduce duplicated branching logic across services.

## Implementation Steps
1. **Type Safety Foundation**
   - Define newtype wrappers for every public identifier in the crate.
   - Update serialization schemas (serde, Avro) to accommodate the new wrapper types.
   - Refactor constructors and existing APIs to accept/return the newtypes.

2. **Scheduling Encapsulation**
   - Move `StoredCardState` fields behind getter methods and controlled update functions.
   - Implement `apply_review` using existing `ValidGrade` utilities while keeping interval and ease factor math internal.
   - Audit the crate for direct field mutations and migrate them to the new API.

3. **OpeningGraph Introduction**
   - Create `OpeningGraph` with adjacency maps derived from `RepertoireMove` inputs.
   - Add builders to ingest existing serialized repertoires and emit adjacency structures.
   - Implement query helpers for shared-prefix detection, DAG traversal, and deduplication.
   - Update call sites to rely on the graph abstraction instead of manual vector scans.

4. **CardAggregate Adoption**
   - Build concrete constructors for opening and tactic cards, leveraging the new type-safe IDs.
   - Provide serialization formats compatible with the previous generic card representation to ease migration.
   - Replace direct `Card<...>` usage across the crate and adjust unit tests accordingly.

5. **Unlock Symmetry**
   - Introduce typed unlock variants and conversion helpers.
   - Update existing unlock serialization and deserialization code.
   - Ensure downstream services receive consistent unlock payloads for both openings and tactics.

6. **Testing & Validation**
   - Expand unit tests to cover the newtype conversions, `apply_review` behavior, and DAG traversal helpers.
   - Add property tests where feasible (e.g., verifying that identical prefixes share node IDs and that applying review grades respects SM-2 constraints).
   - Update documentation and examples to showcase the new abstractions.

## Migration Considerations
- Provide adapter implementations (e.g., `From<CardAggregate>` for the legacy types) to support incremental adoption.
- Document schema changes for services consuming Avro/serde payloads and coordinate version bumps where required.
- Benchmark repertoire traversal before and after introducing `OpeningGraph` to confirm the performance gains.

## Open Questions
- Should we enforce immutability for card aggregates outside of scheduling updates (e.g., make the struct non-exhaustive)?
- What compatibility guarantees are required for serialized data persisted before the newtype introduction?
- Does the unlock redesign affect any cross-service contracts that require formal change management?

## Next Steps
- Socialize this plan with stakeholders and confirm prioritization.
- Create implementation tickets for each numbered step above, including test coverage requirements.
- Sequence the work to land type safety and scheduling encapsulation first, followed by the graph and unlock improvements.
