# Pages

```mermaid
flowchart TD
    pages_dir["pages/"] --> dashboard["DashboardPage.tsx"]
    pages_dir --> opening_review["OpeningReviewPage.tsx"]
    pages_dir --> blank_board["BlankBoardPage.tsx"]
    pages_dir --> tests["__tests__/\n*.test.tsx"]

    classDef leaf fill:#f5f0ff,stroke:#5d3fd3
    class dashboard,opening_review,blank_board,tests leaf;
```

Route-level components that compose services, state, and shared components into complete screens. Pages should avoid duplicating business logic by delegating to modules in `services/` or `state/`.
