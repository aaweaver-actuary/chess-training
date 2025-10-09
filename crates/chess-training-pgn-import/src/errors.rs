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
    pub fn io_error(&self) -> &io::Error {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Error as _;
    use std::error::Error;
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn test_io_error_creation_and_methods() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = IoError {
            path: path.clone(),
            source: io_err,
        };

        assert_eq!(error.path(), path.as_path());
        assert_eq!(error.io_error().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_io_error_display() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let error = IoError {
            path,
            source: io_err,
        };

        let display_str = format!("{}", error);
        assert!(display_str.contains("failed to read config file"));
        assert!(display_str.contains("/test/config.toml"));
        assert!(display_str.contains("Permission denied"));
    }

    #[test]
    fn test_io_error_source() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = IoError {
            path,
            source: io_err,
        };

        let source = Error::source(&error);
        assert!(source.is_some());
        let source_err = source.unwrap().downcast_ref::<io::Error>();
        assert!(source_err.is_some());
        assert_eq!(source_err.unwrap().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_parse_error_creation_and_methods() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let error = ParseError {
            path: path.clone(),
            source: toml_err,
        };

        assert_eq!(error.path(), path.as_path());
        let message = error.toml_error().to_string();
        assert!(message.contains("Invalid TOML syntax"));
    }

    #[test]
    fn test_parse_error_display() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let error = ParseError {
            path,
            source: toml_err,
        };

        let display_str = format!("{}", error);
        assert!(display_str.contains("failed to parse config file"));
        assert!(display_str.contains("/test/invalid.toml"));
        assert!(display_str.contains("Invalid TOML syntax"));
    }

    #[test]
    fn test_parse_error_source() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let error = ParseError {
            path,
            source: toml_err,
        };

        let source = Error::source(&error);
        assert!(source.is_some());
        let source_err = source.unwrap().downcast_ref::<toml::de::Error>();
        assert!(source_err.is_some());
    }

    #[test]
    fn test_config_error_io_variant() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let io_error = IoError {
            path,
            source: io_err,
        };
        let config_error = ConfigError::Io(io_error);

        assert!(matches!(config_error, ConfigError::Io(_)));
    }

    #[test]
    fn test_config_error_variants() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let parse_error = ParseError {
            path,
            source: toml_err,
        };
        let config_error = ConfigError::Parse(parse_error);

        assert!(matches!(config_error, ConfigError::Parse(_)));
        let config_error = ConfigError::NoInputs;

        assert!(matches!(config_error, ConfigError::NoInputs));
    }

    #[test]
    fn test_config_error_display_io() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let io_error = IoError {
            path,
            source: io_err,
        };
        let config_error = ConfigError::Io(io_error);

        let display_str = format!("{}", config_error);
        assert!(display_str.contains("failed to read config file"));
        assert!(display_str.contains("/test/config.toml"));
        assert!(display_str.contains("File not found"));
    }

    #[test]
    fn test_config_error_display_parse() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let parse_error = ParseError {
            path,
            source: toml_err,
        };
        let config_error = ConfigError::Parse(parse_error);

        let display_str = format!("{}", config_error);
        assert!(display_str.contains("failed to parse config file"));
        assert!(display_str.contains("/test/invalid.toml"));
        assert!(display_str.contains("Invalid TOML syntax"));
    }

    #[test]
    fn test_config_error_display_no_inputs() {
        let config_error = ConfigError::NoInputs;

        let display_str = format!("{}", config_error);
        assert_eq!(
            display_str,
            "no PGN inputs were provided via CLI or config file"
        );
    }

    #[test]
    fn test_config_error_source_io() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let io_error = IoError {
            path,
            source: io_err,
        };
        let config_error = ConfigError::Io(io_error);

        let source = config_error.source();
        assert!(source.is_some());
        let source_err = source.unwrap().downcast_ref::<IoError>();
        assert!(source_err.is_some());
    }

    #[test]
    fn test_config_error_source_parse() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let parse_error = ParseError {
            path,
            source: toml_err,
        };
        let config_error = ConfigError::Parse(parse_error);

        let source = config_error.source();
        assert!(source.is_some());
        let source_err = source.unwrap().downcast_ref::<ParseError>();
        assert!(source_err.is_some());
    }

    #[test]
    fn test_config_error_source_no_inputs() {
        let config_error = ConfigError::NoInputs;

        let source = config_error.source();
        assert!(source.is_none());
    }

    #[test]
    fn test_io_error_debug() {
        let path = PathBuf::from("/test/config.toml");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = IoError {
            path,
            source: io_err,
        };

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("IoError"));
        assert!(debug_str.contains("/test/config.toml"));
    }

    #[test]
    fn test_parse_error_debug() {
        let path = PathBuf::from("/test/invalid.toml");
        let toml_err = toml::de::Error::custom("Invalid TOML syntax");
        let error = ParseError {
            path,
            source: toml_err,
        };

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ParseError"));
        assert!(debug_str.contains("/test/invalid.toml"));
    }

    #[test]
    fn test_config_error_debug() {
        let config_error = ConfigError::NoInputs;

        let debug_str = format!("{:?}", config_error);
        assert!(debug_str.contains("NoInputs"));
    }
}
