use shakmaty::fen::Fen;
use shakmaty::san::San;
use shakmaty::{CastlingMode, Chess, Color, EnPassantMode, Move, Position};

use crate::config::IngestConfig;
use crate::model::{Edge, Position as ModelPosition, RepertoireEdge, Tactic};
use crate::storage::{InMemoryStore, Storage};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Tracks various metrics during the import process.
///
/// - `games_total`: The total number of games processed.
/// - `opening_positions`: The number of unique opening positions inserted.
/// - `opening_edges`: The number of opening edges (moves) inserted.
/// - `repertoire_edges`: The number of repertoire edges (moves) inserted.
/// - `tactics`: The number of tactics inserted.
///
/// These metrics are incremented when the corresponding items are successfully inserted
/// during the import process, as tracked by the `note_*` methods.
#[derive(Default)]
pub struct ImportMetrics {
    pub games_total: usize,
    pub opening_positions: usize,
    pub opening_edges: usize,
    pub repertoire_edges: usize,
    pub tactics: usize,
}

impl ImportMetrics {
    fn note_position(&mut self, inserted: bool) {
        if inserted {
            self.opening_positions += 1;
        }
    }

    fn note_edge(&mut self, inserted: bool) {
        if inserted {
            self.opening_edges += 1;
        }
    }

    fn note_repertoire(&mut self, inserted: bool) {
        if inserted {
            self.repertoire_edges += 1;
        }
    }

    fn note_tactic(&mut self, inserted: bool) {
        if inserted {
            self.tactics += 1;
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ImportError {
    #[error("failed to parse PGN: {0}")]
    Pgn(String),
    #[error("invalid FEN {fen}")]
    InvalidFen { fen: String },
    #[error("missing SetUp header for FEN-tagged game {fen}")]
    MissingSetup { fen: String },
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
/// use chess_training_pgn_import::{Importer, InMemoryStore, IngestConfig};
/// let config = IngestConfig::default();
/// let store = InMemoryStore::new();
/// let mut importer = Importer::new(config, store);
/// let pgn_str = r#"[Event "Example"]
/// 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O
/// 9. h3 Nb8 10. d4 Nbd7 *"#;
/// importer.ingest_pgn_str("owner", "repertoire", pgn_str).expect("ingest should succeed");
/// let (store, metrics) = importer.finalize();
/// ```
pub struct Importer<S: Storage> {
    config: IngestConfig,
    store: S,
    metrics: ImportMetrics,
}

impl<S: Storage> Importer<S> {
    pub fn new(config: IngestConfig, store: S) -> Self {
        Self {
            config,
            store,
            metrics: ImportMetrics::default(),
        }
    }

    pub fn ingest_pgn_str(
        &mut self,
        owner: &str,
        repertoire: &str,
        pgn: &str,
    ) -> Result<(), ImportError> {
        for (game_index, game) in parse_games(pgn).enumerate() {
            self.metrics.games_total += 1;
            process_game(
                &self.config,
                &mut self.store,
                &mut self.metrics,
                owner,
                repertoire,
                game,
                game_index,
            )?;
        }
        Ok(())
    }

    pub fn finalize(self) -> (S, ImportMetrics) {
        (self.store, self.metrics)
    }
}

impl Importer<InMemoryStore> {
    pub fn new_in_memory(config: IngestConfig) -> Self {
        Self::new(config, InMemoryStore::default())
    }
}

fn process_game<S: Storage>(
    config: &IngestConfig,
    store: &mut S,
    metrics: &mut ImportMetrics,
    owner: &str,
    repertoire: &str,
    game: RawGame,
    index: usize,
) -> Result<(), ImportError> {
    let fen_tag = game.tag("FEN");
    if let Some(fen) =
        fen_tag.filter(|_| config.require_setup_for_fen && game.tag("SetUp") != Some("1"))
    {
        return Err(ImportError::MissingSetup {
            fen: fen.to_string(),
        });
    }

    let source_hint = game.tag("Event").map(str::to_string);

    let mut board = match fen_tag {
        Some(fen) => match load_fen(fen) {
            Ok(board) => board,
            Err(_err) if config.skip_malformed_fen => return Ok(()),
            Err(err) => return Err(err),
        },
        None => Chess::default(),
    };

    let include_in_trie = fen_tag.is_none() || config.include_fen_in_trie;
    let mut ply = board_to_ply(&board);
    if include_in_trie {
        metrics.note_position(store.upsert_position(position_from_board(&board, ply)));
    }

    let mut pv_moves = Vec::new();

    for san_text in &game.moves {
        let san = parse_san(san_text)?;
        let mv = san.to_move(&board).map_err(|_| ImportError::IllegalSan {
            san: san_text.clone(),
            game: index,
        })?;
        let uci = move_to_uci(&board, mv);
        let mut next_board = board.clone();
        next_board.play_unchecked(mv);

        if include_in_trie {
            let parent = position_from_board(&board, ply);
            let child_ply = board_to_ply(&next_board);
            let child = position_from_board(&next_board, child_ply);
            metrics.note_position(store.upsert_position(child.clone()));
            let edge = Edge::new(
                parent.id,
                &uci,
                &san.to_string(),
                child.id,
                source_hint.clone(),
            );
            metrics.note_edge(store.upsert_edge(edge.clone()));
            metrics.note_repertoire(
                store.upsert_repertoire_edge(RepertoireEdge::new(owner, repertoire, edge.id)),
            );
            ply = child_ply;
        } else {
            ply = board_to_ply(&next_board);
        }

        board = next_board;

        if fen_tag.is_some() && config.tactic_from_fen {
            pv_moves.push(uci);
        }
    }

    if fen_tag.is_some() && config.tactic_from_fen {
        let tactic = Tactic::new(fen_tag.unwrap(), pv_moves, Vec::new(), source_hint);
        metrics.note_tactic(store.upsert_tactic(tactic));
    }

    Ok(())
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
    let trimmed = match line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        Some(inner) => inner,
        None => return None,
    };
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
    base * 2 + if board.turn() == Color::Black { 1 } else { 0 }
}

fn position_from_board(board: &Chess, ply: u32) -> ModelPosition {
    let fen = Fen::from_position(board, EnPassantMode::Legal).to_string();
    let side = board.turn().fold_wb('w', 'b');
    ModelPosition::new(&fen, side, ply)
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
        metrics.note_position(false);
        metrics.note_edge(false);
        metrics.note_repertoire(false);
        metrics.note_tactic(false);

        assert_eq!(metrics.opening_positions, 0);
        assert_eq!(metrics.opening_edges, 0);
        assert_eq!(metrics.repertoire_edges, 0);
        assert_eq!(metrics.tactics, 0);

        metrics.note_position(true);
        metrics.note_edge(true);
        metrics.note_repertoire(true);
        metrics.note_tactic(true);

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
}
