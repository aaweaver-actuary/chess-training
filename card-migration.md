# Step 1: Analysis & Planning

## 1. Inventory Card Definitions

### Card-related types, traits, and functions in `review-domain`
- `crates/review-domain/src/card_kind.rs`:
  - `CardKind` enum (Opening, Tactic)
- `crates/review-domain/src/opening/card.rs`:
  - `OpeningCard` struct
- `crates/review-domain/src/tactic.rs`:
  - `TacticCard` struct
- `crates/review-domain/src/card_aggregate.rs`:
  - `Card` struct (generic over ID, Kind, State)
- `crates/review-domain/src/grade/valid_grade.rs`, `card_state.rs`, etc.:
  - `StoredCardState`, `ReviewRequest`, and related types
- Traits: Standard derives (`Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`)
- Functions: Constructors, conversions, helpers (see above files)

### Duplicate or parallel card definitions in other subcrates
- `crates/card-store/src/model.rs`:
  - Type aliases: `Card`, `CardKind`, `OpeningCard`, `TacticCard`, `CardMap`, `EdgeMap`
  - May wrap or re-export `review-domain` types, but not always 1:1
- `crates/card-store/src/memory/cards.rs`, `edges.rs`, etc.:
  - Use the above aliases and sometimes define their own helpers
- `crates/scheduler-core/src/domain/card_kind.rs`, `sm2_state.rs`, etc.:
  - May define their own enums or state types for cards
- `apps/session-gateway/`, `web-ui/`:
  - May use DTOs or import from `card-store`/`review-domain`

## 2. Dependency Mapping

- `crates/card-store/src/model.rs` and related modules:
  - Use local type aliases for cards, often shadowing or duplicating `review-domain` types
  - Example: `pub type Card = GenericCard<u64, String, CardKind, StoredCardState>;`
- `crates/card-store/src/memory/cards.rs`, `edges.rs`, etc.:
  - Use these aliases throughout for storage and logic
- `crates/scheduler-core/src/domain/`:
  - May define or use their own card state/kind types (e.g., `CardKind`, `Sm2State`)
- Conversions/wrappers:
  - Look for `From`, `TryFrom`, or custom conversion impls in `model.rs`, `state_bridge.rs`, and related files
- Serialization:
  - Check for custom (de)serialization logic in `model.rs`, `storage.rs`, and any API boundary modules

---

# Step 2: Refactoring Plan

## 1. Unify Imports
- Update all subcrates to import card types directly from `review-domain`.

## 2. Remove Duplicates
- Delete or refactor any duplicate card definitions in other subcrates.

## 3. Update Type Usages
- Refactor all code to use the canonical `review-domain` card types.
- Update function signatures, struct fields, and trait bounds as needed.

## 4. Migration of Serialization/Deserialization
- Ensure all serialization (serde) derives and helpers are present on the canonical types.
- Update any custom (de)serialization logic.

## 5. Test & Validate
- Update and run all tests to ensure correctness.
- Add/adjust tests for edge cases and integration points.

---

# Step 3: Step-by-Step Execution

## 1. Step 1: Inventory and Mapping
- List all card types in `review-domain` and all usages/duplicates elsewhere.

## 2. Step 2: Update Imports
- In each subcrate, update imports to use `review-domain` card types.

## 3. Step 3: Remove Duplicates
- Remove or refactor duplicate card types in other subcrates.

## 4. Step 4: Refactor Usages
- Update all code to use the canonical types, fixing any type errors.

## 5. Step 5: Serialization/Deserialization
- Ensure all (de)serialization works with the unified types.

## 6. Step 6: Test and Validate
- Run and fix all tests, ensuring full coverage and correctness.

---

# Migration Checklist: Atomic, Parallelizable Steps

- [ ] 1. Update all imports in `crates/card-store/src/model.rs` to use card types directly from `review-domain`.
- [ ] 2. Remove or refactor all type aliases for cards in `crates/card-store/src/model.rs` (e.g., `Card`, `CardKind`, `OpeningCard`, `TacticCard`).
- [ ] 3. Refactor all usages of card types in `crates/card-store/src/memory/cards.rs` to use `review-domain` types directly.
- [ ] 4. Refactor all usages of card types in `crates/card-store/src/memory/edges.rs` to use `review-domain` types directly.
- [ ] 5. Refactor all usages of card types in `crates/card-store/src/memory/reviews.rs` to use `review-domain` types directly.
- [ ] 6. Refactor all usages of card types in `crates/card-store/src/memory/unlocks.rs` to use `review-domain` types directly.
- [ ] 7. Update all serialization/deserialization derives and helpers in `card-store` to match the canonical types from `review-domain`.
- [ ] 8. Remove any duplicate or obsolete card-related structs/enums in `card-store` after migration.
- [ ] 9. Update all imports and usages of card types in `crates/scheduler-core/src/domain/card_kind.rs` and related files to use `review-domain` types.
- [ ] 10. Refactor any conversion or wrapper logic (e.g., `From`, `TryFrom`, custom impls) in `card-store` and `scheduler-core` to operate on `review-domain` types only.
- [ ] 11. Update all tests in `card-store` and `scheduler-core` to use the unified card types from `review-domain`.
- [ ] 12. Update any DTOs or API boundary types in `apps/session-gateway/` and `web-ui/` to use or map directly to `review-domain` card types.
- [ ] 13. Remove any remaining references to old/duplicate card types in the codebase.
- [ ] 14. Run and fix all tests, ensuring full coverage and correctness after migration.
- [ ] 15. Document any breaking changes or migration notes for downstream consumers.
