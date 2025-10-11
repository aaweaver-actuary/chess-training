# Issue #172: Duplicate textarea IDs in PGN import panel

## Context
- **Issue:** [#172](https://github.com/aaweaver-actuary/chess-training/issues/172)
- **Created:** 2025-10-11T14:29:28Z
- **Assignee:** _Unassigned_
- **Status:** Open

## Current observations
- The PGN import pane renders two `<textarea>` elements that both use the `id="pgn-import-textarea"` attribute, one in paste mode and one in upload mode. This duplicates the HTML `id` and breaks the one-to-one relationship expected by assistive technologies and `label` elements.
- Both labels in each mode point at the shared `id`, so whichever textarea renders last will be associated with both labels, creating confusing focus and accessibility semantics.

Relevant source lines:
- `web-ui/src/components/PgnImportPane.tsx`, lines 334-399.

## Suggested next steps
- Give the upload-mode textarea a distinct identifier (for example, `pgn-import-review-textarea`) and update its associated `<label>`.
- Add a unit test in the PGN import pane test suite that mounts the component in both modes and asserts that the textareas expose distinct `id` attributes. This would prevent regressions.
- Manually verify the PGN import workflow in both paste and upload modes after adjusting identifiers to ensure no references or CSS selectors rely on the old value.

## Notes
- No active development is tracked on this issue (no assignee or linked pull request).
- No automated tests or builds were executed during this triage step.
