# Documentation

This directory contains developer-facing documentation and helper scripts that complement the inline comments and crate READMEs.

* `TYPE_INDEX.md` – generated overview of TypeScript types exported from the front-end and gateway.
* `coverage-debug.md` – troubleshooting notes for coverage tooling in CI.
* `generate_type_index.py` – utility script that keeps `TYPE_INDEX.md` in sync with the source tree.

Regenerate the type index after modifying exported TypeScript types:

```bash
python docs/generate_type_index.py
```

Feel free to add additional guides or ADRs to this directory as new systems are introduced.
