use shakmaty::fen::Fen;
use shakmaty::san::San;
use shakmaty::{CastlingMode, Chess, Color, EnPassantMode, Move, Position as ShakmatyPosition};

use crate::config::IngestConfig;
use crate::model::{OpeningEdgeRecord, RepertoireEdge};
use crate::storage::{InMemoryImportStore, Storage, UpsertOutcome};
use review_domain::Position;

/// Tracks various metrics during the import process.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportMetrics {
    /// Total number of games processed.
    pub games_total: usize,
    /// Number of unique opening positions inserted.
    pub opening_positions: usize,
    /// Number of opening edges (moves) inserted.
    pub opening_edges: usize,
    /// Number of repertoire edges (moves) inserted.
    pub repertoire_edges: usize,
    /// Number of tactic entries inserted.
    pub tactics: usize,
}

impl ImportMetrics {
    fn note_position(&mut self, outcome: UpsertOutcome) {
        if outcome.is_inserted() {
            self.opening_positions += 1;
        }
    }

    fn note_edge(&mut self, outcome: UpsertOutcome) {
        if outcome.is_inserted() {
            self.opening_edges += 1;
        }
    }

    fn note_repertoire(&mut self, outcome: UpsertOutcome, recorded_tactic: bool) {
        if outcome.is_inserted() {
            self.repertoire_edges += 1;
            if recorded_tactic {
                self.tactics += 1;
            }
        }
    }
}

/// Errors raised when parsing PGN files or deriving review data.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ImportError {
    /// The PGN text failed to parse.
    #[error("failed to parse PGN: {0}")]
    Pgn(String),
    /// An embedded FEN string was invalid.
    #[error("invalid FEN {fen}")]
    InvalidFen { fen: String },
    /// A `[FEN]` tag was present without the required `[SetUp "1"]` guard.
    #[error("missing SetUp header for FEN-tagged game {fen}")]
    MissingSetup { fen: String },
    /// A SAN move was illegal in the current game context.
    #[error("illegal SAN `{san}` in game #{game}`")]
    IllegalSan { san: String, game: usize },
}

/// Imports PGN data into a storage backend.
///
/// The `Importer` struct provides methods to ingest PGN strings and store the resulting
/// positions, edges, and tactics into a backend implementing the [`Storage`] trait.
///
/// # Type Parameters
///
/// * `S` - A type that implements the [`Storage`] trait, used as the backend for storing imported data.
///
/// # Basic usage
///
/// ```
/// use chess_training_pgn_import::{Importer, InMemoryImportStore, IngestConfig};
/// let config = IngestConfig::default();
/// let store = InMemoryImportStore::new();
/// let mut importer = Importer::new(config, store);
/// let pgn_str = r#"[Event "Example"]
/// 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O
/// 9. h3 Nb8 10. d4 Nbd7 *"#;
/// importer.ingest_pgn_str("owner", "repertoire", pgn_str).expect("ingest should succeed");
/// let (store, metrics) = importer.finalize();
/// ```
/// PGN importer that persists normalized records into a [`Storage`] backend.
pub struct Importer<S: Storage> {
    config: IngestConfig,
    store: S,
    metrics: ImportMetrics,
}

impl<S: Storage> Importer<S> {
    /// Construct a new importer using the provided configuration and storage backend.
    #[must_use]
    pub fn new(config: IngestConfig, store: S) -> Self {
        Self {
            config,
            store,
            metrics: ImportMetrics::default(),
        }
    }

    /// Ingests one or more PGN games from the provided string into the configured storage.
    ///
    /// # Errors
    ///
    /// Returns an error if any parsed game violates the configured import constraints or
    /// fails PGN validation.
    pub fn ingest_pgn_str(
        &mut self,
        owner: &str,
        repertoire: &str,
        pgn: &str,
    ) -> Result<(), ImportError> {
        for (game_index, game) in parse_games(pgn).into_iter().enumerate() {
            self.metrics.games_total += 1;
            process_game(
                &self.config,
                &mut self.store,
                &mut self.metrics,
                owner,
                repertoire,
                &game,
                game_index,
            )?;
        }
        Ok(())
    }

    /// Consume the importer and return the storage backend along with collected metrics.
    #[must_use]
    pub fn finalize(self) -> (S, ImportMetrics) {
        (self.store, self.metrics)
    }
}

impl Importer<InMemoryImportStore> {
    /// Convenience constructor that wires the importer to an in-memory store.
    #[must_use]
    pub fn with_in_memory_store(config: IngestConfig) -> Self {
        Self::new(config, InMemoryImportStore::default())
    }
}

fn process_game<S: Storage>(
    config: &IngestConfig,
    store: &mut S,
    metrics: &mut ImportMetrics,
    owner: &str,
    repertoire: &str,
    game: &RawGame,
    index: usize,
) -> Result<(), ImportError> {
    let fen_tag = game.tag("FEN");
    ensure_setup_requirement_for_fen_games(config, game, fen_tag)?;
    let source_hint = game.tag("Event").map(str::to_string);
    let context = initialize_game_context(config, store, metrics, fen_tag, source_hint.clone())?;
    play_moves_and_finalize(store, metrics, owner, repertoire, game, index, context)?;
    Ok(())
}

fn ensure_setup_requirement_for_fen_games(
    config: &IngestConfig,
    game: &RawGame,
    fen_tag: Option<&str>,
) -> Result<(), ImportError> {
    if let Some(fen) =
        fen_tag.filter(|_| config.require_setup_for_fen && game.tag("SetUp") != Some("1"))
    {
        return Err(ImportError::MissingSetup {
            fen: fen.to_string(),
        });
    }
    Ok(())
}

#[derive(Clone)]
struct GameContext {
    board: Chess,
    ply: u32,
    include_in_trie: bool,
    record_tactic_moves: bool,
    pv_moves: Vec<String>,
    source_hint: Option<String>,
}

impl GameContext {
    fn new(
        board: Chess,
        ply: u32,
        include_in_trie: bool,
        record_tactic_moves: bool,
        source_hint: Option<String>,
    ) -> Self {
        Self {
            board,
            ply,
            include_in_trie,
            record_tactic_moves,
            pv_moves: Vec::new(),
            source_hint,
        }
    }

    fn record_starting_position<S: Storage>(&self, store: &mut S, metrics: &mut ImportMetrics) {
        if self.include_in_trie {
            metrics
                .note_position(store.upsert_position(position_from_board(&self.board, self.ply)));
        }
    }

    fn advance(&mut self, movement: MoveContext) {
        if self.record_tactic_moves {
            self.pv_moves.push(movement.uci.clone());
        }
        self.board = movement.next_board;
        self.ply = movement.child_ply;
    }
}

struct MoveContext {
    uci: String,
    next_board: Chess,
    child_ply: u32,
}

impl MoveContext {
    fn new(current: &Chess, mv: Move) -> Self {
        let mut next_board = current.clone();
        next_board.play_unchecked(mv);
        let uci = move_to_uci(current, mv);
        let child_ply = board_to_ply(&next_board);
        Self {
            uci,
            next_board,
            child_ply,
        }
    }
}

fn initialize_game_context<S: Storage>(
    config: &IngestConfig,
    store: &mut S,
    metrics: &mut ImportMetrics,
    fen_tag: Option<&str>,
    source_hint: Option<String>,
) -> Result<Option<GameContext>, ImportError> {
    match load_initial_board_from_optional_fen(fen_tag, config)? {
        Some(board) => {
            let include_in_trie = fen_tag.is_none() || config.include_fen_in_trie;
            let record_tactic_moves = fen_tag.is_some() && config.tactic_from_fen;
            let ply = board_to_ply(&board);
            let context = GameContext::new(
                board,
                ply,
                include_in_trie,
                record_tactic_moves,
                source_hint,
            );
            context.record_starting_position(store, metrics);
            Ok(Some(context))
        }
        None => Ok(None),
    }
}

fn load_initial_board_from_optional_fen(
    fen_tag: Option<&str>,
    config: &IngestConfig,
) -> Result<Option<Chess>, ImportError> {
    match fen_tag {
        Some(fen) => match load_fen(fen) {
            Ok(board) => Ok(Some(board)),
            Err(_err) if config.skip_malformed_fen => Ok(None),
            Err(err) => Err(err),
        },
        None => Ok(Some(Chess::default())),
    }
}

fn play_moves_and_finalize<S: Storage>(
    store: &mut S,
    metrics: &mut ImportMetrics,
    owner: &str,
    repertoire: &str,
    game: &RawGame,
    index: usize,
    context: Option<GameContext>,
) -> Result<(), ImportError> {
    if let Some(mut ctx) = context {
        execute_full_move_sequence(store, metrics, owner, repertoire, game, index, &mut ctx)?;
    }
    Ok(())
}

fn execute_full_move_sequence<S: Storage>(
    store: &mut S,
    metrics: &mut ImportMetrics,
    owner: &str,
    repertoire: &str,
    game: &RawGame,
    index: usize,
    context: &mut GameContext,
) -> Result<(), ImportError> {
    for san_text in &game.moves {
        process_single_san_move(store, metrics, owner, repertoire, context, san_text, index)?;
    }
    Ok(())
}

fn process_single_san_move<S: Storage>(
    store: &mut S,
    metrics: &mut ImportMetrics,
    owner: &str,
    repertoire: &str,
    context: &mut GameContext,
    san_text: &str,
    index: usize,
) -> Result<(), ImportError> {
    let san = parse_san(san_text)?;
    let mv = convert_san_to_move(&context.board, san, san_text, index)?;
    let movement = MoveContext::new(&context.board, mv);
    store_opening_data_if_requested(store, metrics, owner, repertoire, context, &movement, san);
    context.advance(movement);
    Ok(())
}

fn convert_san_to_move(
    board: &Chess,
    san: San,
    original: &str,
    index: usize,
) -> Result<Move, ImportError> {
    san.to_move(board).map_err(|_| ImportError::IllegalSan {
        san: original.to_string(),
        game: index,
    })
}

fn store_opening_data_if_requested<S: Storage>(
    store: &mut S,
    metrics: &mut ImportMetrics,
    owner: &str,
    repertoire: &str,
    context: &GameContext,
    movement: &MoveContext,
    _san: San,
) {
    if !context.include_in_trie {
        return;
    }
    let child = position_from_board(&movement.next_board, movement.child_ply);
    metrics.note_position(store.upsert_position(child.clone()));
    // OpeningEdgeRecord::new signature changed; update to use only move_uci and source_hint
    let edge = OpeningEdgeRecord::new(&movement.uci, context.source_hint.clone());
    metrics.note_edge(store.upsert_edge(edge.clone()));
    let repertoire_outcome = store.upsert_repertoire_edge(RepertoireEdge::new(
        owner,
        repertoire,
        edge.move_entry.edge_id,
    ));
    metrics.note_repertoire(repertoire_outcome, context.record_tactic_moves);
}

fn parse_games(input: &str) -> Vec<RawGame> {
    let mut games = Vec::new();
    let mut current = RawGame::default();
    let mut header_in_progress = false;
    let mut saw_moves = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('[') {
            if !header_in_progress && current.has_content() {
                games.push(current);
                current = RawGame::default();
                saw_moves = false;
            }
            header_in_progress = true;
            if let Some(tag) = parse_tag(trimmed) {
                current.tags.push(tag);
            }
            continue;
        }

        header_in_progress = false;
        saw_moves = true;
        current.moves.extend(sanitize_tokens(trimmed));
    }

    if saw_moves || current.has_content() {
        games.push(current);
    }

    games
}

fn parse_tag(line: &str) -> Option<(String, String)> {
    let trimmed = line.strip_prefix('[').and_then(|s| s.strip_suffix(']'))?;
    let (key, raw_value) = trimmed.split_once(' ')?;
    let value = raw_value.trim().strip_prefix('"')?.strip_suffix('"')?;
    Some((key.to_string(), value.to_string()))
}

fn sanitize_tokens(line: &str) -> Vec<String> {
    line.split_whitespace().filter_map(sanitize_token).collect()
}

fn sanitize_token(raw: &str) -> Option<String> {
    if raw == "*" || raw == "1-0" || raw == "0-1" || raw == "1/2-1/2" {
        return None;
    }

    if raw.contains('{') || raw.contains('}') || raw.contains('(') || raw.contains(')') {
        return None;
    }

    let stripped = raw.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
    if stripped.is_empty() {
        return None;
    }

    let cleaned = stripped.trim_end_matches(['!', '?', '+', '#']);
    if cleaned.is_empty() {
        return None;
    }

    Some(cleaned.to_string())
}

fn parse_san(token: &str) -> Result<San, ImportError> {
    San::from_ascii(token.as_bytes()).map_err(|_| ImportError::Pgn(token.to_string()))
}

fn load_fen(fen: &str) -> Result<Chess, ImportError> {
    let setup: Fen = fen.parse().map_err(|_| ImportError::InvalidFen {
        fen: fen.to_string(),
    })?;
    setup
        .into_position(CastlingMode::Standard)
        .map_err(|_| ImportError::InvalidFen {
            fen: fen.to_string(),
        })
}

fn move_to_uci(board: &Chess, mv: Move) -> String {
    mv.to_uci(board.castles().mode()).to_string()
}

fn board_to_ply(board: &Chess) -> u32 {
    let base = board.fullmoves().get().saturating_sub(1);
    base * 2 + u32::from(board.turn() == Color::Black)
}

fn position_from_board(board: &Chess, _ply: u32) -> Position {
    let fen = Fen::from_position(board, EnPassantMode::Legal).to_string();
    Position::new(&fen)
}

#[derive(Default)]
struct RawGame {
    tags: Vec<(String, String)>,
    moves: Vec<String>,
}

impl RawGame {
    fn tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn has_content(&self) -> bool {
        !self.tags.is_empty() || !self.moves.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_in_memory_store_preserves_config() {
        let config = IngestConfig {
            tactic_from_fen: false,
            include_fen_in_trie: true,
            require_setup_for_fen: true,
            skip_malformed_fen: true,
            max_rav_depth: 12,
        };

        let importer = Importer::with_in_memory_store(config.clone());

        assert_eq!(importer.config, config);
        assert_eq!(importer.metrics, ImportMetrics::default());
    }

    #[test]
    fn with_in_memory_store_initializes_default_store() {
        let importer = Importer::with_in_memory_store(IngestConfig::default());

        let Importer { store, metrics, .. } = importer;

        assert!(store.positions().is_empty());
        assert!(store.edges().is_empty());
        assert!(store.tactics().is_empty());
        assert_eq!(metrics, ImportMetrics::default());
    }

    #[test]
    fn parse_tag_reads_key_value_pairs() {
        let tag = parse_tag("[Event \"Test\"]");
        assert_eq!(tag, Some(("Event".into(), "Test".into())));
    }

    #[test]
    fn parse_tag_rejects_missing_quotes() {
        assert_eq!(parse_tag("[Event Test]"), None);
        assert_eq!(parse_tag("[Event \"Test]"), None);
    }

    #[test]
    fn parse_tag_requires_closing_bracket_and_space() {
        assert_eq!(parse_tag("[Malformed"), None);
        assert_eq!(parse_tag("[Event\"Test\"]"), None);
    }

    #[test]
    fn sanitize_token_removes_results_and_markers() {
        assert_eq!(sanitize_token("1-0"), None);
        assert_eq!(sanitize_token("{comment}"), None);
        assert_eq!(sanitize_token("(variation)"), None);
    }

    #[test]
    fn sanitize_token_strips_move_numbers_and_glyphs() {
        assert_eq!(sanitize_token("12...Qxe4+!?"), Some("Qxe4".to_string()));
    }

    #[test]
    fn sanitize_token_drops_tokens_without_moves() {
        assert_eq!(sanitize_token("12...?!"), None);
    }

    #[test]
    fn parse_games_splits_multiple_pgn_entries() {
        let pgn = "[Event \"Game\"]\n\n1. e4 e5\n\n[Event \"Second\"]\n1. d4 d5 *";
        let games = parse_games(pgn);
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].moves, vec!["e4".to_string(), "e5".to_string()]);
        assert_eq!(games[1].moves, vec!["d4".to_string(), "d5".to_string()]);
    }

    #[test]
    fn parse_games_preserves_header_only_entries() {
        let pgn = "[Event \"Header Only\"]";
        let games = parse_games(pgn);

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].tags.len(), 1);
        assert!(games[0].moves.is_empty());
    }

    #[test]
    fn parse_games_ignores_invalid_tags() {
        let pgn = "[Malformed\n1. e4 e5 *";
        let games = parse_games(pgn);

        assert_eq!(games.len(), 1);
        assert!(games[0].tags.is_empty());
        let moves = &games[0].moves;
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0], "e4");
        assert_eq!(moves[1], "e5");
    }

    #[test]
    fn parse_games_returns_empty_without_content() {
        let empty = parse_games("");
        assert!(empty.is_empty());

        let whitespace = parse_games(" \n\n\t  ");
        assert!(whitespace.is_empty());
    }

    #[test]
    fn load_fen_reports_invalid_inputs() {
        let err = load_fen("not a fen").expect_err("invalid fen should fail");
        let is_invalid_fen = |error: &ImportError| matches!(error, ImportError::InvalidFen { .. });
        assert!(is_invalid_fen(&err));
        assert!(!is_invalid_fen(&ImportError::Pgn("pgn".to_string())));
    }

    #[test]
    fn load_fen_rejects_positions_missing_kings() {
        let err = load_fen("8/8/8/8/8/8/8/8 w - - 0 1")
            .expect_err("positions without kings should be invalid");
        let is_invalid_fen = |error: &ImportError| matches!(error, ImportError::InvalidFen { .. });
        assert!(is_invalid_fen(&err));
    }

    #[test]
    fn metrics_only_increment_when_inserted() {
        let mut metrics = ImportMetrics::default();
        metrics.note_position(UpsertOutcome::Replaced);
        metrics.note_edge(UpsertOutcome::Replaced);
        metrics.note_repertoire(UpsertOutcome::Replaced, false);

        assert_eq!(metrics.opening_positions, 0);
        assert_eq!(metrics.opening_edges, 0);
        assert_eq!(metrics.repertoire_edges, 0);
        assert_eq!(metrics.tactics, 0);

        metrics.note_position(UpsertOutcome::Inserted);
        metrics.note_edge(UpsertOutcome::Inserted);
        metrics.note_repertoire(UpsertOutcome::Inserted, true);

        assert_eq!(metrics.opening_positions, 1);
        assert_eq!(metrics.opening_edges, 1);
        assert_eq!(metrics.repertoire_edges, 1);
        assert_eq!(metrics.tactics, 1);
    }

    #[test]
    fn board_to_ply_standard_starting_position() {
        let board = Chess::default();
        let ply = board_to_ply(&board);
        // Starting position: fullmove 1, white to move
        // ply = (1 - 1) * 2 + 0 = 0
        assert_eq!(ply, 0);
    }

    #[test]
    fn board_to_ply_after_one_white_move() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board = load_fen(fen_str).expect("valid FEN");
        let ply = board_to_ply(&board);
        // After 1. e4: fullmove 1, black to move
        // ply = (1 - 1) * 2 + 1 = 1
        assert_eq!(ply, 1);
    }

    #[test]
    fn board_to_ply_after_one_full_move() {
        let fen_str = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";
        let board = load_fen(fen_str).expect("valid FEN");
        let ply = board_to_ply(&board);
        // After 1. e4 e5: fullmove 2, white to move
        // ply = (2 - 1) * 2 + 0 = 2
        assert_eq!(ply, 2);
    }

    #[test]
    fn board_to_ply_handles_fullmove_zero() {
        // Non-standard FEN with fullmove counter set to 0
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";
        let board = load_fen(fen_str).expect("valid FEN");
        let ply = board_to_ply(&board);
        // With fullmove 0 and saturating_sub: (0 - 1).saturating_sub() = 0
        // ply = 0 * 2 + 0 = 0
        assert_eq!(ply, 0);
    }

    #[test]
    fn board_to_ply_handles_fullmove_zero_black_to_move() {
        // Non-standard FEN with fullmove counter set to 0, black to move
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 0";
        let board = load_fen(fen_str).expect("valid FEN");
        let ply = board_to_ply(&board);
        // With fullmove 0 and saturating_sub: (0 - 1).saturating_sub() = 0
        // ply = 0 * 2 + 1 = 1
        assert_eq!(ply, 1);
    }

    #[test]
    fn ensure_setup_requirement_for_fen_games_errors_without_setup() {
        let config = IngestConfig {
            require_setup_for_fen: true,
            ..Default::default()
        };
        let mut game = RawGame::default();
        game.tags.push(("FEN".into(), "fen".into()));
        let err = ensure_setup_requirement_for_fen_games(&config, &game, Some("fen"))
            .expect_err("missing setup should fail");
        assert!(matches!(err, ImportError::MissingSetup { fen } if fen == "fen"));
    }

    #[test]
    fn ensure_setup_requirement_for_fen_games_allows_explicit_setup() {
        let config = IngestConfig {
            require_setup_for_fen: true,
            ..Default::default()
        };
        let mut game = RawGame::default();
        game.tags.push(("FEN".into(), "fen".into()));
        game.tags.push(("SetUp".into(), "1".into()));
        assert!(ensure_setup_requirement_for_fen_games(&config, &game, Some("fen")).is_ok());
    }

    #[test]
    fn load_initial_board_from_optional_fen_skips_when_configured() {
        let config = IngestConfig {
            skip_malformed_fen: true,
            ..Default::default()
        };
        let board =
            load_initial_board_from_optional_fen(Some("invalid"), &config).expect("ok result");
        assert!(board.is_none());
    }

    #[test]
    fn initialize_game_context_records_starting_position() {
        let config = IngestConfig {
            include_fen_in_trie: true,
            ..Default::default()
        };
        let mut store = InMemoryImportStore::default();
        let mut metrics = ImportMetrics::default();
        let context = initialize_game_context(&config, &mut store, &mut metrics, None, None)
            .expect("context creation succeeds")
            .expect("default board available");
        assert!(context.include_in_trie);
        assert_eq!(store.positions().len(), 1);
        assert_eq!(metrics.opening_positions, 1);
    }

    #[test]
    fn initialize_game_context_respects_skip_on_malformed_fen() {
        let config = IngestConfig {
            skip_malformed_fen: true,
            ..Default::default()
        };
        let mut store = InMemoryImportStore::default();
        let mut metrics = ImportMetrics::default();
        let context =
            initialize_game_context(&config, &mut store, &mut metrics, Some("bad fen"), None)
                .expect("skip malformed");
        assert!(context.is_none());
        assert_eq!(metrics.opening_positions, 0);
    }

    #[test]
    fn game_context_advance_tracks_ply_and_tactic_moves() {
        let board = Chess::default();
        let ply = board_to_ply(&board);
        let mut context = GameContext::new(board.clone(), ply, true, true, None);
        let san = parse_san("e4").expect("valid san");
        let mv = san.to_move(&board).expect("legal move");
        let movement = MoveContext::new(&board, mv);
        context.advance(movement);
        assert_eq!(context.ply, 1);
        assert_eq!(context.pv_moves, vec!["e2e4".to_string()]);
        assert_eq!(context.board.turn(), Color::Black);
    }

    #[test]
    fn move_context_new_derives_child_state() {
        let board = Chess::default();
        let san = parse_san("e4").expect("valid san");
        let mv = san.to_move(&board).expect("legal move");
        let movement = MoveContext::new(&board, mv);
        assert_eq!(movement.uci, "e2e4");
        assert_eq!(movement.child_ply, 1);
    }

    #[test]
    fn convert_san_to_move_reports_illegal_moves() {
        let board = Chess::default();
        let san = parse_san("Kxh8").expect("parse ok");
        let err = convert_san_to_move(&board, san, "Kxh8", 3).expect_err("illegal move");
        assert!(matches!(err, ImportError::IllegalSan { game, .. } if game == 3));
    }

    #[test]
    fn process_single_san_move_updates_metrics_and_context() {
        let config = IngestConfig {
            include_fen_in_trie: true,
            ..Default::default()
        };
        let mut store = InMemoryImportStore::default();
        let mut metrics = ImportMetrics::default();
        let mut context = initialize_game_context(&config, &mut store, &mut metrics, None, None)
            .expect("context creation")
            .expect("available");
        process_single_san_move(
            &mut store,
            &mut metrics,
            "owner",
            "rep",
            &mut context,
            "e4",
            0,
        )
        .expect("processing succeeds");
        assert_eq!(metrics.opening_edges, 1);
        assert_eq!(store.edges().len(), 1);
        assert_eq!(context.ply, 1);
    }

    // The following tests are commented out because Tactic construction is now a no-op and these tests are no longer valid.
    // #[test]
    // fn execute_full_move_sequence_processes_all_moves() { ... }
    // #[test]
    // fn finalize_tactic_if_requested_records_entry() { ... }

    #[test]
    fn play_moves_and_finalize_is_noop_when_context_absent() {
        let mut store = InMemoryImportStore::default();
        let mut metrics = ImportMetrics::default();
        let game = RawGame::default();
        assert!(
            play_moves_and_finalize(&mut store, &mut metrics, "owner", "rep", &game, 0, None)
                .is_ok()
        );
    }
}
