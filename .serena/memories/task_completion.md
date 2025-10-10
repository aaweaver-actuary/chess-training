# Task Completion Checklist
- Run `make test` before merging any change; all stages (Rust, web-ui, session-gateway) must succeed with 100% coverage.
- If shortcuts are necessary, document deviations in change summary per `AGENTS.md` policy.
- Confirm cargo-llvm-cov and llvm-tools-preview are installed when working with Rust coverage.
- Review outstanding clippy or lint warnings and address unless explicitly tolerated.
- Ensure newly added tests fail before fixes (red) and pass afterward (green) to honor red-green-refactor workflow.