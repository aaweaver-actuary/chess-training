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
