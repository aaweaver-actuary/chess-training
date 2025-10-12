# Issue #172: Duplicate textarea IDs in PGN import panel

## Context
- **Issue:** [#172](https://github.com/aaweaver-actuary/chess-training/issues/172)
- **Created:** 2025-10-11T14:29:28Z
- **Assignee:** _Unassigned_
- **Status:** ✅ Resolved (no code changes required)

## Current observations
- Both paste-mode and upload-mode textareas already receive distinct IDs generated with React's `useId()` hook (`pgn-import-textarea-paste-*` and `pgn-import-textarea-upload-*`).
- Each `<label>` references the matching textarea via `htmlFor`, so focus and accessibility semantics are correct.
- An automated regression test covers this behavior: `web-ui/src/components/__tests__/PgnImportPane.test.tsx` (`assigns distinct textarea ids to paste and upload modes`).

## Suggested next steps
- Close the upstream issue as "already fixed" and reference this verification note.

## Notes
- Manual inspection of `web-ui/src/components/PgnImportPane.tsx` confirms unique IDs in both modes of the component.
- Existing Jest test coverage exercises both modes and asserts distinct `id` values.
