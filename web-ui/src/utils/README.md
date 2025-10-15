# Utils

```mermaid
flowchart TD
    utils_dir["utils/"] --> command_dispatcher["commandDispatcher.ts"]
    utils_dir --> dashboard_overview["dashboardOverview.ts"]
    utils_dir --> format_unlock["formatUnlockDate.ts"]
    utils_dir --> imported_lines["importedLines.ts"]
    utils_dir --> tests["__tests__/\n*.test.ts"]

    classDef leaf fill:#f0f7ff,stroke:#3a6ea5
    class command_dispatcher,dashboard_overview,format_unlock,imported_lines,tests leaf;
```

Generic utility functions and hooks that do not fit neatly into other categories. Utilities should be pure or easily testable to encourage reuse across the application.

- `commandDispatcher.ts` exposes a registry for omnibox commands and keyboard triggers.
- `importedLines.ts` contains helpers for deduplicating and scheduling imported opening lines.
- `dashboardOverview.ts` and `formatUnlockDate.ts` provide small formatting helpers used by the dashboard components.
- Tests under `__tests__/` cover edge cases for the scheduler and formatting logic.
