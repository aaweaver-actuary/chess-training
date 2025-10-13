# Web UI Source

```mermaid
flowchart TD
    src_dir["src/"] --> application["application/"]
    src_dir --> clients["clients/"]
    src_dir --> components["components/"]
    src_dir --> fixtures["fixtures/"]
    src_dir --> pages["pages/"]
    src_dir --> services["services/"]
    src_dir --> state["state/"]
    src_dir --> styles["styles/"]
    src_dir --> types["types/"]
    src_dir --> utils["utils/"]

    classDef leaf fill:#f0f7ff,stroke:#3a6ea5
    class application,clients,components,fixtures,pages,services,state,styles,types,utils leaf;
```

The React application is organised into feature-focused directories:

* `application/` – view models and controllers that prepare data for pages and components.
* `clients/` – API clients that talk to the session gateway.
* `components/` – Presentational and container components.
* `fixtures/` – Mock data used in tests and local development.
* `pages/` – Route-level components.
* `services/` – Data-fetching and orchestration utilities.
* `state/` – Global state management helpers.
* `styles/` – Shared style primitives.
* `types/` – TypeScript contracts shared within the UI.
* `utils/` – Generic helpers and hooks.

Each directory includes tests where applicable in an adjacent `__tests__` folder. When introducing a new feature area, add a README describing its purpose and key conventions.
