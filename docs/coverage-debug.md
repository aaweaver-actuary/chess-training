# Card Store Coverage Verification

To reproduce the GitHub Actions `Enforce coverage (card-store)` step locally, run:

```bash
mkdir -p target/llvm-cov
cargo llvm-cov \
  --package card-store \
  --release \
  --all-features \
  --fail-under-lines 100 \
  --fail-under-functions 100 \
  --fail-under-regions 100 \
  --show-missing-lines \
  --lcov --output-path target/llvm-cov/card-store.lcov
```

When run against the current state of the repository, the command should finish successfully with no uncovered lines reported. If it fails, inspect the paths listed under "Uncovered Lines" in the command output to identify the missing coverage and add targeted tests.
