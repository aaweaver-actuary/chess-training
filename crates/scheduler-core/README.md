# Scheduler core overview

This crate now favors a modular layout so that the SM-2 review math, queue building, and storage
abstractions remain digestible:

- `config.rs` defines `SchedulerConfig` defaults.
- `grade.rs` documents the supported review grades.
- `domain.rs` contains core card data structures shared across modules.
- `errors.rs` exposes `SchedulerError`.
- `store.rs` provides the persistence trait plus the test-friendly `InMemoryStore` implementation.
- `sm2.rs` implements the spaced-repetition transitions in small helper functions.
- `queue.rs` gathers due cards and unlocks new openings without exceeding five statements per
  public function.
- `scheduler.rs` orchestrates the high-level API for applications.

Every module hosts unit tests beside its implementation so the behaviour stays transparent.
