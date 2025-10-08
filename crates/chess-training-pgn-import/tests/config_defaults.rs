use chess_training_pgn_import::config::IngestConfig;

#[test]
fn ingest_config_defaults_match_plan() {
    let cfg = IngestConfig::default();

    assert!(
        cfg.tactic_from_fen,
        "tactic-from-fen should default to enabled"
    );
    assert!(
        !cfg.include_fen_in_trie,
        "include-fen-in-trie should default to disabled"
    );
    assert!(
        !cfg.require_setup_for_fen,
        "require-setup-for-fen should default to disabled"
    );
    assert!(
        !cfg.skip_malformed_fen,
        "skip-malformed-fen should default to fail-fast"
    );
    assert_eq!(cfg.max_rav_depth, 8, "max RAV depth should default to 8");
}
