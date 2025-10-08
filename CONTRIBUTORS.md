# Contributing to **chess-trainer**

Thank you for contributing!  
This guide defines project conventions, naming rules, testing expectations, and style preferences across Rust, TypeScript, and Python components.

---

## Table of Contents

1. [Philosophy](#philosophy)
2. [Repository Structure](#repository-structure)
3. [Coding Standards](#coding-standards)
   - [Rust](#rust)
   - [TypeScript / Node](#typescript--node)
   - [Python](#python)
4. [Naming Conventions](#naming-conventions)
5. [Testing & Validation](#testing--validation)
6. [Git Workflow](#git-workflow)
7. [Commit Style](#commit-style)
8. [Code Review Guidelines](#code-review-guidelines)
9. [Documentation Standards](#documentation-standards)
10. [Versioning](#versioning)
11. [Local Development Commands](#local-development-commands)
12. [FAQ](#faq)

---

## Philosophy

**chess-trainer** aims for *deterministic reproducibility* and *explicit modularity*.  
Every piece of code should be:
1. **Predictable** — same inputs → same outputs.
2. **Composable** — small crates/services that work independently.
3. **Observable** — structured logs, counters, metrics.
4. **Documented** — each module explains *what* it does and *why* it exists.

Prefer *clarity over cleverness*.

---

## Repository Structure

```
chess-trainer/
├── crates/
│   ├── pgn-import/        # PGN parsing, trie building, tactic extraction
│   ├── scheduler-core/    # SRS algorithm and unlock logic
│   ├── openings-store/    # Database layer (sqlx)
│   └── shared-models/     # Reusable structs and hashing helpers
├── apps/
│   ├── session-gateway/   # Node/TS WebSocket & REST bridge
│   └── web/               # React front-end (chessboard.js)
├── workers/
│   └── analysis/          # Python engine analysis + FSRS fitting
├── infrastructure/        # docker-compose, migrations, env templates
└── tests/                 # cross-crate integration fixtures
```

---

## Coding Standards

### Rust
- **Edition:** 2021
- **Style:** `cargo fmt` + `cargo clippy -- -D warnings`
- **Testing:** `cargo test --all`
- **Crates:** prefer `anyhow` for app-level errors, `thiserror` for typed library errors.
- **Logging:** use `tracing` with `info!`, `warn!`, `error!`; never `println!`.
- **Docs:** Every public struct/function needs a `///` doc comment.
- **Modules:** keep under 500 lines; factor into submodules early.
- **Imports:** group as
  ```rust
  use std::*;
  use crate::*;
  use external_crate::*;
  ```

### TypeScript / Node
- **Runtime:** Node 20+
- **Formatter:** Prettier + ESLint (Airbnb base)
- **Style:** semicolons required; 2-space indent.
- **Modules:** ES modules (`import/export`), not CommonJS.
- **Typing:** always use explicit interfaces or `type` aliases; no `any`.
- **API Layer:** use `zod` schemas for request/response validation.
- **Tests:** Jest or Vitest with `npm test` or `pnpm test`.
- **Naming:** PascalCase for components, camelCase for functions/vars.

### Python (workers)
- **Version:** 3.11+
- **Style:** `ruff format` + `ruff check --fix`
- **Type hints:** required for all function signatures (`from __future__ import annotations`)
- **Env:** managed with `uv` or `poetry`
- **Logging:** `structlog` or `logging` JSON handler; no `print()`
- **Testing:** `pytest -q --cov`

---

## Naming Conventions

| Concept | Convention | Example |
|----------|-------------|----------|
| Crate / Package | kebab-case | `pgn-import` |
| Module / File | snake_case | `opening_trie.rs`, `scheduler.rs` |
| Struct / Enum | PascalCase | `Position`, `SchedulerState` |
| Trait | PascalCase, noun | `Storage`, `Scheduler` |
| Function / Method | snake_case | `ingest_pgn_path`, `update_intervals` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_EASE_FACTOR` |
| JSON/DB field | snake_case | `move_uci`, `child_id` |
| React Component | PascalCase | `OpeningReviewBoard` |
| Frontend route | kebab-case | `/deck-manager` |
| Config keys | snake_case | `tactic_from_fen` |

---

## Testing & Validation

### Rust
- Unit tests in same module (`#[cfg(test)] mod tests`)
- Property tests for hash determinism
- Integration tests in `tests/`
- Golden tests (compare JSONL outputs) for pgn-import

### Frontend
- Component tests: Jest/Vitest
- E2E (optional): Playwright / Cypress

### Python
- Pytest with coverage ≥ 80%
- Mock external engines; don’t require Stockfish during CI

### Metrics expectations
Each long-running service must export:
- `games_total`, `positions_inserted`, `edges_deduped`, `reviews_logged`, etc.

---

## Git Workflow

- Default branch: `main`
- Branch naming: `feature/<short-name>` or `fix/<short-name>`
- Always create a PR; no direct pushes to `main`.
- Use draft PRs for WIP.
- Rebase, don’t merge, to keep linear history.

Example:
```bash
git checkout -b feature/opening-trie-tests
git commit -m "feat(pgn-import): add property tests for opening trie"
git push origin feature/opening-trie-tests
```

---

## Commit Style

Follows **Conventional Commits**:

```
<type>(scope): <description>

[optional body]
```

### Common types
| Type | Meaning |
|------|----------|
| `feat` | New feature |
| `fix` | Bug fix |
| `chore` | Maintenance, CI/CD |
| `docs` | Documentation only |
| `refactor` | Internal code change |
| `test` | Adding or updating tests |
| `perf` | Performance improvement |
| `style` | Formatting only |

Example:
```
feat(scheduler-core): implement SM-2 scheduling logic
fix(pgn-import): handle missing SetUp tag with FEN
docs(readme): add architecture diagram
```

---

## Code Review Guidelines

- Focus on **correctness**, **clarity**, and **test coverage**, not micro-optimization.
- Request changes only for *objective* issues (safety, determinism, missing tests).
- Each PR should:
  - Pass `cargo test --all`, `pnpm test`, or `pytest`
  - Include new/updated tests for new code paths
  - Update README or inline docs if public APIs change
- Approvals required: 1 maintainer + 1 peer review.

---

## Documentation Standards

- Each crate must contain:
  - `README.md` describing purpose and API surface
  - `CHANGELOG.md` (auto-generated optional)
- Public APIs require Rustdoc examples.
- Keep diagrams in `/docs` or `README.md` as ASCII or Mermaid.
- Code comments should explain *why*, not *what*.

---

## Versioning

- Semantic Versioning (`MAJOR.MINOR.PATCH`)
- Breaking API changes → bump MAJOR.
- Internal non-breaking changes (refactors, test-only) → PATCH.
- Shared `Cargo.toml` workspace version tags major releases.

---

## Local Development Commands

| Purpose | Command |
|----------|----------|
| Run all tests | `cargo test --all && pnpm test && pytest` |
| Lint / format | `cargo fmt && cargo clippy && pnpm lint && ruff check` |
| Build project | `cargo build --workspace` |
| Run PGN import | `cargo run -p pgn-import -- --input ./tests/data/opening_and_tactic.pgn` |
| Start scheduler service | `cargo run -p scheduler-core` |
| Launch UI | `pnpm --prefix apps/web dev` |
| Run Docker infra | `docker compose up -d postgres redis` |

---

## FAQ

**Q:** *Can I use a different PGN parser?*  
A: Only if it preserves move legality and determinism; `shakmaty` is default.

**Q:** *Where should I log debug output?*  
A: Use structured `tracing` logs in Rust; `logger.info()` in Node; `structlog` in Python.

**Q:** *How do I add a new SRS algorithm?*  
A: Implement `trait Scheduler` in `scheduler-core` and register it in `SchedulerFactory`.

**Q:** *Do I need to test UI components?*  
A: Yes—each visual component must have at least a snapshot test and one behavioral test.

**Q:** *When should I split into a new crate?*  
A: When the module has ≥500 LOC, a separate responsibility, or distinct dependencies.

---

Thank you for helping keep **chess-trainer** maintainable, deterministic, and fun to work on
