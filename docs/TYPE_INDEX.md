# Type Index

This file lists every `struct` and `enum` defined in the repository.

| Type | Kind | Location |
| --- | --- | --- |
| Card | struct | `crates/review-domain/src/card.rs#L5` |
| CardKind | enum | `crates/review-domain/src/card_kind.rs#L5` |
| ChessPosition | struct | `crates/card-store/src/chess_position.rs#L5` |
| CliArgs | struct | `crates/chess-training-pgn-import/src/config.rs#L93` |
| ConfigError | enum | `crates/chess-training-pgn-import/src/errors.rs#L80` |
| EdgeInput | struct | `crates/card-store/src/model.rs#L18` |
| ExistingUnlocks | struct | `crates/scheduler-core/src/queue.rs#L26` |
| FileConfig | struct | `crates/chess-training-pgn-import/src/config.rs#L64` |
| GameContext | struct | `crates/chess-training-pgn-import/src/importer.rs#L170` |
| ImportError | enum | `crates/chess-training-pgn-import/src/importer.rs#L56` |
| ImportInMemoryStore | struct | `crates/chess-training-pgn-import/src/storage.rs#L24` |
| ImportMetrics | struct | `crates/chess-training-pgn-import/src/importer.rs#L21` |
| Importer | struct | `crates/chess-training-pgn-import/src/importer.rs#L89` |
| InMemoryCardStore | struct | `crates/card-store/src/memory/mod.rs#L30` |
| InMemoryStore | struct | `crates/scheduler-core/src/store.rs#L20` |
| IngestConfig | struct | `crates/chess-training-pgn-import/src/config.rs#L40` |
| IoError | struct | `crates/chess-training-pgn-import/src/errors.rs#L8` |
| MoveContext | struct | `crates/chess-training-pgn-import/src/importer.rs#L225` |
| OpeningCard | struct | `crates/review-domain/src/opening.rs#L5` |
| OpeningEdge | struct | `crates/review-domain/src/opening.rs#L20` |
| OpeningEdgeRecord | struct | `crates/chess-training-pgn-import/src/model.rs#L31` |
| ParseError | struct | `crates/chess-training-pgn-import/src/errors.rs#L44` |
| Position | struct | `crates/chess-training-pgn-import/src/model.rs#L11` |
| PositionError | enum | `crates/card-store/src/errors.rs#L6` |
| RawGame | struct | `crates/chess-training-pgn-import/src/importer.rs#L487` |
| RelearningFixture | struct | `crates/scheduler-core/tests/scheduler_sm2.rs#L13` |
| RepertoireEdge | struct | `crates/chess-training-pgn-import/src/model.rs#L53` |
| ReviewGrade | enum | `crates/scheduler-core/src/grade.rs#L4` |
| ReviewOutcome | struct | `crates/scheduler-core/src/domain/mod.rs#L31` |
| ReviewRequest | struct | `crates/card-store/src/model.rs#L82` |
| ReviewTransition | struct | `crates/card-store/src/memory/reviews.rs#L9` |
| Scheduler | struct | `crates/scheduler-core/src/scheduler.rs#L14` |
| SchedulerConfig | struct | `crates/scheduler-core/src/config.rs#L4` |
| SchedulerError | enum | `crates/scheduler-core/src/errors.rs#L7` |
| SchedulerOpeningCard | struct | `crates/scheduler-core/src/domain/card_kind.rs#L5` |
| SchedulerTacticCard | struct | `crates/scheduler-core/src/domain/card_kind.rs#L19` |
| SchedulerUnlockDetail | struct | `crates/scheduler-core/src/domain/mod.rs#L22` |
| Sm2State | struct | `crates/scheduler-core/src/domain/sm2_state.rs#L6` |
| StorageConfig | struct | `crates/card-store/src/config.rs#L5` |
| StoreError | enum | `crates/card-store/src/store.rs#L14` |
| StoredCardState | struct | `crates/card-store/src/model.rs#L51` |
| StudyStage | enum | `crates/review-domain/src/study_stage.rs#L5` |
| Tactic | struct | `crates/chess-training-pgn-import/src/model.rs#L70` |
| TacticCard | struct | `crates/review-domain/src/tactic.rs#L5` |
| TimedStore | struct | `crates/scheduler-core/tests/opening_scheduling.rs#L15` |
| UnlockDetail | struct | `crates/review-domain/src/unlock.rs#L29` |
| UnlockRecord | struct | `crates/review-domain/src/unlock.rs#L7` |

Generated automatically to support DRY efforts.

Run `python docs/generate_type_index.py` to regenerate.
