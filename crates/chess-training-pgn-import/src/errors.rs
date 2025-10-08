use std::error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Input/output errors that can occur during configuration loading.
#[derive(Debug)]
pub struct IoError {
    pub path: PathBuf,
    pub source: io::Error,
}

impl IoError {
    /// Returns the path that failed to load.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the underlying IO error that caused the failure.
    pub fn source(&self) -> &io::Error {
        &self.source
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to read config file {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl error::Error for IoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Parse errors that can occur during configuration loading.
#[derive(Debug)]
pub struct ParseError {
    pub path: PathBuf,
    pub source: toml::de::Error,
}

impl ParseError {
    /// Returns the path of the configuration file that failed to parse.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the underlying TOML parse error.
    pub fn toml_error(&self) -> &toml::de::Error {
        &self.source
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to parse config file {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Errors that can occur while loading configuration from external sources.
#[derive(Debug)]
pub enum ConfigError {
    /// The requested configuration file could not be read.
    Io(IoError),
    /// The configuration file contained invalid TOML.
    Parse(ParseError),
    /// Neither the CLI nor configuration file provided any PGN inputs.
    NoInputs,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => fmt::Display::fmt(error, f),
            Self::Parse(error) => fmt::Display::fmt(error, f),
            Self::NoInputs => write!(f, "no PGN inputs were provided via CLI or config file"),
        }
    }
}

impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::NoInputs => None,
        }
    }
}
