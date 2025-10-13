# Tests

```mermaid
flowchart TD
    tests_dir["tests/"] --> config_test["config.test.ts"]
    tests_dir --> gateway_test["sessionGateway.test.ts"]
    tests_dir --> broadcaster_test["broadcaster.test.ts"]
    tests_dir --> client_test["httpSchedulerClient.test.ts"]
    tests_dir --> store_test["inMemoryStore.test.ts"]
    tests_dir --> service_test["sessionService.test.ts"]
    tests_dir --> index_test["index.test.ts"]

    classDef leaf fill:#eef9f2,stroke:#2e7d32
    class config_test,gateway_test,broadcaster_test,client_test,store_test,service_test,index_test leaf;
```

Integration tests targeting the session gateway live in this directory. Tests are executed with Vitest via `npm run test`. Use these files to exercise HTTP routes, WebSocket broadcasting, scheduler client behaviour, and the bootstrap script end-to-end.
