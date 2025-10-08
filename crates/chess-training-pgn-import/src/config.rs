//! Ingestion configuration primitives.
//!
//! Assumptions for the MVP:
//! - The default maximum variation depth is set to 8 to keep parsing deterministic and
//!   inexpensive. This value will be surfaced via CLI/TOML once task 2 wiring is complete.
//! - All boolean toggles default to the most permissive, fail-fast friendly behavior to
//!   simplify early importer development. Each toggle will be backed by CLI flags in later
//!   commits.
//! - CLI parsing only exposes primitive flags and repeated `--input` arguments for now;
//!   richer configuration sources (files/env) will compose on top of these constants in a
//!   follow-up change.
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
use std::path::PathBuf;

use clap::error::Result as ClapResult;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};

/// Runtime configuration for the PGN ingest pipeline.
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

/// Command-line arguments supported by the importer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliArgs {
    /// One or more PGN files to ingest.
    pub inputs: Vec<PathBuf>,

    /// When set, also add FEN-rooted games to the opening trie.
    include_fen_in_trie: bool,

    /// When set, require `[SetUp "1"]` alongside `[FEN]` tags.
    require_setup_for_fen: bool,

    /// When set, skip malformed FEN headers instead of failing-fast.
    skip_malformed_fen: bool,

    /// Disable tactic extraction from `[FEN]` tagged games.
    disable_tactic_from_fen: bool,

    /// Limit how deep recursive annotation variations are processed.
    max_rav_depth: u32,
}

impl CliArgs {
    const ARG_INPUT: &'static str = "input";
    const ARG_INCLUDE_FEN_IN_TRIE: &'static str = "include-fen-in-trie";
    const ARG_REQUIRE_SETUP_FOR_FEN: &'static str = "require-setup-for-fen";
    const ARG_SKIP_MALFORMED_FEN: &'static str = "skip-malformed-fen";
    const ARG_DISABLE_TACTIC_FROM_FEN: &'static str = "disable-tactic-from-fen";
    const ARG_MAX_RAV_DEPTH: &'static str = "max-rav-depth";

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
                    .required(true),
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

    fn from_matches(matches: ArgMatches) -> Self {
        let inputs = matches
            .get_many::<PathBuf>(Self::ARG_INPUT)
            .expect("required input argument should be present")
            .cloned()
            .collect();

        let include_fen_in_trie = matches.get_flag(Self::ARG_INCLUDE_FEN_IN_TRIE);
        let require_setup_for_fen = matches.get_flag(Self::ARG_REQUIRE_SETUP_FOR_FEN);
        let skip_malformed_fen = matches.get_flag(Self::ARG_SKIP_MALFORMED_FEN);
        let disable_tactic_from_fen = matches.get_flag(Self::ARG_DISABLE_TACTIC_FROM_FEN);
        let max_rav_depth = matches
            .get_one::<u32>(Self::ARG_MAX_RAV_DEPTH)
            .copied()
            .unwrap_or(DEFAULT_MAX_RAV_DEPTH);

        Self {
            inputs,
            include_fen_in_trie,
            require_setup_for_fen,
            skip_malformed_fen,
            disable_tactic_from_fen,
            max_rav_depth,
        }
    }

    /// Attempts to parse CLI arguments using the custom command definition.
    pub fn try_parse_from<I, T>(iterator: I) -> ClapResult<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::command()
            .try_get_matches_from(iterator)
            .map(Self::from_matches)
    }

    /// Converts the parsed CLI arguments into the runtime configuration and remaining inputs.
    pub fn into_ingest_config(self) -> (IngestConfig, Vec<PathBuf>) {
        let config = IngestConfig {
            tactic_from_fen: !self.disable_tactic_from_fen,
            include_fen_in_trie: self.include_fen_in_trie,
            require_setup_for_fen: self.require_setup_for_fen,
            skip_malformed_fen: self.skip_malformed_fen,
            max_rav_depth: self.max_rav_depth,
        };

        (config, self.inputs)
    }
}
