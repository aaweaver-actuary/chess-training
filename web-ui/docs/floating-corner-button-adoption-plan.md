# Floating corner button adoption plan

## Objectives

- Replace bespoke floating buttons with the shared `FloatingCornerButton` so we present consistent visuals and behavior across the UI.
- Extend the component so it can anchor to any viewport corner and support both `absolute` (relative container) and `fixed` (viewport) positioning.
- Ensure the component can render anchors (`<a>`/`<Link>`) as well as native `<button>` elements without losing accessibility affordances.

## Current state snapshot

- The reusable component is defined in `FloatingCornerButton.tsx` and only forwards props to a styled `<button>` element while always positioning itself at the top-right of its nearest positioned ancestor.【F:web-ui/src/components/FloatingCornerButton.tsx†L1-L20】【F:web-ui/src/components/FloatingCornerButton.css†L1-L59】
- The command console launcher renders a bottom-right fixed button with custom styling, which is a prime candidate for adopting the shared component once placement can be configured.【F:web-ui/src/components/CommandConsole.tsx†L133-L189】【F:web-ui/src/components/CommandConsole.css†L1-L40】
- Several navigation affordances (dashboard CTA, review back-link, Lichess shortcut) reuse the `.floating-action` style token rather than the new component; two of them are React Router `<Link>` elements and one is a plain anchor rendered within the board container.【F:web-ui/src/pages/DashboardPage.tsx†L18-L40】【F:web-ui/src/pages/OpeningReviewPage.tsx†L39-L56】【F:web-ui/src/components/OpeningReviewBoard.tsx†L220-L260】【F:web-ui/src/App.css†L261-L360】

## Target integration candidates

1. **Command console launcher** — needs `fixed` positioning anchored to the bottom-right corner and support for toggling an "active" visual state.
2. **Review navigation links** — both `DashboardPage` and `OpeningReviewPage` require Link/anchor semantics with floating positioning tokens that mirror the component design.
3. **Lichess analysis shortcut** — absolutely positioned anchor inside the board wrapper; should share placement logic and radius/hover treatments with the reusable component.
4. **Future overlays** — e.g., tutorial affordances or board controls that could sit in other corners; having configurable placement avoids further one-off buttons.

## Proposed enhancements to `FloatingCornerButton`

- **Placement prop**: accept a limited set such as `"top-right" | "top-left" | "bottom-right" | "bottom-left"`. Apply modifier classes (e.g., `floating-corner-button--bottom-right`) that flip the appropriate `top/bottom` and `left/right` rules and translate offsets.
- **Positioning strategy**: add a `strategy` prop defaulting to `'absolute'` but permitting `'fixed'`. This switches between container-relative and viewport-relative positioning for use cases like the command console.
- **Polymorphic rendering**: expose an `as` prop (default `'button'`) so callers can render `<a>` or `<Link>` elements without wrapping. When rendering a button, continue to default the `type` prop; when rendering links, avoid setting `type`.
- **State modifiers**: optionally allow a `data-active` flag or dedicated prop to adjust styling when toggled (needed for the console launcher fade-out behavior).
- **Theming tokens**: refactor shared gradient, typography, and spacing values into CSS custom properties so the `.floating-action` class and the component share the same visual system during migration.

## Implementation plan

1. **Refactor component structure**
   - Introduce polymorphic typing for `FloatingCornerButton` (e.g., using a generic over `ElementType`).
   - Extend accepted props to include `placement`, `strategy`, optional `offset` overrides, and `isActive` (maps to a `data-state="active"` attribute).
   - Maintain backward compatibility by defaulting to `placement="top-right"`, `strategy="absolute"`, and `isActive=false`.

2. **Update styles**
   - Replace hard-coded `top/right` rules with logical properties controlled by modifier classes and CSS variables (e.g., `--floating-corner-offset-x`, `--floating-corner-offset-y`).
   - Add modifier selectors for each placement and for `data-state="active"` to support the launcher opacity toggle.
   - Extract the gradient/background/hover tokens shared with `.floating-action` into a new `floating-corner-base` layer or dedicated CSS file that can be imported by both the component and legacy selectors during migration.

3. **Add tests**
   - Extend the existing test suite to cover placement and polymorphic rendering (e.g., rendering as a link, ensuring classes/attributes are applied, verifying `type` is omitted for anchors).
   - Add visual regression guidance (e.g., Chromatic or screenshot tests) if available, or at minimum document manual QA steps for hover/focus states across placements.

4. **Migrate usages**
   - Replace the command console launcher with `FloatingCornerButton` configured as `{ placement: 'bottom-right', strategy: 'fixed', 'data-state': isVisible ? 'active' : undefined }` and move specialized sizing/animation into component modifiers or wrapper styles.
   - Wrap `.floating-action` usages (dashboard CTA, review back-link, Lichess shortcut) with the polymorphic component using `as={Link}` or `<FloatingCornerButton as="a">`, ensuring existing accessibility attributes are preserved. During migration, deprecate the `.floating-action` class in `App.css`.
   - Remove redundant bespoke button CSS once each consumer switches to the shared modifiers.

5. **Documentation and examples**
   - Update `web-ui/src/components/README.md` (or add a dedicated MDX/story) to explain supported props, placement examples, and accessibility considerations.
   - Note required manual QA (e.g., verify keyboard focus indicators and pointer interactions in each placement) in the docs for future contributors.

## Risks and mitigations

- **Polymorphic typing complexity**: leverage existing patterns (e.g., Radix UI `Slot` approach) to keep TypeScript typings manageable; include runtime fallback by coercing `as` to a valid element.
- **Visual regressions**: stage migration by enabling new placement modifiers while retaining legacy classes until each consumer has moved over; use Storybook or unit snapshots to guard against CSS regressions.
- **Overlap with responsive layouts**: test placements at mobile breakpoints to ensure offsets do not collide with other UI elements (command console overlay, board edges). Provide configurable offsets for fine-tuning when necessary.

## Rollout strategy

1. Ship the generalized component and updated tests behind non-breaking defaults.
2. Convert one consumer at a time (command console first to validate `fixed` placement, then navigation links) while monitoring for layout issues.
3. After all consumers migrate, remove deprecated `.floating-action` styles and any unused bespoke button rules, followed by a final regression test pass across supported browsers/devices.
