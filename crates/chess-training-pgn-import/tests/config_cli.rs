use std::io::Write;
use std::path::PathBuf;

use chess_training_pgn_import::config::{CliArgs, ConfigError, IngestConfig};
use std::error::Error as _;
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
        .build_ingest_config()
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
        .build_ingest_config()
        .expect_err("conversion should fail when no inputs are provided");

    let is_no_inputs = |error: &ConfigError| matches!(error, ConfigError::NoInputs);
    assert!(is_no_inputs(&err));
    assert!(!is_no_inputs(&ConfigError::Io(
        chess_training_pgn_import::errors::IoError {
            path: PathBuf::from("/tmp/missing"),
            source: std::io::Error::other("other"),
        }
    )));
    assert_eq!(
        err.to_string(),
        "no PGN inputs were provided via CLI or config file",
        "display should describe missing inputs"
    );
    assert!(
        err.source().is_none(),
        "no-input error should have no source"
    );
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
        .build_ingest_config()
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
        .build_ingest_config()
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

#[test]
fn config_loader_reports_io_errors_with_context() {
    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--config-file",
        "/definitely/not/present/config.toml",
        "--input",
        "fallback.pgn",
    ])
    .expect("CLI parsing should allow nonexistent config paths");

    let err = cli
        .build_ingest_config()
        .expect_err("missing file should surface as an IO error");

    if let ConfigError::Io(io_error) = &err {
        assert!(
            io_error.path.ends_with("config.toml"),
            "error should preserve the requested config path"
        );
        assert_eq!(
            io_error.source.kind(),
            std::io::ErrorKind::NotFound,
            "inner IO error should be a not-found"
        );
    } else {
        panic!("expected IO error, got {err:?}");
    }

    let display = err.to_string();
    assert!(
        display.contains("failed to read config file"),
        "Display implementation should describe IO failures"
    );
    assert!(
        err.source().is_some(),
        "IO errors should retain their source for debugging"
    );
}

#[test]
fn config_loader_reports_parse_errors_with_context() {
    let mut file = NamedTempFile::new().expect("temp config should be created");
    writeln!(file, "invalid = {{ this is = not }}")
        .expect("temp config should accept invalid TOML for testing");
    let path = file.into_temp_path();
    let path_string = path.to_string_lossy().to_string();

    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--config-file",
        path.to_str().expect("path should be valid UTF-8"),
        "--input",
        "fallback.pgn",
    ])
    .expect("CLI parsing should allow invalid config contents");

    let err = cli
        .build_ingest_config()
        .expect_err("invalid TOML should surface as a parse error");

    if let ConfigError::Parse(parse_error) = &err {
        assert_eq!(
            &parse_error.path,
            &std::path::PathBuf::from(&path_string),
            "parse error should report the temporary file path"
        );
        assert!(
            parse_error.source.to_string().contains("expected"),
            "toml parse error should include diagnostic context"
        );
    } else {
        panic!("expected parse error, got {err:?}");
    }

    let display = err.to_string();
    assert!(
        display.contains("failed to parse config file"),
        "Display implementation should describe parse failures"
    );
    assert!(
        err.source().is_some(),
        "parse errors should retain their TOML error source"
    );
}

#[test]
fn config_file_flags_apply_when_cli_does_not_override() {
    let mut file = NamedTempFile::new().expect("temp config should be created");
    writeln!(
        file,
        r#"
inputs = ["config-only.pgn"]
require_setup_for_fen = true
skip_malformed_fen = true
max_rav_depth = 5
"#
    )
    .expect("temp config should be writeable");
    let path = file.into_temp_path();

    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--config-file",
        path.to_str().expect("path should be valid UTF-8"),
    ])
    .expect("CLI parsing should succeed when relying on config inputs");

    let (config, inputs) = cli
        .build_ingest_config()
        .expect("config file should supply inputs and flags");

    assert_eq!(inputs, vec![PathBuf::from("config-only.pgn")]);
    assert!(
        config.require_setup_for_fen,
        "config file should enable the setup requirement"
    );
    assert!(
        config.skip_malformed_fen,
        "config file should enable skipping malformed FENs"
    );
    assert_eq!(
        config.max_rav_depth, 5,
        "config file depth should be preserved without CLI overrides"
    );
}

#[test]
fn config_loader_handles_missing_optional_fields() {
    let mut file = NamedTempFile::new().expect("temp config should be created");
    writeln!(
        file,
        "include_fen_in_trie = true\nrequire_setup_for_fen = true"
    )
    .expect("temp config should be writeable");
    let path = file.into_temp_path();

    let cli = CliArgs::try_parse_from([
        "pgn-import",
        "--config-file",
        path.to_str().expect("path should be valid UTF-8"),
        "--input",
        "cli-only.pgn",
    ])
    .expect("CLI parsing should succeed with config defaults");

    let (config, inputs) = cli
        .build_ingest_config()
        .expect("loader should allow missing optional fields");

    assert_eq!(inputs, vec![PathBuf::from("cli-only.pgn")]);
    assert!(
        config.include_fen_in_trie,
        "config flag should be respected when inputs are absent",
    );
    assert!(
        config.require_setup_for_fen,
        "config flag should be respected without CLI override",
    );
    assert_eq!(
        config.max_rav_depth,
        IngestConfig::default().max_rav_depth,
        "missing max depth should keep the default",
    );
}
