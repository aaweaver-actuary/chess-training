# In-memory card store layout

The in-memory implementation of the `CardStore` trait is organized into small helper modules so
that each responsibility remains focused:

- `mod.rs` wires the thread-safe store together and exposes the trait implementation.
- `positions.rs` handles canonicalization and deduplicated storage of chess positions.
- `edges.rs` manages opening edge upserts and collision detection.
- `cards.rs` creates cards, collects due reviews, and locates cards for updates.
- `reviews.rs` encapsulates the SM-2 style review math used during `record_review`.
- `unlocks.rs` deduplicates unlock records for opening moves.

Each helper exports only the functions consumed by `mod.rs`, and every helper is exhaustively
covered by unit tests in the same file to keep behaviour easy to audit.
