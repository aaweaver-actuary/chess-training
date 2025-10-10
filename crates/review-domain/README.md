# review-domain

`review-domain` defines the core data structures used across the chess training workspace. These types are designed to be serialization-friendly and to share deterministic identifiers so that services can communicate without bespoke translation layers.

## Highlights

* Generic card representation that parameterises the owner, card kind, and scheduling state.
* Opening and tactic payloads with deterministic hashing helpers.
* Unlock record types for progressive content releases.
* Study stage, review grade, and validated grade enums reused by the scheduler and storage layers.

## Usage

Add the crate to your `Cargo.toml` within the workspace:

```toml
review-domain = { path = "../review-domain" }
```

Then import the types you need:

```rust
use review_domain::{Card, OpeningCard, ReviewGrade, StudyStage};
```

Refer to the inline documentation (`cargo doc --open -p review-domain`) for detailed descriptions of each type.
