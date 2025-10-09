//! Ingestion configuration primitives.
//!
//! Assumptions for the MVP:
//! - The default maximum variation depth is set to 8 to keep parsing deterministic and
//!   inexpensive. This value will be surfaced via CLI/TOML once task 2 wiring is complete.
//! - All boolean toggles default to the most permissive, fail-fast friendly behavior to
//!   simplify early importer development. Each toggle will be backed by CLI flags in later
//!   commits.
//! - CLI parsing exposes primitive flags, repeated `--input` arguments, and an optional
//!   `--config-file` path. Environment variable overrides remain future work but the
//!   relevant constants make it easy to extend the configuration sources.
//!
//! These assumptions are intentionally captured as constants so they can be overridden by
//! future configuration layers without touching downstream code.

/// Default toggle for extracting tactics from games containing FEN headers.
pub const DEFAULT_TACTIC_FROM_FEN: bool = true;
/// Default toggle for whether FEN-rooted games should populate the opening trie.
pub const DEFAULT_INCLUDE_FEN_IN_TRIE: bool = false;
/// Default toggle for requiring `[SetUp "1"]` alongside `[FEN]`.
pub const DEFAULT_REQUIRE_SETUP_FOR_FEN: bool = false;
/// Default toggle to skip (instead of error on) malformed FEN headers.
pub const DEFAULT_SKIP_MALFORMED_FEN: bool = false;
/// Default maximum depth for parsing recursive annotation variations.
pub const DEFAULT_MAX_RAV_DEPTH: u32 = 8;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use clap::error::Result as ClapResult;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use serde::Deserialize;

pub use crate::errors::ConfigError;
use crate::errors::{IoError, ParseError};

/// Runtime configuration for the PGN ingest pipeline.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngestConfig {
    pub tactic_from_fen: bool,
    pub include_fen_in_trie: bool,
    pub require_setup_for_fen: bool,
    pub skip_malformed_fen: bool,
    pub max_rav_depth: u32,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            tactic_from_fen: DEFAULT_TACTIC_FROM_FEN,
            include_fen_in_trie: DEFAULT_INCLUDE_FEN_IN_TRIE,
            require_setup_for_fen: DEFAULT_REQUIRE_SETUP_FOR_FEN,
            skip_malformed_fen: DEFAULT_SKIP_MALFORMED_FEN,
            max_rav_depth: DEFAULT_MAX_RAV_DEPTH,
        }
    }
}

type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileConfig {
    inputs: Option<Vec<PathBuf>>,
    tactic_from_fen: Option<bool>,
    include_fen_in_trie: Option<bool>,
    require_setup_for_fen: Option<bool>,
    skip_malformed_fen: Option<bool>,
    max_rav_depth: Option<u32>,
}

impl FileConfig {
    fn from_path(path: &Path) -> ConfigResult<Self> {
        let contents = fs::read_to_string(path).map_err(|source| {
            ConfigError::Io(IoError {
                path: path.to_path_buf(),
                source,
            })
        })?;

        toml::from_str(&contents).map_err(|source| {
            ConfigError::Parse(ParseError {
                path: path.to_path_buf(),
                source,
            })
        })
    }
}

/// Command-line arguments supported by the importer.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliArgs {
    /// One or more PGN files to ingest.
    pub inputs: Vec<PathBuf>,

    /// Optional TOML configuration file that seeds defaults.
    config_file: Option<PathBuf>,

    /// When set, also add FEN-rooted games to the opening trie.
    include_fen_in_trie: bool,

    /// When set, require `[SetUp "1"]` alongside `[FEN]` tags.
    require_setup_for_fen: bool,

    /// When set, skip malformed FEN headers instead of failing-fast.
    skip_malformed_fen: bool,

    /// Disable tactic extraction from `[FEN]` tagged games.
    disable_tactic_from_fen: bool,

    /// Limit how deep recursive annotation variations are processed.
    max_rav_depth: Option<u32>,
}

impl CliArgs {
    const ARG_INPUT: &'static str = "input";
    const ARG_INCLUDE_FEN_IN_TRIE: &'static str = "include-fen-in-trie";
    const ARG_REQUIRE_SETUP_FOR_FEN: &'static str = "require-setup-for-fen";
    const ARG_SKIP_MALFORMED_FEN: &'static str = "skip-malformed-fen";
    const ARG_DISABLE_TACTIC_FROM_FEN: &'static str = "disable-tactic-from-fen";
    const ARG_MAX_RAV_DEPTH: &'static str = "max-rav-depth";
    const ARG_CONFIG_FILE: &'static str = "config-file";

    /// Builds the clap command definition for parsing CLI arguments.
    fn command() -> Command {
        Command::new("pgn-import")
            .about("Import PGN files into structured data.")
            .arg(
                Arg::new(Self::ARG_INPUT)
                    .long("input")
                    .value_name("FILE")
                    .action(ArgAction::Append)
                    .value_parser(value_parser!(PathBuf))
                    .help("Add a PGN file to ingest (repeatable)."),
            )
            .arg(
                Arg::new(Self::ARG_CONFIG_FILE)
                    .long("config-file")
                    .value_name("FILE")
                    .value_parser(value_parser!(PathBuf))
                    .help("Load defaults, including inputs, from a TOML configuration file."),
            )
            .arg(
                Arg::new(Self::ARG_INCLUDE_FEN_IN_TRIE)
                    .long("include-fen-in-trie")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(Self::ARG_REQUIRE_SETUP_FOR_FEN)
                    .long("require-setup-for-fen")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(Self::ARG_SKIP_MALFORMED_FEN)
                    .long("skip-malformed-fen")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(Self::ARG_DISABLE_TACTIC_FROM_FEN)
                    .long("disable-tactic-from-fen")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(Self::ARG_MAX_RAV_DEPTH)
                    .long("max-rav-depth")
                    .value_name("DEPTH")
                    .value_parser(value_parser!(u32)),
            )
    }

    fn from_matches(matches: &ArgMatches) -> Self {
        let inputs = matches
            .get_many::<PathBuf>(Self::ARG_INPUT)
            .map(|values| values.cloned().collect())
            .unwrap_or_default();

        let config_file = matches.get_one::<PathBuf>(Self::ARG_CONFIG_FILE).cloned();

        let include_fen_in_trie = matches.get_flag(Self::ARG_INCLUDE_FEN_IN_TRIE);
        let require_setup_for_fen = matches.get_flag(Self::ARG_REQUIRE_SETUP_FOR_FEN);
        let skip_malformed_fen = matches.get_flag(Self::ARG_SKIP_MALFORMED_FEN);
        let disable_tactic_from_fen = matches.get_flag(Self::ARG_DISABLE_TACTIC_FROM_FEN);
        let max_rav_depth = matches.get_one::<u32>(Self::ARG_MAX_RAV_DEPTH).copied();

        Self {
            inputs,
            config_file,
            include_fen_in_trie,
            require_setup_for_fen,
            skip_malformed_fen,
            disable_tactic_from_fen,
            max_rav_depth,
        }
    }

    /// Attempts to parse CLI arguments using the custom command definition.
    ///
    /// # Errors
    ///
    /// Returns an error if clap fails to parse the provided iterator of arguments.
    pub fn try_parse_from<I, T>(iterator: I) -> ClapResult<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::command()
            .try_get_matches_from(iterator)
            .map(|matches| Self::from_matches(&matches))
    }

    /// Converts the parsed CLI arguments into the runtime configuration and remaining inputs.
    ///
    /// # Errors
    ///
    /// Returns an error if a configuration file is requested but cannot be read or parsed,
    /// or if no PGN inputs are supplied after merging CLI and file sources.
    pub fn into_ingest_config(self) -> ConfigResult<(IngestConfig, Vec<PathBuf>)> {
        let CliArgs {
            inputs,
            config_file,
            include_fen_in_trie,
            require_setup_for_fen,
            skip_malformed_fen,
            disable_tactic_from_fen,
            max_rav_depth,
        } = self;

        let mut config = IngestConfig::default();
        let mut merged_inputs = Vec::new();

        if let Some(path) = config_file {
            let file_config = FileConfig::from_path(&path)?;
            if let Some(file_inputs) = file_config.inputs {
                merged_inputs.extend(file_inputs);
            }
            if let Some(value) = file_config.tactic_from_fen {
                config.tactic_from_fen = value;
            }
            if let Some(value) = file_config.include_fen_in_trie {
                config.include_fen_in_trie = value;
            }
            if let Some(value) = file_config.require_setup_for_fen {
                config.require_setup_for_fen = value;
            }
            if let Some(value) = file_config.skip_malformed_fen {
                config.skip_malformed_fen = value;
            }
            if let Some(value) = file_config.max_rav_depth {
                config.max_rav_depth = value;
            }
        }

        merged_inputs.extend(inputs);

        if include_fen_in_trie {
            config.include_fen_in_trie = true;
        }
        if require_setup_for_fen {
            config.require_setup_for_fen = true;
        }
        if skip_malformed_fen {
            config.skip_malformed_fen = true;
        }
        if disable_tactic_from_fen {
            config.tactic_from_fen = false;
        }
        if let Some(depth) = max_rav_depth {
            config.max_rav_depth = depth;
        }

        if merged_inputs.is_empty() {
            return Err(ConfigError::NoInputs);
        }

        Ok((config, merged_inputs))
    }
}
