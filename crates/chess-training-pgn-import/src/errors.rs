use std::error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Input/output errors that can occur during configuration loading.
#[derive(Debug)]
pub struct IoError {
    pub path: PathBuf,
    pub source: io::Error,
}

/// Implement Display for IoError to provide a user-friendly error message.
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

/// Parse errors that can occur during configuration loading.
#[derive(Debug)]
pub struct ParseError {
    pub path: PathBuf,
    pub source: toml::de::Error,
}

impl ParseError {
    /// Returns the line and column number where the parse error occurred, if available.
    pub fn line_col(&self) -> Option<(usize, usize)> {
        self.source.line_col()
    }

    /// Returns the underlying TOML parse error.
    pub fn source(&self) -> &toml::de::Error {
        &self.source
    }

    /// Returns the path of the configuration file that failed to parse.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    
}

/// Implement Display for ParseError to provide a user-friendly error message.
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
            Self::Io(error) => self::fmt::Display::fmt(error, f),
            Self::Parse(error) => write!(f, "failed to parse config file {}: {}", error.path.display(), error.source),
                )
            }
            Self::Parse { path, source } => {
                write!(
                    f,
                    "failed to parse config file {}: {}",
                    path.display(),
                    source
                )
            }
            Self::NoInputs => write!(f, "no PGN inputs were provided via CLI or config file"),
        }
    }
}

impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::NoInputs => None,
        }
    }
}
