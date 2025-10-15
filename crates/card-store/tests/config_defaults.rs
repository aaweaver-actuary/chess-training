use card_store::config::StorageConfig;

#[test]
fn storage_config_defaults_match_documented_values() {
    let config = StorageConfig::default();
    assert!(config.dsn.is_none());
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.batch_size, 5_000);
    assert_eq!(config.retry_attempts, 3);
}

#[test]
fn storage_config_can_be_customized() {
    let config = StorageConfig {
        dsn: Some("postgres://example".into()),
        max_connections: 42,
        batch_size: 1_024,
        retry_attempts: 5,
    };

    assert_eq!(config.dsn.as_deref(), Some("postgres://example"));
    assert_eq!(config.max_connections, 42);
    assert_eq!(config.batch_size, 1_024);
    assert_eq!(config.retry_attempts, 5);
}
