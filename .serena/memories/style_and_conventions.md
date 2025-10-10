# Style and Conventions
- Follow instructions in `AGENTS.md`: strict red-green-refactor workflow, maintain 100% coverage thresholds using cargo-llvm-cov and Vitest.
- Default to ASCII when editing/creating files; only add succinct explanatory comments when logic is non-obvious.
- Do not revert user changes; respect existing formatting. Use provided tools for editing rather than ad-hoc commands.
- Keep React/Vitest tests wrapped in act when triggering state updates; ensure coverage ignores only for unreachable branches.
- Rust code adheres to clippy warnings (except known expect_fun_call warnings) and uses deterministic hashing model for chess entities.