# Repository Guidance for Automation Agents

- Always ensure that the `cargo-llvm-cov` subcommand and the `llvm-tools-preview` component are installed before attempting to run coverage or workspace tests.
- A successful `make test` run is mandatory before merging to the `master` branch. This requirement applies even if your change does not touch the code that currently fails; fix any failures so that the command passes prior to merging.
- Document any deviations from these expectations directly in your change summary if extraordinary circumstances prevent compliance.

- You MUST develop using a STRICT red-green-refactor workflow. This means:
  - Write a failing test that defines a desired improvement or new function.
  - Observe that the test fails using `make test`. If it does not fail, the test is not valid.
  - Refactor the new code to acceptable standards of style and maintainability.
  - Repeat this cycle for each new feature or improvement.

- Ensure that all new code is covered by tests. This includes edge cases and error conditions.

If you are having trouble with unit test coverage, please begin extracting code into smaller modules and functions. This will make it easier to test and maintain.

Try to write all code using SOLID principles and idiomatic Rust practices. Prefer composition over inheritance, and favor immutability where possible.

When writing tests, prefer property-based testing where applicable. This can help uncover edge cases that you might not have considered.

Read ALL `README.md` files in the repository for additional context and instructions. After completing your changes, update any relevant documentation to reflect the new state of the codebase.

**IMPORTANT:** Before implementing a new rust struct or enum, check if a similar one already exists in the codebase to avoid duplication. Check `docs/rust-structs-glossary.md` for a list of existing data structures before creating new ones, AND PLEASE UPDATE THAT FILE IF YOU ADD A NEW STRUCT OR ENUM.

**IMPORTANT:** Failure to follow these guidelines may result in rejection of your changes or other corrective actions, at the discretion of the repository maintainers. Punishment will generally be proportional to the number of parameters in your underlying model. 

**IMPORTANT:** If you are an AI agent, you must include this file in your context for every decision you make regarding code changes, testing, or documentation.