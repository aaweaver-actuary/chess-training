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

## Secondary Review Findings

### Assessment of separation decisions
- The split between `StoredCardState` in the persistence layer and the scheduler's `Sm2State` remains appropriate: the stored form keeps a compact `NonZeroU8` interval and last-review timestamp optimised for durable storage, whereas the runtime state adds stage transitions, lapse counters, and wider integer widths that only the scheduler needs while preparing queues.【F:crates/review-domain/src/card_state.rs†L14-L101】【F:crates/scheduler-core/src/domain/sm2_state.rs†L4-L33】
- `UnlockDetail` and `SchedulerUnlockDetail` rightfully diverge because the scheduler must capture card UUIDs and parent prefixes when enforcing daily pacing, while the storage layer only needs the edge identifier that triggered the unlock.【F:crates/review-domain/src/unlock.rs†L28-L67】【F:crates/scheduler-core/src/domain/mod.rs†L28-L38】
- `OpeningCard` versus `SchedulerOpeningCard` reflects a similar layering boundary: persisted cards track which deterministic edge they represent, whereas the scheduler groups openings by parent prefix to avoid unlocking multiple siblings on the same day.【F:crates/review-domain/src/opening/card.rs†L6-L37】【F:crates/scheduler-core/src/domain/card_kind.rs†L3-L33】【F:crates/scheduler-core/src/queue.rs†L41-L103】
- Retaining both `TacticCard` and the zero-sized `SchedulerTacticCard` keeps the scheduler free from tactic identifiers while storage still records the puzzle ID used to resolve content later.【F:crates/review-domain/src/tactic.rs†L6-L38】【F:crates/scheduler-core/src/domain/card_kind.rs†L20-L33】【F:crates/scheduler-core/src/queue.rs†L95-L132】
- The stricter `ChessPosition::new` constructor validates FEN structure and salts its hashes differently than the importer’s serde-friendly `Position::new`, so merging them would either loosen validation guarantees or break deterministic IDs within the importer pipeline.【F:crates/review-domain/src/position.rs†L5-L144】【F:crates/chess-training-pgn-import/src/model.rs†L13-L118】
- Keeping `SchedulerConfig` separate from the wasm-facing DTOs preserves defaulting and validation for Rust callers while still allowing JavaScript clients to send partial patches through serde-derived option fields.【F:crates/scheduler-core/src/config.rs†L1-L33】【F:crates/scheduler-wasm/src/config.rs†L1-L69】
- The distinct in-memory stores serve different contracts: `InMemoryCardStore` wraps thread-safe locks and review-domain models to satisfy the `CardStore` trait, whereas `ImportInMemoryStore` is a single-threaded `Storage` implementation tailored to ingestion metrics and BTree collections.【F:crates/card-store/src/memory/in_memory_card_store.rs†L1-L158】【F:crates/chess-training-pgn-import/src/storage.rs†L1-L147】

### Additional streamlining opportunities
- Consolidate `OpeningCard` and `UnlockDetail` into a single `OpeningEdgeHandle` (or reuse `OpeningCard`) that stores an `EdgeId`, then migrate callers accordingly. This eliminates duplicate structs that currently only wrap a `u64` edge identifier and encourages wider adoption of the existing newtype for safer ID handling.【F:crates/review-domain/src/opening/card.rs†L6-L37】【F:crates/review-domain/src/unlock.rs†L28-L67】【F:crates/review-domain/src/ids.rs†L59-L76】
- Extend the `EdgeId` and `PositionId` newtypes into other opening models—such as `OpeningEdge` and `RepertoireEdge`—so related structs stop transporting raw `u64` identifiers. The stronger typing would make cross-domain conversions explicit and reduce accidental mix-ups during future refactors.【F:crates/review-domain/src/opening/edge.rs†L6-L41】【F:crates/chess-training-pgn-import/src/model.rs†L39-L118】【F:crates/review-domain/src/ids.rs†L59-L76】
- Introduce conversion helpers between `StoredCardState` and `Sm2State` (for example, via `From`/`TryFrom` implementations) to document the mapping rules in one place and prevent the two representations from drifting apart as scheduling logic evolves.【F:crates/review-domain/src/card_state.rs†L30-L101】【F:crates/scheduler-core/src/domain/sm2_state.rs†L4-L33】
