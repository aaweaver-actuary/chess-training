# Repository Guidance for Automation Agents

- Always ensure that the `cargo-llvm-cov` subcommand and the `llvm-tools-preview` component are installed before attempting to run coverage or workspace tests.
- A successful `make test` run is mandatory before merging to the `master` branch. This requirement applies even if your change does not touch the code that currently fails; fix any failures so that the command passes prior to merging.
- Document any deviations from these expectations directly in your change summary if extraordinary circumstances prevent compliance.
