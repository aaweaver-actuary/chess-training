# Services

```mermaid
flowchart TD
    services_dir["services/"] --> review_planner["ReviewPlanner.ts"]
    services_dir --> tests["__tests__/ReviewPlanner.test.ts"]

    classDef leaf fill:#f5f0ff,stroke:#5d3fd3
    class review_planner,tests leaf;
```

Data-fetching and orchestration utilities that mediate between the UI and external APIs. Services should return typed results and remain framework agnostic so they can be reused across components and tests.

`ReviewPlanner.ts` produces recommendations, unlock projections, and session summaries from raw fixture data. Scenario-based tests ensure the planner remains deterministic.
