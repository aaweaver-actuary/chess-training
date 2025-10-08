//! Configuration for card-store implementations.

/// Runtime configuration for a [`CardStore`](crate::store::CardStore) implementation.
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
