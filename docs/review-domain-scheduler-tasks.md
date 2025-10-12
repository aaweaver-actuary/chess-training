# Review Domain & Scheduler Foundation – Atomic Task List

1. **Milestone 1 – Identifier wrappers:** Inventory every usage of raw `u64` identifiers across `scheduler-core`, importer crates, and shared Avro schemas to catalogue the refactor surface and confirm compatibility requirements.
2. **Milestone 1 – Wrapper module implementation:** Create a `domain::ids` module exporting `PositionId`, `EdgeId`, `MoveId`, `CardId`, and conversion helpers, including serde derives and `From`/`TryFrom` implementations.
3. **Milestone 1 – Schema updates:** Regenerate Avro/serde schemas and fixture data so that serialized payloads emit the newtype identifiers while preserving wire compatibility assertions.
4. **Milestone 1 – Constructor refactor:** Update public constructors, builders, and factory functions to accept the wrapper types, adding compile-time assertions or clippy lints to prevent reintroduction of raw `u64` signatures.
5. **Milestone 1 – Migration sweep:** Replace raw identifier usage in dependent crates (scheduler, importer, review-domain) with the new wrappers and adjust unit/property tests to cover conversion success and failure paths.
6. **Milestone 1 – Card aggregate analysis:** Document current card state invariants (unlock rules, SM-2 fields) and note any implicit assumptions that the new aggregate must enforce.
7. **Milestone 1 – CardAggregate struct:** Implement the `CardAggregate` type with `new_opening`/`new_tactic` constructors, validation of initial state, and serde support.
8. **Milestone 1 – Review application helper:** Implement an `apply_review` method encapsulating SM-2 updates, ensuring grade inputs are validated and unit tested against legacy behaviour.
9. **Milestone 1 – Aggregate adoption:** Refactor existing mutation sites to call the new constructors and helpers, removing direct struct field edits and expanding regression/property tests for review updates.
10. **Milestone 2 – Graph design spike:** Define the adjacency data structures and builder interfaces for `OpeningGraph`, including error handling for malformed repertoires.
11. **Milestone 2 – Graph builder implementation:** Implement the builder that ingests `RepertoireMove` collections, materialises adjacency maps, and tracks metadata needed for traversal.
12. **Milestone 2 – Navigation API:** Add traversal helpers (`children`, `parents`, `path_to`, `roots`) with accompanying unit tests and property checks for DAG invariants.
13. **Milestone 2 – Legacy compatibility:** Provide a serializer that flattens the graph back into the legacy edge list and add snapshot tests proving parity with existing fixtures.
14. **Milestone 2 – Consumer migration:** Replace manual vector scans in repertoire consumers with `OpeningGraph` queries, updating benchmarks or profiling notes to confirm acceptable performance.
15. **Milestone 2 – Unlock symmetry design:** Draft symmetric unlock enums mapped to `CardKind`, documenting how they interact with existing unlock pipelines and telemetry.
16. **Milestone 2 – Unlock implementation:** Implement the enums, conversion helpers from `CardAggregate`, and serializer/deserializer updates, adjusting payload emitters accordingly.
17. **Milestone 2 – Unlock verification:** Add unit and integration tests that ensure unlock symmetry, including cases where tactics or openings are gated by multiple prerequisites.
18. **Milestone 3 – Importer contract audit:** Document importer artefact shapes and determine the mapping into redesigned domain types, including deterministic identifier strategies.
19. **Milestone 3 – Adapter scaffolding:** Implement a streaming adapter that converts importer outputs into domain structs, handling incremental ingestion and validation errors.
20. **Milestone 3 – Adapter validation:** Add validation layers that enforce identifier uniqueness, unlock integrity, and SM-2 invariants before persistence.
21. **Milestone 3 – Store integration:** Extend the card-store integration to persist adapted openings/cards using the new aggregates, including transaction boundaries and retry semantics.
22. **Milestone 3 – Unlock logging:** Ensure unlock creation emits the expected audit/log records without leaking internal state, updating telemetry as necessary.
23. **Milestone 3 – End-to-end tests:** Build integration tests that exercise importer → adapter → store flows, asserting schema compatibility and fixture parity with legacy paths.
24. **Milestone 3 – Failure-path coverage:** Add negative tests that simulate invalid importer payloads and confirm the adapter rejects them without mutating persistent state.
25. **Milestone 4 – API contract definition:** Align scheduler HTTP payloads with gateway expectations, producing OpenAPI/JSON schema docs and example fixtures.
26. **Milestone 4 – Queue endpoint:** Implement the `/queue` handler backed by redesigned store contracts, including pagination, filtering, and due-card computation.
27. **Milestone 4 – Grade endpoint:** Implement the `/grade` handler that accepts review submissions, applies `CardAggregate::apply_review`, and returns updated scheduling metadata.
28. **Milestone 4 – Store bridge:** Build the bridge translating persisted card-store records into scheduler-ready items, ensuring unlocks and scheduling state stay in sync.
29. **Milestone 4 – Contract tests:** Create contract or pact-style tests validating request/response parity with the Node gateway, covering session IDs, timing metrics, and error payloads.
30. **Milestone 4 – Operational readiness:** Add smoke tests or health-check endpoints, plus observability hooks (metrics/logging) needed for deployment confidence.
