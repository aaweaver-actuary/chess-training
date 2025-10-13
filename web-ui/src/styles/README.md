# Styles

```mermaid
flowchart TD
    styles_dir["styles/"] --> theme["theme.css"]

    classDef leaf fill:#fff0f6,stroke:#c2185b
    class theme leaf;
```

Global style utilities, theme tokens, and CSS modules shared across the React application. Keep design-system specific guidance here for quick reference. `theme.css` centralises colour variables consumed by the command console and dashboard components.
