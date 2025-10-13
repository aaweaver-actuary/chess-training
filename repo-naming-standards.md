# Repository Naming Standards

> **Purpose:** Provide a single, authoritative naming guide for chess-training contributors, derived from the findings in [`rust-naming-audit.md`](./rust-naming-audit.md). Follow these conventions for all new code, when reviewing existing code, and when planning refactors.

## Table of Contents
1. [Core Principles](#core-principles)
2. [Verb Guidelines](#verb-guidelines)
3. [Structs, Enums, and Type Aliases](#structs-enums-and-type-aliases)
4. [Traits and Implementations](#traits-and-implementations)
5. [Modules and Files](#modules-and-files)
6. [Cross-Crate Consistency](#cross-crate-consistency)
7. [Glossary Maintenance](#glossary-maintenance)
8. [Refactoring Checklist](#refactoring-checklist)

---

## Core Principles
[Back to Top](#repository-naming-standards)

- **Be descriptive and domain-focused.** Favor names that capture what a type or function *does in the chess-training domain*, not how it is implemented. (Example: `SchedulerConfig` rather than `ConfigOptions`).
- **Prefer nouns for data, verbs for actions.** This maintains clarity between data containers (e.g., `UnlockDetail`) and operations (e.g., `record_unlock`).
- **Avoid duplication.** When introducing new items, search for existing equivalents to prevent multiple names for the same concept (e.g., avoid introducing a third variant of `CardStore`).
- **Converge on established prefixes/suffixes.** Align new work with the most common existing patterns cataloged in the audit (`Config`, `Record`, `Id`, `Store`, etc.).

## Verb Guidelines
[Back to Top](#repository-naming-standards)

Use verbs consistently to signal how an API behaves. When adding a new function, pick the verb by intent and do not mix alternatives without reason.

| Intent | Preferred Verb(s) | Avoid / Notes | Example |
| ------ | ----------------- | ------------- | ------- |
| Build a new value without side effects | `new_*`, `build_*`, `create_*` | Avoid `make_*` and `into_*` for constructors. `into_*` implies type conversion that consumes `self`. | `build_ingest_config`. |
| Convert types while consuming the source | `into_*` | Only use when the method takes ownership and converts to another type. | `EdgeInput::into_edge`. |
| Convert types without consuming | `as_*`, `to_*` | Follow Rust idioms: `to_*` returns owned data, `as_*` returns borrowed/cast views. | `Grade::to_u8`, `Grade::as_u8`. |
| Persist or update storage | `upsert_*`, `record_*` | Avoid mixing `store_*`, `insert_*`, `save_*` for the same action. Pick the dominant verb in the module/crate and use it everywhere. | `upsert_canonical_position`, `record_unlock`. |
| Read-only queries | `get_*`, `load_*`, `fetch_*` | Prefer a single verb per module (`get` vs `fetch`). Avoid `collect_*` unless building a derived collection. | `get_due_cards_for_owner`. |
| Queue or workflow building | `build_*`, `prepare_*` | Avoid mixing `build_queue` with queue-length names; expose `queue_length` for size checks. | `build_queue_for_day`, `queue_length`. |

## Structs, Enums, and Type Aliases
[Back to Top](#repository-naming-standards)

- **Structs and enums use singular nouns** describing the concept (`CardAggregate`, `UnlockRecord`). If a specialized constructor exists, suffix with the differentiator (`new_opening`).
- **Type aliases clarify specialization.** Use suffixes like `_Map`, `_Set`, `_Id` to communicate the alias purpose (`CardMap`, `UnlockSet`). When introducing new aliases, follow the strongest existing pattern or rename nearby aliases to match.
- **Distinguish overlapping names.** If two items share a root name but serve different roles, add disambiguating adjectives (`GenericCardAggregate` vs. `StoredCardAggregate`).
- **Hash/ID helpers.** Prefer `hash_*_id` or `build_*_id` naming to make intent explicit and align with deterministic ID generation across crates.

## Traits and Implementations
[Back to Top](#repository-naming-standards)

- **Trait names describe capability in noun form.** (`CardStore`, `SchedulerStore`, `Storage`). When two traits could collide in scope, rename to clarify ownership (`ReviewCardStore` vs. `SchedulerStore`).
- **Method verbs on traits follow module rules.** If the trait expresses persistence, ensure all implementations use `upsert_*`/`record_*` consistently.
- **Suffix `Error`, `Result`, or `Handle` for helpers** that encapsulate state machines or result types (`StoreError`, `UnlockHandle`).
- **Builders and facades.** Use `Facade`, `Builder`, or `Factory` only for types that orchestrate multiple subsystems, and ensure methods reinforce their role (`SchedulerFacade::new`).

## Modules and Files
[Back to Top](#repository-naming-standards)

- **Module names are plural nouns or domain nouns.** (`config`, `store`, `queue`, `grade`). Do not name modules with verbs.
- **Match file names to the main type.** If a file contains `sm2_state.rs`, the primary type should be `Sm2State`.
- **Re-export modules deliberately.** Use `pub mod`/`pub use` to expose nouns that mirror their file names, keeping the public API predictable.

## Cross-Crate Consistency
[Back to Top](#repository-naming-standards)

- **Align shared concepts.** If a name appears in multiple crates, use the same spelling and suffix (`CardStore` vs. `SchedulerCardStore`). Consider renaming conflicting traits per the audit recommendations to avoid double imports.
- **Queue terminology.** Export `queue`-related functions with matching verbs across crates (`queue_length`, `build_queue`). Avoid introducing mismatched verb+noun hybrids.
- **In-memory stores.** Standardize on `InMemory*Store` (`InMemoryCardStore`, `InMemoryImportStore`, `InMemorySchedulerStore`).
- **Unlock flow.** Harmonize verbs between crates so card-store and scheduler both use `record_unlock` or `upsert_unlock`, not a mix of `insert`/`record`.

## Glossary Maintenance
[Back to Top](#repository-naming-standards)

- Update [`docs/rust-structs-glossary.md`](./docs/rust-structs-glossary.md) whenever you add, rename, or remove structs/enums.
- Cross-reference this document and the glossary during reviews to catch drift early.
- Document legacy names slated for refactor so the team understands transitional states.

## Refactoring Checklist
[Back to Top](#repository-naming-standards)

Use this checklist when touching names:

1. **Audit existing usage.** Search the repo (e.g., `rg "queue_length"`) to understand current patterns before changing anything.
2. **Select verbs/nouns per this guide.** Ensure new names align with the tables and conventions above.
3. **Update related items together.** When renaming a trait, adjust implementations, docs, and re-exports in the same change.
4. **Refresh documentation.** Update this standard and the glossary when the repo’s naming expectations evolve.
5. **Verify tests.** Run `make test` after refactors to ensure no behavior regressed while names shifted.

[Back to Top](#repository-naming-standards)
