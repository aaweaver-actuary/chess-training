# Components

```mermaid
flowchart TD
    components_dir["components/"] --> command_console["CommandConsole.tsx"]
    components_dir --> floating_button["FloatingCornerButton.tsx"]
    components_dir --> opening_board["OpeningReviewBoard.tsx"]
    components_dir --> pgn_import["PgnImportPane.tsx"]
    components_dir --> review_dashboard["ReviewDashboard.tsx"]
    components_dir --> session_routes["SessionRoutes.tsx"]
    components_dir --> helpers_dir["_helpers/"]
    components_dir --> tests["__tests__/\ncomponent suites"]

    classDef leaf fill:#f0f7ff,stroke:#3a6ea5
    class command_console,floating_button,opening_board,pgn_import,review_dashboard,session_routes,helpers_dir,tests leaf;
```

React components grouped by feature. Co-locate component-specific hooks, styles, and tests beside their implementation. Re-export commonly used components from an index file for ergonomic imports.

- `CommandConsole.tsx` renders the omnibox overlay and handles keyboard bindings.
- `SessionRoutes.tsx` wires React Router pages to state stores and fixtures.
- `OpeningReviewBoard.tsx` wraps `chessboard-element` with the scheduling helpers from `utils/`.
- `PgnImportPane.tsx`, `ReviewDashboard.tsx`, and `FloatingCornerButton.tsx` round out the review UI.
- `_helpers/` contains shared styling helpers consumed by multiple components.
