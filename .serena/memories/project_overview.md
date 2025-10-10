# Project Overview
- chess-training is a multi-language workspace delivering a spaced-repetition training system for chess with import, scheduling, session gateway, and React UI components.
- Core tech stack: Rust crates (scheduler-core, card-store, pgn-import), TypeScript/Node session gateway, React web UI (Vite), plus supporting infrastructure scripts.
- Architecture emphasizes modular services: Rust backend crates generate and schedule review material, session-gateway mediates browser sessions, and the web UI provides interactive chessboard-based training.
- Repository organized as Cargo workspace under `crates/`, Node apps under `apps/`, frontend under `web-ui`, supporting docs/config in `docs/` and `infrastructure/`.