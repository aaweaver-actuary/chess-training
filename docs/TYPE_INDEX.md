# Type Index

This file lists every `struct` and `enum` defined in the repository.

| Type | Kind | Location |
| --- | --- | --- |
| Card | struct | `crates/review-domain/src/card.rs#L5` |
| CardKind | enum | `crates/review-domain/src/card_kind.rs#L5` |
| ChessPosition | struct | `crates/review-domain/src/position.rs#L21` |
| CliArgs | struct | `crates/chess-training-pgn-import/src/config.rs#L95` |
| ConfigError | enum | `crates/chess-training-pgn-import/src/errors.rs#L84` |
| EdgeInput | struct | `crates/review-domain/src/opening.rs#L7` |
| ExistingUnlocks | struct | `crates/scheduler-core/src/queue.rs#L27` |
| FileConfig | struct | `crates/chess-training-pgn-import/src/config.rs#L65` |
| GameContext | struct | `crates/chess-training-pgn-import/src/importer.rs#L179` |
| ImportError | enum | `crates/chess-training-pgn-import/src/importer.rs#L56` |
| ImportInMemoryStore | struct | `crates/chess-training-pgn-import/src/storage.rs#L50` |
| ImportMetrics | struct | `crates/chess-training-pgn-import/src/importer.rs#L21` |
| Importer | struct | `crates/chess-training-pgn-import/src/importer.rs#L89` |
| InMemoryCardStore | struct | `crates/card-store/src/memory/in_memory_card_store.rs#L25` |
| InMemoryStore | struct | `crates/scheduler-core/src/store.rs#L20` |
| IngestConfig | struct | `crates/chess-training-pgn-import/src/config.rs#L41` |
| IoError | struct | `crates/chess-training-pgn-import/src/errors.rs#L8` |
| MoveContext | struct | `crates/chess-training-pgn-import/src/importer.rs#L234` |
| OpeningCard | struct | `crates/review-domain/src/opening.rs#L38` |
| OpeningEdge | struct | `crates/review-domain/src/opening.rs#L54` |
| OpeningEdgeRecord | struct | `crates/chess-training-pgn-import/src/model.rs#L32` |
| ParseError | struct | `crates/chess-training-pgn-import/src/errors.rs#L46` |
| Position | struct | `crates/chess-training-pgn-import/src/model.rs#L11` |
| PositionError | enum | `crates/review-domain/src/position.rs#L7` |
| RawGame | struct | `crates/chess-training-pgn-import/src/importer.rs#L496` |
| RelearningFixture | struct | `crates/scheduler-core/tests/scheduler_sm2.rs#L13` |
| RepertoireEdge | struct | `crates/chess-training-pgn-import/src/model.rs#L55` |
| ReviewGrade | enum | `crates/scheduler-core/src/grade.rs#L4` |
| ReviewOutcome | struct | `crates/scheduler-core/src/domain/mod.rs#L31` |
| ReviewRequest | struct | `crates/review-domain/src/review.rs#L7` |
| ReviewTransition | struct | `crates/card-store/src/memory/reviews.rs#L35` |
| Scheduler | struct | `crates/scheduler-core/src/scheduler.rs#L14` |
| SchedulerConfig | struct | `crates/scheduler-core/src/config.rs#L4` |
| SchedulerError | enum | `crates/scheduler-core/src/errors.rs#L7` |
| SchedulerOpeningCard | struct | `crates/scheduler-core/src/domain/card_kind.rs#L5` |
| SchedulerTacticCard | struct | `crates/scheduler-core/src/domain/card_kind.rs#L20` |
| SchedulerUnlockDetail | struct | `crates/scheduler-core/src/domain/mod.rs#L22` |
| Sm2State | struct | `crates/scheduler-core/src/domain/sm2_state.rs#L6` |
| StorageConfig | struct | `crates/card-store/src/config.rs#L5` |
| StoreError | enum | `crates/card-store/src/store.rs#L14` |
| StoredCardState | struct | `crates/review-domain/src/card_state.rs#L9` |
| StudyStage | enum | `crates/review-domain/src/study_stage.rs#L5` |
| Tactic | struct | `crates/chess-training-pgn-import/src/model.rs#L73` |
| TacticCard | struct | `crates/review-domain/src/tactic.rs#L5` |
| TimedStore | struct | `crates/scheduler-core/tests/opening_scheduling.rs#L15` |
| UnlockDetail | struct | `crates/review-domain/src/unlock.rs#L30` |
| UnlockRecord | struct | `crates/review-domain/src/unlock.rs#L7` |
| UpsertOutcome | enum | `crates/chess-training-pgn-import/src/storage.rs#L26` |
| ValidGrade | struct | `crates/card-store/src/memory/reviews.rs#L8` |

Generated automatically to support DRY efforts.

Run `python docs/generate_type_index.py` to regenerate.
