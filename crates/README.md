# Rust Crates

The `crates/` directory contains the Rust libraries that implement the chess training domain, scheduling algorithms, and supporting utilities. Each crate is a first-class Cargo package and can be built or tested independently.

| Crate | Description |
| --- | --- |
| `review-domain/` | Canonical domain types shared across services (cards, positions, unlock records, hashes). |
| `card-store/` | Persistence abstractions and in-memory store for positions, edges, cards, and unlocks. |
| `scheduler-core/` | SM-2 scheduling engine, unlock policy, and queue construction logic. |
| `chess-training-pgn-import/` | PGN ingestion pipeline that produces canonical positions, opening edges, and tactics. |

Each crate contains a README detailing its API surface, configuration, and extension points. Use `cargo test -p <crate>` to execute unit tests for a specific package.
