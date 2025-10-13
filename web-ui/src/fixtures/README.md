# Fixtures

```mermaid
flowchart TD
    fixtures_dir["fixtures/"] --> sample_snapshot["sampleSnapshot.ts"]

    classDef leaf fill:#fff6f0,stroke:#d17d00
    class sample_snapshot leaf;
```

Static data used in tests, stories, and manual QA. Keep fixtures lightweight and representative of real-world payloads returned by the session gateway. `sampleSnapshot.ts` seeds the UI with pre-built sessions for local development.
