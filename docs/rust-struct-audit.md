# Rust Struct Audit

## Overview
The review examined the structs catalogued in `docs/rust-structs-glossary.md` to determine where duplicated responsibilities might be simplified. Each family of look-alike structs was compared across crates to confirm whether they intentionally diverge for layering reasons or whether they should converge on a common representation.

## Separation Decisions
- **StoredCardState vs. Sm2State** – These structs target different lifecycle stages: the card-store persists a minimal snapshot, while the scheduler mutates richer runtime counters. Joining them would either bloat storage with scheduler-only fields or lose per-review analytics inside the scheduler.
- **UnlockDetail vs. SchedulerUnlockDetail** – Storage only needs to log which opening edge unlocked, whereas the scheduler must track card UUIDs and parent prefixes to enforce pacing constraints. Mixing them would couple persistence to queue-building details.
- **OpeningCard vs. SchedulerOpeningCard** – Persisted cards only rely on a normalized `edge_id`, but the scheduler groups cards by parent prefix so it can stagger unlocks. A single struct cannot satisfy both without leaking implementation details.
- **TacticCard vs. SchedulerTacticCard** – Review cards must reference tactic content IDs; the scheduler merely needs a marker so tactic-specific pacing rules can evolve independently.
- **ChessPosition vs. importer Position** – The review domain enforces strict FEN validation and uses a different hashing salt than the PGN importer, which focuses on serde friendliness and namespace-specific determinism. Sharing a struct would force one side to compromise on guarantees.
- **SchedulerConfig vs. SchedulerConfigDto/Patch** – The Rust configuration enforces invariants and defaults, while the wasm DTOs expose optional fields to JavaScript callers. A merged struct would either weaken validation or break the wasm contract.
- **InMemoryCardStore/InMemoryStore vs. ImportInMemoryStore** – Each in-memory helper satisfies a different trait with unique data types and concurrency requirements. Generalising them would introduce unnecessary generics and feature flags into otherwise simple test fixtures.

## Recommended Consolidations
### Consolidate `OpeningCard` and `UnlockDetail`
Both structs live in `review-domain`, expose identical data (`edge_id`), and represent the same conceptual link back to an opening edge—one for the card payload and one for unlock history. Replacing them with a shared `OpeningEdgeHandle` (or reusing `OpeningCard`) would eliminate duplication and ensure that future metadata additions only need to be made in one place.

**Short-term impact**
- Update the unlock record type aliases and storage models to reference the shared struct.
- Adjust serde derivations and constructor helpers so unlock logs and card payloads continue to round-trip without breaking external APIs.
- Touch card-store tests and scheduler-to-storage mapping code to use the new helper; because both structs already expose the same field, the mechanical change should be low risk and mostly renaming.

### Harden identifier usage
While not an outright merge, several structs that carry raw `u64` identifiers (`OpeningCard`, `UnlockDetail`, `OpeningEdge`) duplicate the semantics already captured by the `EdgeId` newtype. After consolidating the opening-edge handle, migrate the shared struct to wrap `EdgeId`. This keeps conversions consistent and reduces accidental cross-wiring between unrelated ID domains.

**Short-term impact**
- Introduce conversion helpers between `EdgeId` and the shared handle for any serde boundaries.
- Update call sites that currently pass raw `u64` values to use `EdgeId` or explicit `.get()` calls, improving type safety without large behavioural changes.

## Next Steps
Prioritise the opening-edge handle consolidation first; it offers the clearest win with minimal blast radius. Once complete, evaluate whether additional metadata (for example, SAN/parent context) should live alongside the shared handle to support future unlock analytics before expanding the scope to tactics or other card kinds.

## Secondary Review

### Alignment with the existing recommendations
- **Separation calls hold up in code.** `StoredCardState` remains persistence-oriented with invariants like `NonZeroU8` intervals and mutation helpers, whereas `Sm2State` keeps richer runtime counters such as lapses and reviews that the scheduler needs to evolve pacing logic.【F:crates/review-domain/src/card_state.rs†L14-L101】【F:crates/scheduler-core/src/domain/sm2_state.rs†L4-L34】 Collapsing them would either leak scheduler-only stages into storage or weaken storage validation.
- **Unlock payloads diverge intentionally.** `UnlockDetail` is only an opening-edge reference today, while `SchedulerUnlockDetail` carries card UUIDs plus optional parent prefixes so queueing logic can batch unlocks. They feed different generic `UnlockRecord` instantiations, so retaining the split avoids coupling storage formats to scheduler queue shapes.【F:crates/review-domain/src/unlock.rs†L5-L67】【F:crates/scheduler-core/src/domain/mod.rs†L28-L38】 
- **Scheduler card kinds encapsulate extra context.** `OpeningCard` is a plain edge pointer but `SchedulerOpeningCard` bundles the parent prefix hint the pacing engine requires, confirming that a shared struct would expose scheduler-specific fields to persistence callers.【F:crates/review-domain/src/opening/card.rs†L6-L30】【F:crates/scheduler-core/src/domain/card_kind.rs†L3-L32】 The same logic applies to the tactic markers noted in the original audit.
- **Domain positions still need bespoke validation.** `ChessPosition` refuses malformed FEN strings up front, while the importer `Position` prioritises serde friendliness and deterministic hashing without extra checks, matching the audit’s rationale for keeping both representations.【F:crates/review-domain/src/position.rs†L5-L145】【F:crates/chess-training-pgn-import/src/model.rs†L13-L117】
- **DTO vs. config split is justified.** Although not revisited in depth here, the scheduler crate continues to enforce invariants internally while the wasm bindings expose optional fields; sharing a struct would force optionality into Rust code that presently relies on defaults, validating the original recommendation.

### Additional streamlining opportunities
- **Follow through on the opening-edge handle merger.** Both `OpeningCard` and `UnlockDetail` are now single-field wrappers around `edge_id`; extracting a shared `OpeningEdgeHandle` (or inlining `OpeningCard`) would remove duplication and establish a single place to hang future metadata like SAN context.【F:crates/review-domain/src/opening/card.rs†L6-L30】【F:crates/review-domain/src/unlock.rs†L28-L67】
- **Adopt `EdgeId`/`PositionId` wrappers at the boundaries.** The review domain already defines newtypes for identifiers, yet `OpeningEdge`, `OpeningEdgeRecord`, `RepertoireEdge`, and the two single-field structs above still expose raw `u64`s. Refactoring them to accept and return the wrappers would tighten type safety and stop accidental cross-wiring between identifier domains without changing serialization defaults.【F:crates/review-domain/src/ids.rs†L5-L76】【F:crates/review-domain/src/opening/edge.rs†L3-L52】【F:crates/chess-training-pgn-import/src/model.rs†L39-L117】
- **Rename once the handle exists.** After consolidation, rename the shared struct to something intention-revealing (for example, `OpeningEdgeRef` or `OpeningEdgeHandle`) so callers immediately understand it is just a pointer payload. Aligning the scheduler unlock type alias and any serde helpers around the new name will reinforce that there is only one canonical opening-edge reference.
- **Backfill glossary and tests.** When the shared handle lands, update `docs/rust-structs-glossary.md` and extend unlock/card-store regression tests to cover the newtype conversions. This keeps the documentation trustworthy and ensures the migration does not silently reintroduce raw `u64` usage.
