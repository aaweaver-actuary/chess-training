# Review Domain Card State Invariants

This note documents the guard rails that exist around the `StoredCardState`
structure today. The goal is to make the requirements explicit before replacing
the legacy `Card<...>` usage with the forthcoming `CardAggregate` API. Having a
single source of truth makes it easier to translate these guarantees into
constructors and review helpers.

## Scheduling Field Guarantees

The SM-2 implementation inside `StoredCardState` enforces three critical
constraints:

1. **Intervals never reach zero.** The type uses `NonZeroU8` and clamps the next
   interval to at least one day for every grade. Future constructors must reject
   zero-day intervals when bootstrapping new cards.
2. **Ease factor is clamped.** `next_ease_factor` limits the value to the
   `[1.3, 2.8]` range. Any aggregate constructor must ensure the initial ease
   sits within the same bounds and that updates continue to clamp values.
3. **Due date derives from the last review.** A successful review sets
   `due_on = last_reviewed_on + interval`. Persisted state that violates this
   relationship should be rejected during loading or migration.

## Review Flow Expectations

* **Streak resets on failure.** `next_streak` drops the consecutive-correct
  counter to `0` for grades `0` and `1`. Aggregates must rely on the helper
  rather than mutating the streak manually.
* **Last review timestamp is optional.** Newly created cards can omit
  `last_reviewed_on`, but once present the date must never exceed `due_on`.
* **Due date monotonicity.** When `last_reviewed_on` exists, the due date may not
  move backwards. If migration tooling encounters a regression it should either
  fix the due date or flag the card as corrupt.

## Unlock and Stage Implications

Unlock workflows rely on the card state to decide when new openings become
available. Because due dates and streaks drive queue selection, any aggregate
constructor must preserve the following assumptions:

* Unlock checks only see cards that are **at least one day into the future**
  after a successful review. Cards due immediately could re-enter the queue
  before the unlock pipeline records the progression.
* Correct answers are the only path to a consecutive streak increment. Unlock
  telemetry assumes a streak of `n` corresponds to `n` straight correct grades.

## Migration Checklist

When implementing `CardAggregate::new_opening` and related helpers ensure that:

- Initial state obeys the same bounds (`interval >= 1`, `ease_factor` within the
  SM-2 range, due date not in the past for the given creation day).
- Review application routes through the existing SM-2 helpers so streaks, ease,
  and intervals stay in sync.
- Deserialisation of legacy records validates these invariants before exposing
  them to callers.
