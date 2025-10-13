# Clients

```mermaid
flowchart TD
    clients_dir["clients/"] --> session_gateway["sessionGateway.ts"]
    clients_dir --> tests["__tests__/sessionGateway.test.ts"]

    classDef leaf fill:#f0f7ff,stroke:#3a6ea5
    class session_gateway,tests leaf;
```

HTTP and WebSocket clients used by the UI to communicate with backend services. Modules here should expose typed interfaces and keep network concerns isolated from components.

`sessionGateway.ts` currently provides a typed wrapper around the session gateway REST API. Tests ensure the client handles success and error payloads consistently.
