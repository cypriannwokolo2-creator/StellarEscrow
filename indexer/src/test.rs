#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert!(config.database.url.contains("postgres"));
        assert_eq!(config.stellar.network, "testnet");
    }

    #[tokio::test]
    async fn test_health_check() {
        // This would require setting up a test server
        // For now, just ensure the function exists
    }
}