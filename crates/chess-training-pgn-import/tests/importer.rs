use chess_training_pgn_import::config::IngestConfig;
use chess_training_pgn_import::importer::{ImportError, Importer};
use chess_training_pgn_import::storage::InMemoryImportStore;

fn sample_pgn() -> &'static str {
    r#"[Event "Opening"]
[Site "Local"]
[Date "2025.10.08"]
[Round "-"]
[White "White"]
[Black "Black"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *

[Event "Tactic"]
[Site "Local"]
[Date "2025.10.08"]
[Round "-"]
[White "White"]
[Black "Black"]
[SetUp "1"]
[FEN "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQ1RK1 w kq - 4 6"]

6. Nxe5 Nxe5 7. d4 Nxc4 8. dxc5 *
"#
}

#[test]
fn importer_builds_opening_trie_and_tactics() {
    let config = IngestConfig::default();
    let store = InMemoryImportStore::default();
    let mut importer = Importer::new(config, store);

    importer
        .ingest_pgn_str("owner", "main", sample_pgn())
        .expect("import should succeed");

    let (store, metrics) = importer.finalize();

    assert_eq!(metrics.games_total, 2, "two games should be processed");
    assert_eq!(metrics.opening_edges, 6, "opening game adds six moves");
    assert_eq!(metrics.tactics, 1, "one tactic should be extracted");

    let positions = store.positions();
    assert!(
        positions
            .iter()
            .any(|pos| pos.fen.starts_with("rnbqkbnr/pppppppp")),
        "start position must be recorded"
    );

    let edge_uci: Vec<_> = store
        .edges()
        .into_iter()
        .map(|edge| edge.edge.move_uci)
        .collect();
    assert!(edge_uci.contains(&"e2e4".to_string()));
    assert!(edge_uci.contains(&"e7e5".to_string()));
    assert!(edge_uci.contains(&"g1f3".to_string()));

    let tactics = store.tactics();
    assert_eq!(tactics.len(), 1, "exactly one tactic is expected");
    let tactic = &tactics[0];
    assert_eq!(
        tactic.pv_uci,
        vec![
            "f3e5".to_string(),
            "c6e5".to_string(),
            "d2d4".to_string(),
            "e5c4".to_string(),
            "d4c5".to_string()
        ]
    );
}

#[test]
fn importer_respects_require_setup_flag() {
    let config = IngestConfig {
        require_setup_for_fen: true,
        ..IngestConfig::default()
    };

    let store = InMemoryImportStore::default();
    let mut importer = Importer::new(config, store);

    let pgn = r#"[Event "Tactic"]
[FEN "8/8/8/8/8/8/8/8 w - - 0 1"]

1. Kh2 *
"#;

    let err = importer
        .ingest_pgn_str("owner", "main", pgn)
        .expect_err("missing SetUp header should error");

    let is_missing_setup = |error: &ImportError| matches!(error, ImportError::MissingSetup { .. });
    assert!(is_missing_setup(&err));
    assert!(!is_missing_setup(&ImportError::InvalidFen {
        fen: "fen".to_string(),
    }));
}

#[test]
fn importer_skips_malformed_fens_when_configured() {
    let config = IngestConfig {
        skip_malformed_fen: true,
        ..IngestConfig::default()
    };

    let mut importer = Importer::with_in_memory_store(config);

    let malformed = r#"[Event "Invalid"]
[SetUp "1"]
[FEN "not a real fen"]

1. e4 *
"#;

    importer
        .ingest_pgn_str("owner", "main", malformed)
        .expect("malformed FEN should be skipped when flag is set");

    let (store, metrics) = importer.finalize();

    assert_eq!(metrics.games_total, 1, "game counter should increment");
    assert!(
        store.positions().is_empty(),
        "malformed game should be ignored"
    );
    assert!(
        store.edges().is_empty(),
        "malformed game should not add edges"
    );
    assert!(
        store.tactics().is_empty(),
        "malformed game should not add tactics"
    );
}

#[test]
fn importer_reports_illegal_san_tokens() {
    let mut importer = Importer::with_in_memory_store(IngestConfig::default());

    let bad_san = r#"[Event "Corrupt"]

1. invalid *
"#;

    let err = importer
        .ingest_pgn_str("owner", "main", bad_san)
        .expect_err("invalid SAN token should error");

    let is_pgn_error =
        |error: &ImportError| matches!(error, ImportError::Pgn(token) if token == "invalid");
    assert!(is_pgn_error(&err));
    assert!(!is_pgn_error(&ImportError::MissingSetup {
        fen: "fen".to_string(),
    }));
}

#[test]
fn importer_reports_contextual_illegal_san() {
    let mut importer = Importer::with_in_memory_store(IngestConfig::default());

    let impossible_move = r#"[Event "Illegal"]

1. Qh4 *
"#;

    let err = importer
        .ingest_pgn_str("owner", "main", impossible_move)
        .expect_err("contextually illegal SAN should error");

    let is_illegal_san = |error: &ImportError| matches!(error, ImportError::IllegalSan { san, game: 0 } if san == "Qh4");
    assert!(is_illegal_san(&err));
    assert!(!is_illegal_san(&ImportError::InvalidFen {
        fen: "fen".to_string(),
    }));
}

#[test]
fn importer_does_not_emit_tactics_when_disabled() {
    let config = IngestConfig {
        tactic_from_fen: false,
        include_fen_in_trie: true,
        ..IngestConfig::default()
    };

    let mut importer = Importer::with_in_memory_store(config);

    let fen_game = r#"[Event "Tactic"]
[SetUp "1"]
[FEN "8/8/8/8/8/8/8/K6k w - - 0 1"]

1. Ka2 *
"#;

    importer
        .ingest_pgn_str("owner", "main", fen_game)
        .expect("FEN game should import without tactics");

    let (store, metrics) = importer.finalize();

    assert_eq!(metrics.tactics, 0, "tactic extraction should be disabled");
    assert!(store.tactics().is_empty(), "no tactic should be stored");
    assert!(
        !store.positions().is_empty(),
        "FEN game should populate trie when configured"
    );
}

#[test]
fn importer_ignores_empty_inputs() {
    let mut importer = Importer::with_in_memory_store(IngestConfig::default());

    importer
        .ingest_pgn_str("owner", "main", " \n\n")
        .expect("empty input should succeed");

    let (_store, metrics) = importer.finalize();
    assert_eq!(metrics.games_total, 0, "no games should be recorded");
    assert_eq!(metrics.opening_edges, 0);
    assert_eq!(metrics.tactics, 0);
}

#[test]
fn importer_errors_on_invalid_fen_without_skip() {
    let mut importer = Importer::with_in_memory_store(IngestConfig::default());

    let malformed = r#"[Event "Invalid"]
[SetUp "1"]
[FEN "invalid fen"]

1. e4 *
"#;

    let err = importer
        .ingest_pgn_str("owner", "main", malformed)
        .expect_err("invalid FEN should bubble up without skip flag");

    let is_invalid_fen = |error: &ImportError| matches!(error, ImportError::InvalidFen { .. });
    assert!(is_invalid_fen(&err));
    assert!(!is_invalid_fen(&ImportError::Pgn("pgn".to_string())));
}
