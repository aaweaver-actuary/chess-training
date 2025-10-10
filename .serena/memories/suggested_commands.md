# Suggested Commands
- `make test` — runs full pipeline: cargo fmt/clippy/test + cargo-llvm-cov, web-ui format/lint/typecheck/build/test:coverage, session-gateway format/lint/typecheck/build/test:coverage.
- `cargo llvm-cov --fail-under-lines 100 --fail-under-functions 100 --fail-under-regions 100` within each crate for focused Rust coverage checks.
- `npm run test:coverage` inside `web-ui` or `apps/session-gateway` for targeted coverage reruns.
- `npm run format && npm run lint && npm run typecheck` within `web-ui` or `apps/session-gateway` when iterating on TS code.
- `cargo fmt && cargo clippy && cargo test` within a specific crate during Rust development.
- Standard macOS shell utilities available: `ls`, `cd`, `grep`, `find`, `rg`, `git status`, `git diff`.