# State

```mermaid
flowchart TD
    state_dir["state/"] --> session_store["sessionStore.ts"]
    state_dir --> tests["__tests__/sessionStore.test.ts"]

    classDef leaf fill:#f0fff4,stroke:#2e7d32
    class session_store,tests leaf;
```

State management helpers, stores, and hooks that coordinate application-wide data. Keep modules here framework-friendly and ensure tests cover reducers or selectors. `sessionStore.ts` exposes the central observable store used by `SessionRoutes`.
