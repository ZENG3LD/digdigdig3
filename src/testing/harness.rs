//! # Test Harness
//!
//! `TestHarness` creates connectors and provides helpers for running
//! the standard test suite against any exchange.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::types::{ExchangeId, ExchangeResult};
use crate::core::traits::Credentials;
use crate::connector_manager::{AnyConnector, ConnectorFactory, ConnectorRegistry, Features};

use super::env_loader;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HARNESS
// ═══════════════════════════════════════════════════════════════════════════════

/// Test harness that can run the full test suite against any connector.
///
/// Loads credentials from `.env` at construction time and provides helpers
/// to create public or authenticated connectors, query registry metadata,
/// and look up default test symbols.
///
/// # Example
///
/// ```ignore
/// let harness = TestHarness::new();
///
/// for id in harness.available_exchanges() {
///     let connector = harness.create_authenticated(id).await
///         .expect("creds found")
///         .expect("connector created");
///     // run suite...
/// }
/// ```
pub struct TestHarness {
    credentials: HashMap<ExchangeId, Credentials>,
}

impl TestHarness {
    /// Create a new harness, loading credentials from `.env`.
    pub fn new() -> Self {
        Self {
            credentials: env_loader::load_credentials(),
        }
    }

    /// Get the list of exchanges that have API keys available in `.env`.
    pub fn available_exchanges(&self) -> Vec<ExchangeId> {
        self.credentials.keys().copied().collect()
    }

    /// Get the list of ALL registered exchanges (for public-only tests).
    ///
    /// Uses the static `ConnectorRegistry` to enumerate every known connector.
    pub fn all_exchanges() -> Vec<ExchangeId> {
        let registry = ConnectorRegistry::new();
        registry.list_all().iter().map(|m| m.id).collect()
    }

    /// Create a public (unauthenticated) connector for `id`.
    pub async fn create_public(&self, id: ExchangeId) -> ExchangeResult<Arc<AnyConnector>> {
        ConnectorFactory::create_public(id).await
    }

    /// Create an authenticated connector for `id`, if credentials are available.
    ///
    /// Returns `None` if no credentials are present for this exchange.
    /// Returns `Some(Err(_))` if credentials exist but connector creation fails.
    pub async fn create_authenticated(
        &self,
        id: ExchangeId,
    ) -> Option<ExchangeResult<Arc<AnyConnector>>> {
        let creds = self.credentials.get(&id)?.clone();
        Some(ConnectorFactory::create_authenticated(id, creds).await)
    }

    /// Get a default, liquid test symbol for an exchange.
    ///
    /// Returns a safe symbol appropriate for the exchange category:
    /// - Crypto CEX/DEX derivatives → `"BTC/USDT"` or perpetual equivalent
    /// - US stocks → `"AAPL"`
    /// - Forex → `"EUR/USD"`
    /// - Data aggregators → `"BTC/USD"`
    pub fn test_symbol(id: ExchangeId) -> &'static str {
        match id {
            // Crypto CEX — spot BTC/USDT pair
            ExchangeId::Binance
            | ExchangeId::Bybit
            | ExchangeId::OKX
            | ExchangeId::KuCoin
            | ExchangeId::Kraken
            | ExchangeId::GateIO
            | ExchangeId::Bitfinex
            | ExchangeId::Bitstamp
            | ExchangeId::Gemini
            | ExchangeId::MEXC
            | ExchangeId::HTX
            | ExchangeId::Bitget
            | ExchangeId::BingX
            | ExchangeId::Phemex
            | ExchangeId::CryptoCom
            | ExchangeId::Upbit => "BTC/USDT",

            // Coinbase uses different pair format
            ExchangeId::Coinbase => "BTC-USD",

            // Derivatives / perpetuals
            ExchangeId::Deribit => "BTC-PERPETUAL",
            ExchangeId::HyperLiquid => "BTC",
            ExchangeId::Lighter => "BTC/USDC",
            ExchangeId::Paradex => "BTC-USD-PERP",
            ExchangeId::Dydx => "BTC-USD",

            // DEX — use pool-level symbols
            ExchangeId::Uniswap => "ETH/USDC",
            ExchangeId::Jupiter => "SOL/USDC",
            ExchangeId::Raydium => "SOL/USDC",
            ExchangeId::Gmx => "BTC/USD",

            // Prediction markets
            ExchangeId::Polymarket => "BTC-2024",

            // US stocks data providers
            ExchangeId::Polygon
            | ExchangeId::Finnhub
            | ExchangeId::Tiingo
            | ExchangeId::Twelvedata
            | ExchangeId::Alpaca => "AAPL",

            // Indian brokers
            ExchangeId::AngelOne
            | ExchangeId::Zerodha
            | ExchangeId::Fyers
            | ExchangeId::Dhan
            | ExchangeId::Upstox => "RELIANCE",

            // Japan
            ExchangeId::JQuants => "7203", // Toyota

            // Korea
            ExchangeId::Krx => "005930", // Samsung

            // Russia
            ExchangeId::Tinkoff | ExchangeId::Moex => "SBER",

            // HK/CN
            ExchangeId::Futu => "00700", // Tencent HK

            // Forex brokers / data providers
            ExchangeId::Oanda | ExchangeId::AlphaVantage | ExchangeId::Dukascopy => "EUR/USD",

            // Crypto data / intelligence feeds
            ExchangeId::Coinglass
            | ExchangeId::CryptoCompare
            | ExchangeId::WhaleAlert
            | ExchangeId::Bitquery
            | ExchangeId::DefiLlama => "BTC/USD",

            // Economic data feeds
            ExchangeId::Fred => "GDP",
            ExchangeId::Bls => "LNS14000000", // US unemployment rate series

            // Multi-asset aggregators
            ExchangeId::YahooFinance => "AAPL",
            ExchangeId::Ib => "AAPL",

            // Fallback for custom / unknown
            ExchangeId::Custom(_) => "BTC/USDT",
        }
    }

    /// Get registry feature flags for an exchange, if the exchange is registered.
    pub fn features(id: ExchangeId) -> Option<Features> {
        let registry = ConnectorRegistry::new();
        registry.get(&id).map(|m| m.supported_features)
    }
}

impl Default for TestHarness {
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

    #[test]
    fn test_harness_new() {
        let h = TestHarness::new();
        // Must not panic; credentials may be empty in CI
        let _ = h.available_exchanges();
    }

    #[test]
    fn test_all_exchanges_non_empty() {
        let all = TestHarness::all_exchanges();
        assert!(!all.is_empty(), "registry should have at least one exchange");
    }

    #[test]
    fn test_test_symbol_binance() {
        assert_eq!(TestHarness::test_symbol(ExchangeId::Binance), "BTC/USDT");
    }

    #[test]
    fn test_test_symbol_coinbase() {
        assert_eq!(TestHarness::test_symbol(ExchangeId::Coinbase), "BTC-USD");
    }

    #[test]
    fn test_test_symbol_alpaca() {
        assert_eq!(TestHarness::test_symbol(ExchangeId::Alpaca), "AAPL");
    }

    #[test]
    fn test_test_symbol_oanda() {
        assert_eq!(TestHarness::test_symbol(ExchangeId::Oanda), "EUR/USD");
    }

    #[test]
    fn test_features_binance() {
        let f = TestHarness::features(ExchangeId::Binance);
        assert!(f.is_some());
        assert!(f.unwrap().market_data);
    }
}
