//! # Connector Configuration Management
//!
//! This module provides configuration management for exchange connectors,
//! including credential storage, environment variable loading, and builder patterns.
//!
//! ## Overview
//!
//! - `ExchangeCredentials` - Stores API credentials for a single exchange
//! - `ConnectorConfig` - Configuration for a single connector
//! - `ConnectorConfigManager` - Manages multiple connector configurations
//!
//! ## Example
//!
//! ```ignore
//! use connectors_v5::connector_manager::config::*;
//! use connectors_v5::core::types::ExchangeId;
//!
//! // Create credentials using builder pattern
//! let creds = ExchangeCredentials::new(
//!     ExchangeId::Binance,
//!     "api_key".to_string(),
//!     "api_secret".to_string()
//! )
//! .with_testnet(true);
//!
//! // Create config with credentials
//! let config = ConnectorConfig::new(ExchangeId::Binance)
//!     .with_credentials(creds)
//!     .enabled(true);
//!
//! // Manage multiple configs
//! let mut manager = ConnectorConfigManager::new();
//! manager.add_config(config);
//!
//! // Load from environment
//! let manager = ConnectorConfigManager::load_from_env();
//! ```

use crate::core::types::ExchangeId;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE CREDENTIALS
// ═══════════════════════════════════════════════════════════════════════════════

/// Credentials for a single exchange
///
/// Stores API authentication credentials with optional passphrase support
/// for exchanges like OKX and KuCoin.
///
/// # Builder Pattern
///
/// ```ignore
/// let creds = ExchangeCredentials::new(
///     ExchangeId::OKX,
///     "my-api-key".to_string(),
///     "my-secret".to_string()
/// )
/// .with_passphrase("my-passphrase".to_string())
/// .with_testnet(true);
/// ```
#[derive(Clone, Debug)]
pub struct ExchangeCredentials {
    pub exchange_id: ExchangeId,
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,
    pub testnet: bool,
}

impl ExchangeCredentials {
    /// Create new credentials with API key and secret
    ///
    /// # Arguments
    ///
    /// * `exchange_id` - The exchange identifier
    /// * `api_key` - API key (must be non-empty)
    /// * `api_secret` - API secret (must be non-empty)
    ///
    /// # Panics
    ///
    /// Panics if `api_key` or `api_secret` is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let creds = ExchangeCredentials::new(
    ///     ExchangeId::Binance,
    ///     "key123".to_string(),
    ///     "secret456".to_string()
    /// );
    /// ```
    pub fn new(exchange_id: ExchangeId, api_key: String, api_secret: String) -> Self {
        assert!(!api_key.is_empty(), "API key cannot be empty");
        assert!(!api_secret.is_empty(), "API secret cannot be empty");

        Self {
            exchange_id,
            api_key,
            api_secret,
            passphrase: None,
            testnet: false,
        }
    }

    /// Add passphrase to credentials (builder pattern)
    ///
    /// Required for exchanges like OKX, KuCoin that use 3-factor authentication.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let creds = ExchangeCredentials::new(id, key, secret)
    ///     .with_passphrase("my-passphrase".to_string());
    /// ```
    pub fn with_passphrase(mut self, passphrase: String) -> Self {
        self.passphrase = Some(passphrase);
        self
    }

    /// Set testnet flag (builder pattern)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let creds = ExchangeCredentials::new(id, key, secret)
    ///     .with_testnet(true);
    /// ```
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }

    /// Validate that credentials are complete and non-empty
    ///
    /// Checks that api_key and api_secret are non-empty strings.
    ///
    /// # Returns
    ///
    /// `true` if credentials are valid, `false` otherwise.
    pub fn is_complete(&self) -> bool {
        !self.api_key.is_empty() && !self.api_secret.is_empty()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for a connector
///
/// Can be created with or without credentials:
/// - Without credentials: public data access only
/// - With credentials: full private trading capabilities
///
/// # Builder Pattern
///
/// ```ignore
/// // Public-only config
/// let config = ConnectorConfig::new(ExchangeId::Binance);
///
/// // Authenticated config
/// let config = ConnectorConfig::new(ExchangeId::Binance)
///     .with_credentials(creds)
///     .with_testnet(true)
///     .enabled(true);
/// ```
#[derive(Clone, Debug)]
pub struct ConnectorConfig {
    pub exchange_id: ExchangeId,
    pub credentials: Option<ExchangeCredentials>,
    pub testnet: bool,
    pub enabled: bool,
}

impl ConnectorConfig {
    /// Create new public-only connector config
    ///
    /// Config will have no credentials and be enabled by default.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ConnectorConfig::new(ExchangeId::Binance);
    /// assert!(config.is_public());
    /// ```
    pub fn new(exchange_id: ExchangeId) -> Self {
        Self {
            exchange_id,
            credentials: None,
            testnet: false,
            enabled: true,
        }
    }

    /// Add credentials to config (builder pattern)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ConnectorConfig::new(ExchangeId::Binance)
    ///     .with_credentials(creds);
    /// ```
    pub fn with_credentials(mut self, credentials: ExchangeCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set testnet flag (builder pattern)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ConnectorConfig::new(ExchangeId::Binance)
    ///     .with_testnet(true);
    /// ```
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }

    /// Set enabled flag (builder pattern)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ConnectorConfig::new(ExchangeId::Binance)
    ///     .enabled(false);
    /// ```
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if config is public-only (no credentials)
    ///
    /// # Returns
    ///
    /// `true` if no credentials are set.
    pub fn is_public(&self) -> bool {
        self.credentials.is_none()
    }

    /// Check if config has valid authentication credentials
    ///
    /// # Returns
    ///
    /// `true` if credentials are present and complete.
    pub fn is_authenticated(&self) -> bool {
        self.credentials
            .as_ref()
            .map(|c| c.is_complete())
            .unwrap_or(false)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR CONFIG MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Manager for multiple connector configurations
///
/// Provides methods for adding, retrieving, filtering, and loading
/// configurations from environment variables.
///
/// # Example
///
/// ```ignore
/// let mut manager = ConnectorConfigManager::new();
///
/// // Add individual configs
/// manager.add_config(binance_config);
/// manager.add_config(okx_config);
///
/// // Load from environment
/// let manager = ConnectorConfigManager::load_from_env();
///
/// // Query configs
/// let enabled = manager.enabled_configs();
/// let authenticated = manager.authenticated_configs();
/// ```
pub struct ConnectorConfigManager {
    configs: HashMap<ExchangeId, ConnectorConfig>,
}

impl ConnectorConfigManager {
    /// Create empty configuration manager
    ///
    /// # Example
    ///
    /// ```ignore
    /// let manager = ConnectorConfigManager::new();
    /// assert!(manager.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Add or update a connector configuration
    ///
    /// If a config for this exchange already exists, it will be replaced.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ConnectorConfig::new(ExchangeId::Binance);
    /// manager.add_config(config);
    /// ```
    pub fn add_config(&mut self, config: ConnectorConfig) {
        self.configs.insert(config.exchange_id, config);
    }

    /// Get configuration by exchange ID
    ///
    /// # Returns
    ///
    /// `Some(&ConnectorConfig)` if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(config) = manager.get_config(&ExchangeId::Binance) {
    ///     println!("Found Binance config");
    /// }
    /// ```
    pub fn get_config(&self, id: &ExchangeId) -> Option<&ConnectorConfig> {
        self.configs.get(id)
    }

    /// Remove configuration by exchange ID
    ///
    /// # Returns
    ///
    /// `Some(ConnectorConfig)` if removed, `None` if not found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let removed = manager.remove_config(&ExchangeId::Binance);
    /// ```
    pub fn remove_config(&mut self, id: &ExchangeId) -> Option<ConnectorConfig> {
        self.configs.remove(id)
    }

    /// Get all configurations
    ///
    /// # Returns
    ///
    /// Iterator over all configs.
    pub fn all_configs(&self) -> Vec<&ConnectorConfig> {
        self.configs.values().collect()
    }

    /// Get only enabled configurations
    ///
    /// # Returns
    ///
    /// Vector of configs where `enabled = true`.
    pub fn enabled_configs(&self) -> Vec<&ConnectorConfig> {
        self.configs
            .values()
            .filter(|c| c.enabled)
            .collect()
    }

    /// Get configurations with authentication credentials
    ///
    /// # Returns
    ///
    /// Vector of configs with valid credentials.
    pub fn authenticated_configs(&self) -> Vec<&ConnectorConfig> {
        self.configs
            .values()
            .filter(|c| c.is_authenticated())
            .collect()
    }

    /// Load configurations from environment variables
    ///
    /// Parses environment variables in the format:
    /// - `{EXCHANGE}_API_KEY` - API key
    /// - `{EXCHANGE}_API_SECRET` - API secret
    /// - `{EXCHANGE}_PASSPHRASE` - Passphrase (optional)
    /// - `{EXCHANGE}_TESTNET` - Testnet flag (optional, "true"/"false")
    ///
    /// Exchange names are uppercase versions of `ExchangeId::as_str()`.
    ///
    /// # Example
    ///
    /// ```bash
    /// export BINANCE_API_KEY=abc123
    /// export BINANCE_API_SECRET=secret456
    /// export BINANCE_TESTNET=true
    ///
    /// export OKX_API_KEY=okx_key
    /// export OKX_API_SECRET=okx_secret
    /// export OKX_PASSPHRASE=okx_pass
    /// ```
    ///
    /// ```ignore
    /// let manager = ConnectorConfigManager::load_from_env();
    /// ```
    pub fn load_from_env() -> Self {
        let mut manager = Self::new();

        // List of all exchanges to check
        let exchanges = [
            ExchangeId::Binance,
            ExchangeId::Bybit,
            ExchangeId::OKX,
            ExchangeId::KuCoin,
            ExchangeId::Kraken,
            ExchangeId::Coinbase,
            ExchangeId::GateIO,
            ExchangeId::Bitfinex,
            ExchangeId::Bitstamp,
            ExchangeId::Gemini,
            ExchangeId::MEXC,
            ExchangeId::HTX,
            ExchangeId::Bitget,
            ExchangeId::BingX,
            ExchangeId::CryptoCom,
            ExchangeId::Upbit,
            ExchangeId::Deribit,
            ExchangeId::HyperLiquid,
            ExchangeId::Lighter,
            ExchangeId::Dydx,
            ExchangeId::Polygon,
            ExchangeId::Finnhub,
            ExchangeId::Tiingo,
            ExchangeId::Twelvedata,
            ExchangeId::Coinglass,
            ExchangeId::CryptoCompare,
            ExchangeId::WhaleAlert,
            ExchangeId::DefiLlama,
            ExchangeId::Oanda,
            ExchangeId::AlphaVantage,
            ExchangeId::Dukascopy,
            ExchangeId::AngelOne,
            ExchangeId::Zerodha,
            ExchangeId::Fyers,
            ExchangeId::Dhan,
            ExchangeId::Upstox,
            ExchangeId::Alpaca,
            ExchangeId::JQuants,
            ExchangeId::Tinkoff,
            ExchangeId::Moex,
            ExchangeId::Krx,
            ExchangeId::Fred,
            ExchangeId::Futu,
            ExchangeId::Bitquery,
            ExchangeId::YahooFinance,
        ];

        for exchange_id in exchanges {
            // Convert exchange name to uppercase for env var prefix
            let prefix = exchange_id.as_str().to_uppercase();

            // Check for API key and secret
            let api_key_var = format!("{}_API_KEY", prefix);
            let api_secret_var = format!("{}_API_SECRET", prefix);

            if let (Ok(api_key), Ok(api_secret)) = (
                std::env::var(&api_key_var),
                std::env::var(&api_secret_var),
            ) {
                // Skip if empty
                if api_key.is_empty() || api_secret.is_empty() {
                    continue;
                }

                // Create credentials
                let mut creds = ExchangeCredentials::new(exchange_id, api_key, api_secret);

                // Check for optional passphrase
                let passphrase_var = format!("{}_PASSPHRASE", prefix);
                if let Ok(passphrase) = std::env::var(&passphrase_var) {
                    if !passphrase.is_empty() {
                        creds = creds.with_passphrase(passphrase);
                    }
                }

                // Check for testnet flag
                let testnet_var = format!("{}_TESTNET", prefix);
                if let Ok(testnet_str) = std::env::var(&testnet_var) {
                    if testnet_str.to_lowercase() == "true" {
                        creds = creds.with_testnet(true);
                    }
                }

                // Create config and add to manager
                let config = ConnectorConfig::new(exchange_id)
                    .with_credentials(creds)
                    .enabled(true);

                manager.add_config(config);
            }
        }

        manager
    }

    /// Count total configurations
    ///
    /// # Returns
    ///
    /// Number of configs in the manager.
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// Check if manager has no configurations
    ///
    /// # Returns
    ///
    /// `true` if empty, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }
}

impl Default for ConnectorConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────────────────
    // ExchangeCredentials Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_credentials_new() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key123".to_string(),
            "secret456".to_string(),
        );

        assert_eq!(creds.exchange_id, ExchangeId::Binance);
        assert_eq!(creds.api_key, "key123");
        assert_eq!(creds.api_secret, "secret456");
        assert_eq!(creds.passphrase, None);
        assert!(!creds.testnet);
    }

    #[test]
    #[should_panic(expected = "API key cannot be empty")]
    fn test_credentials_empty_key_panics() {
        ExchangeCredentials::new(
            ExchangeId::Binance,
            "".to_string(),
            "secret".to_string(),
        );
    }

    #[test]
    #[should_panic(expected = "API secret cannot be empty")]
    fn test_credentials_empty_secret_panics() {
        ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "".to_string(),
        );
    }

    #[test]
    fn test_credentials_with_passphrase() {
        let creds = ExchangeCredentials::new(
            ExchangeId::OKX,
            "key".to_string(),
            "secret".to_string(),
        )
        .with_passphrase("pass123".to_string());

        assert_eq!(creds.passphrase, Some("pass123".to_string()));
    }

    #[test]
    fn test_credentials_with_testnet() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "secret".to_string(),
        )
        .with_testnet(true);

        assert!(creds.testnet);
    }

    #[test]
    fn test_credentials_builder_chain() {
        let creds = ExchangeCredentials::new(
            ExchangeId::KuCoin,
            "key".to_string(),
            "secret".to_string(),
        )
        .with_passphrase("pass".to_string())
        .with_testnet(true);

        assert_eq!(creds.passphrase, Some("pass".to_string()));
        assert!(creds.testnet);
    }

    #[test]
    fn test_credentials_is_complete() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "secret".to_string(),
        );

        assert!(creds.is_complete());
    }

    // ───────────────────────────────────────────────────────────────────────────
    // ConnectorConfig Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_config_new() {
        let config = ConnectorConfig::new(ExchangeId::Binance);

        assert_eq!(config.exchange_id, ExchangeId::Binance);
        assert!(config.credentials.is_none());
        assert!(!config.testnet);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_with_credentials() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "secret".to_string(),
        );

        let config = ConnectorConfig::new(ExchangeId::Binance)
            .with_credentials(creds.clone());

        assert!(config.credentials.is_some());
        assert_eq!(config.credentials.unwrap().api_key, "key");
    }

    #[test]
    fn test_config_with_testnet() {
        let config = ConnectorConfig::new(ExchangeId::Binance)
            .with_testnet(true);

        assert!(config.testnet);
    }

    #[test]
    fn test_config_enabled() {
        let config = ConnectorConfig::new(ExchangeId::Binance)
            .enabled(false);

        assert!(!config.enabled);
    }

    #[test]
    fn test_config_is_public() {
        let config = ConnectorConfig::new(ExchangeId::Binance);
        assert!(config.is_public());

        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "secret".to_string(),
        );
        let config_with_creds = ConnectorConfig::new(ExchangeId::Binance)
            .with_credentials(creds);

        assert!(!config_with_creds.is_public());
    }

    #[test]
    fn test_config_is_authenticated() {
        let config = ConnectorConfig::new(ExchangeId::Binance);
        assert!(!config.is_authenticated());

        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "secret".to_string(),
        );
        let config_with_creds = ConnectorConfig::new(ExchangeId::Binance)
            .with_credentials(creds);

        assert!(config_with_creds.is_authenticated());
    }

    #[test]
    fn test_config_builder_chain() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key".to_string(),
            "secret".to_string(),
        );

        let config = ConnectorConfig::new(ExchangeId::Binance)
            .with_credentials(creds)
            .with_testnet(true)
            .enabled(false);

        assert!(config.is_authenticated());
        assert!(config.testnet);
        assert!(!config.enabled);
    }

    // ───────────────────────────────────────────────────────────────────────────
    // ConnectorConfigManager Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_manager_new() {
        let manager = ConnectorConfigManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_manager_add_config() {
        let mut manager = ConnectorConfigManager::new();
        let config = ConnectorConfig::new(ExchangeId::Binance);

        manager.add_config(config);

        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_manager_get_config() {
        let mut manager = ConnectorConfigManager::new();
        let config = ConnectorConfig::new(ExchangeId::Binance);

        manager.add_config(config);

        let retrieved = manager.get_config(&ExchangeId::Binance);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().exchange_id, ExchangeId::Binance);

        let not_found = manager.get_config(&ExchangeId::OKX);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_manager_remove_config() {
        let mut manager = ConnectorConfigManager::new();
        let config = ConnectorConfig::new(ExchangeId::Binance);

        manager.add_config(config);
        assert_eq!(manager.len(), 1);

        let removed = manager.remove_config(&ExchangeId::Binance);
        assert!(removed.is_some());
        assert_eq!(manager.len(), 0);

        let not_found = manager.remove_config(&ExchangeId::Binance);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_manager_all_configs() {
        let mut manager = ConnectorConfigManager::new();

        manager.add_config(ConnectorConfig::new(ExchangeId::Binance));
        manager.add_config(ConnectorConfig::new(ExchangeId::OKX));
        manager.add_config(ConnectorConfig::new(ExchangeId::Bybit));

        let all = manager.all_configs();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_manager_enabled_configs() {
        let mut manager = ConnectorConfigManager::new();

        manager.add_config(ConnectorConfig::new(ExchangeId::Binance).enabled(true));
        manager.add_config(ConnectorConfig::new(ExchangeId::OKX).enabled(false));
        manager.add_config(ConnectorConfig::new(ExchangeId::Bybit).enabled(true));

        let enabled = manager.enabled_configs();
        assert_eq!(enabled.len(), 2);
    }

    #[test]
    fn test_manager_authenticated_configs() {
        let mut manager = ConnectorConfigManager::new();

        let creds1 = ExchangeCredentials::new(
            ExchangeId::Binance,
            "key1".to_string(),
            "secret1".to_string(),
        );
        let creds2 = ExchangeCredentials::new(
            ExchangeId::Bybit,
            "key2".to_string(),
            "secret2".to_string(),
        );

        manager.add_config(ConnectorConfig::new(ExchangeId::Binance).with_credentials(creds1));
        manager.add_config(ConnectorConfig::new(ExchangeId::OKX)); // No creds
        manager.add_config(ConnectorConfig::new(ExchangeId::Bybit).with_credentials(creds2));

        let authenticated = manager.authenticated_configs();
        assert_eq!(authenticated.len(), 2);
    }

    #[test]
    fn test_manager_default() {
        let manager = ConnectorConfigManager::default();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_manager_replace_existing_config() {
        let mut manager = ConnectorConfigManager::new();

        let config1 = ConnectorConfig::new(ExchangeId::Binance).enabled(true);
        let config2 = ConnectorConfig::new(ExchangeId::Binance).enabled(false);

        manager.add_config(config1);
        assert_eq!(manager.len(), 1);
        assert!(manager.get_config(&ExchangeId::Binance).unwrap().enabled);

        manager.add_config(config2);
        assert_eq!(manager.len(), 1); // Still 1, replaced
        assert!(!manager.get_config(&ExchangeId::Binance).unwrap().enabled);
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Environment Variable Loading Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_load_from_env_basic() {
        // Set test environment variables
        std::env::set_var("BINANCE_API_KEY", "test_key");
        std::env::set_var("BINANCE_API_SECRET", "test_secret");

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::Binance);
        assert!(config.is_some());

        let config = config.unwrap();
        assert!(config.is_authenticated());
        assert_eq!(config.credentials.as_ref().unwrap().api_key, "test_key");
        assert_eq!(config.credentials.as_ref().unwrap().api_secret, "test_secret");

        // Cleanup
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_API_SECRET");
    }

    #[test]
    fn test_load_from_env_with_passphrase() {
        std::env::set_var("OKX_API_KEY", "okx_key");
        std::env::set_var("OKX_API_SECRET", "okx_secret");
        std::env::set_var("OKX_PASSPHRASE", "okx_pass");

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::OKX);
        assert!(config.is_some());

        let creds = config.unwrap().credentials.as_ref().unwrap();
        assert_eq!(creds.passphrase, Some("okx_pass".to_string()));

        // Cleanup
        std::env::remove_var("OKX_API_KEY");
        std::env::remove_var("OKX_API_SECRET");
        std::env::remove_var("OKX_PASSPHRASE");
    }

    #[test]
    fn test_load_from_env_with_testnet() {
        std::env::set_var("BYBIT_API_KEY", "bybit_key");
        std::env::set_var("BYBIT_API_SECRET", "bybit_secret");
        std::env::set_var("BYBIT_TESTNET", "true");

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::Bybit);
        assert!(config.is_some());

        let creds = config.unwrap().credentials.as_ref().unwrap();
        assert!(creds.testnet);

        // Cleanup
        std::env::remove_var("BYBIT_API_KEY");
        std::env::remove_var("BYBIT_API_SECRET");
        std::env::remove_var("BYBIT_TESTNET");
    }

    #[test]
    fn test_load_from_env_testnet_false() {
        std::env::set_var("KRAKEN_API_KEY", "kraken_key");
        std::env::set_var("KRAKEN_API_SECRET", "kraken_secret");
        std::env::set_var("KRAKEN_TESTNET", "false");

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::Kraken);
        assert!(config.is_some());

        let creds = config.unwrap().credentials.as_ref().unwrap();
        assert!(!creds.testnet);

        // Cleanup
        std::env::remove_var("KRAKEN_API_KEY");
        std::env::remove_var("KRAKEN_API_SECRET");
        std::env::remove_var("KRAKEN_TESTNET");
    }

    #[test]
    fn test_load_from_env_multiple_exchanges() {
        std::env::set_var("BINANCE_API_KEY", "binance_key");
        std::env::set_var("BINANCE_API_SECRET", "binance_secret");
        std::env::set_var("OKX_API_KEY", "okx_key");
        std::env::set_var("OKX_API_SECRET", "okx_secret");
        std::env::set_var("KUCOIN_API_KEY", "kucoin_key");
        std::env::set_var("KUCOIN_API_SECRET", "kucoin_secret");

        let manager = ConnectorConfigManager::load_from_env();

        assert_eq!(manager.authenticated_configs().len(), 3);
        assert!(manager.get_config(&ExchangeId::Binance).is_some());
        assert!(manager.get_config(&ExchangeId::OKX).is_some());
        assert!(manager.get_config(&ExchangeId::KuCoin).is_some());

        // Cleanup
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_API_SECRET");
        std::env::remove_var("OKX_API_KEY");
        std::env::remove_var("OKX_API_SECRET");
        std::env::remove_var("KUCOIN_API_KEY");
        std::env::remove_var("KUCOIN_API_SECRET");
    }

    #[test]
    fn test_load_from_env_empty_values_ignored() {
        std::env::set_var("GEMINI_API_KEY", "");
        std::env::set_var("GEMINI_API_SECRET", "secret");

        let manager = ConnectorConfigManager::load_from_env();

        // Should not load because key is empty
        assert!(manager.get_config(&ExchangeId::Gemini).is_none());

        // Cleanup
        std::env::remove_var("GEMINI_API_KEY");
        std::env::remove_var("GEMINI_API_SECRET");
    }

    #[test]
    fn test_load_from_env_missing_secret_ignored() {
        std::env::set_var("COINBASE_API_KEY", "coinbase_key");
        // No API_SECRET set

        let manager = ConnectorConfigManager::load_from_env();

        // Should not load because secret is missing
        assert!(manager.get_config(&ExchangeId::Coinbase).is_none());

        // Cleanup
        std::env::remove_var("COINBASE_API_KEY");
    }

    #[test]
    fn test_load_from_env_case_insensitive_testnet() {
        std::env::set_var("MEXC_API_KEY", "mexc_key");
        std::env::set_var("MEXC_API_SECRET", "mexc_secret");
        std::env::set_var("MEXC_TESTNET", "TRUE"); // Uppercase

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::MEXC);
        assert!(config.is_some());
        assert!(config.unwrap().credentials.as_ref().unwrap().testnet);

        // Cleanup
        std::env::remove_var("MEXC_API_KEY");
        std::env::remove_var("MEXC_API_SECRET");
        std::env::remove_var("MEXC_TESTNET");
    }

    #[test]
    fn test_load_from_env_empty_passphrase_ignored() {
        std::env::set_var("BITGET_API_KEY", "bitget_key");
        std::env::set_var("BITGET_API_SECRET", "bitget_secret");
        std::env::set_var("BITGET_PASSPHRASE", "");

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::Bitget);
        assert!(config.is_some());

        // Empty passphrase should not be set
        assert_eq!(config.unwrap().credentials.as_ref().unwrap().passphrase, None);

        // Cleanup
        std::env::remove_var("BITGET_API_KEY");
        std::env::remove_var("BITGET_API_SECRET");
        std::env::remove_var("BITGET_PASSPHRASE");
    }

    #[test]
    fn test_load_from_env_all_enabled() {
        std::env::set_var("HTX_API_KEY", "htx_key");
        std::env::set_var("HTX_API_SECRET", "htx_secret");

        let manager = ConnectorConfigManager::load_from_env();

        let config = manager.get_config(&ExchangeId::HTX);
        assert!(config.is_some());
        assert!(config.unwrap().enabled);

        // Cleanup
        std::env::remove_var("HTX_API_KEY");
        std::env::remove_var("HTX_API_SECRET");
    }
}
