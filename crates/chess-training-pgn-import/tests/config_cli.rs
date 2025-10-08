use std::io::Write;
use std::path::PathBuf;

use chess_training_pgn_import::config::{CliArgs, ConfigError, IngestConfig};
use tempfile::NamedTempFile;

#[test]
fn cli_parses_inputs_with_default_config() {
    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--input",
        "games/foo.pgn",
        "--input",
        "games/bar.pgn",
    ])
    .expect("CLI parsing should succeed");

    let (config, inputs) = cli
        .into_ingest_config()
        .expect("CLI conversion should succeed");

    assert_eq!(
        inputs,
        vec![
            PathBuf::from("games/foo.pgn"),
            PathBuf::from("games/bar.pgn")
        ]
    );
    assert_eq!(config, IngestConfig::default());
}

#[test]
fn cli_requires_at_least_one_input() {
    let cli = CliArgs::try_parse_from(["pgn-import"]) 
        .expect("parsing should succeed to allow config-file usage");

    let err = cli
        .into_ingest_config()
        .expect_err("conversion should fail when no inputs are provided");

    assert!(matches!(err, ConfigError::NoInputs));
}

#[test]
fn cli_applies_boolean_and_depth_overrides() {
    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--input",
        "games/foo.pgn",
        "--include-fen-in-trie",
        "--require-setup-for-fen",
        "--skip-malformed-fen",
        "--disable-tactic-from-fen",
        "--max-rav-depth",
        "3",
    ])
    .expect("CLI parsing should succeed with overrides");

    let (config, inputs) = cli
        .into_ingest_config()
        .expect("CLI conversion should succeed with overrides");

    assert_eq!(inputs, vec![PathBuf::from("games/foo.pgn")]);
    assert!(!config.tactic_from_fen, "tactic flag should invert default");
    assert!(
        config.include_fen_in_trie,
        "include-fen flag should enable trie population"
    );
    assert!(
        config.require_setup_for_fen,
        "require-setup flag should enable validation"
    );
    assert!(
        config.skip_malformed_fen,
        "skip-malformed flag should enable skipping"
    );
    assert_eq!(
        config.max_rav_depth, 3,
        "max rav depth should reflect CLI override"
    );
}

#[test]
fn cli_help_mentions_config_file_flag() {
    let err = CliArgs::try_parse_from(["pgn-import", "--help"])
        .expect_err("help invocation should short-circuit");

    let help = err.to_string();

    assert!(
        help.contains("--config-file <FILE>"),
        "help text should list the config file option"
    );
    assert!(
        help.contains("TOML"),
        "help text should mention TOML configuration support"
    );
}

#[test]
fn config_loader_merges_file_and_cli_sources() {
    let mut file = NamedTempFile::new().expect("temp config should be created");
    writeln!(
        file,
        r#"
inputs = ["config/alpha.pgn", "config/bravo.pgn"]
include_fen_in_trie = true
max_rav_depth = 12
tactic_from_fen = false
"#
    )
    .expect("temp config should be writeable");
    let path = file.into_temp_path();

    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--config-file",
        path.to_str().expect("path should be valid UTF-8"),
        "--input",
        "cli/override.pgn",
        "--skip-malformed-fen",
        "--max-rav-depth",
        "3",
    ])
    .expect("CLI parsing should succeed with config file");

    let (config, inputs) = cli
        .into_ingest_config()
        .expect("loader should merge file and CLI sources");

    assert_eq!(
        inputs,
        vec![
            PathBuf::from("config/alpha.pgn"),
            PathBuf::from("config/bravo.pgn"),
            PathBuf::from("cli/override.pgn"),
        ]
    );
    assert!(
        config.include_fen_in_trie,
        "file-provided flag should propagate"
    );
    assert!(
        config.skip_malformed_fen,
        "CLI flag should override config defaults"
    );
    assert_eq!(
        config.max_rav_depth, 3,
        "CLI max depth should override file value"
    );
    assert!(
        !config.tactic_from_fen,
        "file config should disable tactic extraction when CLI does not override"
    );
}
