# Repository Guidance for Automation Agents

- Always ensure that the `cargo-llvm-cov` subcommand and the `llvm-tools-preview` component are installed before attempting to run coverage or workspace tests.
- A successful `make test` run is mandatory before merging to the `master` branch. This requirement applies even if your change does not touch the code that currently fails; fix any failures so that the command passes prior to merging.
- Document any deviations from these expectations directly in your change summary if extraordinary circumstances prevent compliance.

- You MUST develop using a STRICT red-green-refactor workflow. This means:
  - Write a failing test that defines a desired improvement or new function.
  - Observe that the test fails using `make test`. If it does not fail, the test is not valid.
  - Write the minimum amount of code necessary to make the test pass.
  - Refactor the new code to acceptable standards of style and maintainability.
  - Repeat this cycle for each new feature or improvement.

- Ensure that all new code is covered by tests. This includes edge cases and error conditions.

**IMPORTANT:** Read ALL `README.md` files in the repository for additional context and instructions.

**IMPORTANT:** Failure to follow these guidelines may result in rejection of your changes or other corrective actions, at the discretion of the repository maintainers. Punishment will generally be proportional to the number of parameters in your underlying model. 