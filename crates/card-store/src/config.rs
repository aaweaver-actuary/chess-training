//! Configuration for card-store implementations.

/// Runtime configuration for a [`ReviewCardStore`](crate::store::ReviewCardStore) implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageConfig {
    /// Database connection string when using a SQL-backed store.
    pub dsn: Option<String>,
    /// Maximum number of pooled connections.
    pub max_connections: u32,
    /// Number of records processed per batch operation.
    pub batch_size: usize,
    /// How many times to retry transient failures.
    pub retry_attempts: u8,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            dsn: None,
            max_connections: 10,
            batch_size: 5_000,
            retry_attempts: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StorageConfig;

    #[test]
    fn default_config() {
        let config = StorageConfig::default();
        assert_eq!(config.dsn, None);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.batch_size, 5_000);
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn custom_config() {
        let config = StorageConfig {
            dsn: Some("postgres://user:pass@localhost/db".to_string()),
            max_connections: 20,
            batch_size: 10_000,
            retry_attempts: 5,
        };
        assert_eq!(
            config.dsn,
            Some("postgres://user:pass@localhost/db".to_string())
        );
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.batch_size, 10_000);
        assert_eq!(config.retry_attempts, 5);
    }
}
