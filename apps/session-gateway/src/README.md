# Session Gateway Source

```mermaid
flowchart TD
    src_dir["src/"] --> index_ts["index.ts\nbootstrap"]
    src_dir --> config_ts["config.ts"]
    src_dir --> server_ts["server.ts"]
    src_dir --> session_service["sessionService.ts"]
    src_dir --> broadcaster["broadcaster.ts"]
    src_dir --> clients_dir["clients/"]
    src_dir --> stores_dir["stores/"]
    src_dir --> types_ts["types.ts"]

    classDef leaf fill:#f5f0ff,stroke:#5d3fd3
    class index_ts,config_ts,server_ts,session_service,broadcaster,clients_dir,stores_dir,types_ts leaf;
```

The source tree keeps HTTP wiring, business logic, and integration points isolated from one another.

* `index.ts` – bootstraps the Express app, WebSocket server, scheduler client, and session store.
* `broadcaster.ts` – manages WebSocket fan-out to subscribers by session id.
* `clients/` – scheduler client implementations (currently HTTP based).
* `config.ts` – runtime configuration derived from environment variables.
* `server.ts` – Express routes, validation, and WebSocket upgrade handling.
* `sessionService.ts` – core orchestration for session lifecycle and stats.
* `stores/` – pluggable session persistence adapters; defaults to an in-memory map.
* `types.ts` – TypeScript contracts shared across modules and tests.

All exported functions and types include JSDoc comments to ensure IDEs and generated documentation reflect the current behaviour. When adding new modules, include a brief description here outlining their role within the service.
