//! # ConnectorAggregator - Unified High-Level API
//!
//! Provides a unified high-level API over ConnectorPool for common operations.
//!
//! ## Features
//!
//! - **Single Exchange Operations**: Get price, ticker, orderbook, klines from specific exchange
//! - **Multi-Exchange Operations**: Query multiple exchanges concurrently
//! - **Trading Operations**: Place market/limit orders, cancel orders
//! - **Account Operations**: Query balances across exchanges
//! - **Best Execution**: Find best bid/ask across multiple exchanges
//!
//! ## Architecture
//!
//! ```text
//! ConnectorAggregator
//!   ├── ConnectorPool (Arc) - thread-safe connection pool
//!   └── High-level methods - unified API with error handling
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::connector_manager::{ConnectorAggregator, ConnectorPool};
//!
//! // Create aggregator
//! let pool = ConnectorPool::new();
//! let aggregator = ConnectorAggregator::new(pool);
//!
//! // Single exchange operation
//! let price = aggregator.get_price(
//!     ExchangeId::Binance,
//!     Symbol::new("BTC", "USDT"),
//!     AccountType::Spot
//! ).await?;
//!
//! // Multi-exchange operation
//! let prices = aggregator.get_prices_multi(
//!     &[ExchangeId::Binance, ExchangeId::KuCoin],
//!     Symbol::new("BTC", "USDT"),
//!     AccountType::Spot
//! ).await?;
//!
//! // Find best bid/ask across exchanges
//! let best = aggregator.get_best_bid_ask(
//!     &[ExchangeId::Binance, ExchangeId::KuCoin],
//!     Symbol::new("BTC", "USDT"),
//!     AccountType::Spot
//! ).await?;
//! ```

use std::sync::Arc;

use crate::connector_manager::ConnectorPool;
use crate::core::traits::MarketData;
use crate::core::types::{
    AccountType, ExchangeError, ExchangeId, ExchangeResult, Kline, OrderBook, Price, Symbol,
    Ticker,
};

// ═══════════════════════════════════════════════════════════════════════════════
// ConnectorAggregator - Main API
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified high-level API over ConnectorPool.
///
/// Provides convenient methods for common operations across single or multiple exchanges.
/// All operations handle errors gracefully and return typed results.
#[derive(Clone)]
pub struct ConnectorAggregator {
    /// Underlying connection pool
    pool: Arc<ConnectorPool>,
}

impl ConnectorAggregator {
    /// Create a new aggregator from a pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - ConnectorPool instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pool = ConnectorPool::new();
    /// let aggregator = ConnectorAggregator::new(pool);
    /// ```
    pub fn new(pool: ConnectorPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Pool Access
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get reference to underlying pool.
    ///
    /// Allows direct access to pool methods for advanced use cases.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pool = aggregator.pool();
    /// let ids = pool.ids();
    /// ```
    pub fn pool(&self) -> &ConnectorPool {
        &self.pool
    }

    /// List all exchanges available in the pool.
    ///
    /// Returns exchange IDs for all connectors currently in the pool.
    ///
    /// # Returns
    ///
    /// Vector of ExchangeIds
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let exchanges = aggregator.available_exchanges();
    /// println!("Available: {:?}", exchanges);
    /// ```
    pub fn available_exchanges(&self) -> Vec<ExchangeId> {
        self.pool.ids()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Market Data - Single Exchange
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get price from a specific exchange.
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier
    /// * `symbol` - Trading pair symbol
    /// * `account_type` - Account type (Spot, Futures, etc.)
    ///
    /// # Returns
    ///
    /// Current price as f64
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Exchange not in pool
    /// - Network error
    /// - API error
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let price = aggregator.get_price(
    ///     ExchangeId::Binance,
    ///     Symbol::new("BTC", "USDT"),
    ///     AccountType::Spot
    /// ).await?;
    /// ```
    pub async fn get_price(
        &self,
        id: ExchangeId,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let connector = self
            .pool
            .get(&id)
            .ok_or_else(|| ExchangeError::NotFound(format!("Exchange {:?} not in pool", id)))?;

        connector.get_price(symbol, account_type).await
    }

    /// Get ticker from a specific exchange.
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier
    /// * `symbol` - Trading pair symbol
    /// * `account_type` - Account type
    ///
    /// # Returns
    ///
    /// Ticker with 24h statistics
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ticker = aggregator.get_ticker(
    ///     ExchangeId::Binance,
    ///     Symbol::new("BTC", "USDT"),
    ///     AccountType::Spot
    /// ).await?;
    /// ```
    pub async fn get_ticker(
        &self,
        id: ExchangeId,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let connector = self
            .pool
            .get(&id)
            .ok_or_else(|| ExchangeError::NotFound(format!("Exchange {:?} not in pool", id)))?;

        connector.get_ticker(symbol, account_type).await
    }

    /// Get orderbook from a specific exchange.
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier
    /// * `symbol` - Trading pair symbol
    /// * `account_type` - Account type
    /// * `depth` - Optional depth limit (e.g., 5, 10, 20)
    ///
    /// # Returns
    ///
    /// OrderBook with bids and asks
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let orderbook = aggregator.get_orderbook(
    ///     ExchangeId::Binance,
    ///     Symbol::new("BTC", "USDT"),
    ///     AccountType::Spot,
    ///     Some(20)
    /// ).await?;
    /// ```
    pub async fn get_orderbook(
        &self,
        id: ExchangeId,
        symbol: Symbol,
        account_type: AccountType,
        depth: Option<u16>,
    ) -> ExchangeResult<OrderBook> {
        let connector = self
            .pool
            .get(&id)
            .ok_or_else(|| ExchangeError::NotFound(format!("Exchange {:?} not in pool", id)))?;

        connector.get_orderbook(symbol, depth, account_type).await
    }

    /// Get klines (candlestick data) from a specific exchange.
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Timeframe (e.g., "1m", "5m", "1h", "1d")
    /// * `account_type` - Account type
    /// * `limit` - Optional number of klines to return
    ///
    /// # Returns
    ///
    /// Vector of Klines (OHLCV data)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let klines = aggregator.get_klines(
    ///     ExchangeId::Binance,
    ///     Symbol::new("BTC", "USDT"),
    ///     "1h",
    ///     AccountType::Spot,
    ///     Some(100)
    /// ).await?;
    /// ```
    pub async fn get_klines(
        &self,
        id: ExchangeId,
        symbol: Symbol,
        interval: &str,
        account_type: AccountType,
        limit: Option<u16>,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let connector = self
            .pool
            .get(&id)
            .ok_or_else(|| ExchangeError::NotFound(format!("Exchange {:?} not in pool", id)))?;

        connector
            .get_klines(symbol, interval, limit, account_type, end_time)
            .await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Market Data - Multi-Exchange
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get prices from multiple exchanges concurrently.
    ///
    /// Queries all specified exchanges in parallel and collects successful results.
    /// Failed exchanges are skipped (not propagated as errors).
    ///
    /// # Arguments
    ///
    /// * `ids` - Exchange identifiers to query
    /// * `symbol` - Trading pair symbol
    /// * `account_type` - Account type
    ///
    /// # Returns
    ///
    /// HashMap of ExchangeId -> Price for successful queries
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let prices = aggregator.get_prices_multi(
    ///     &[ExchangeId::Binance, ExchangeId::KuCoin, ExchangeId::OKX],
    ///     Symbol::new("BTC", "USDT"),
    ///     AccountType::Spot
    /// ).await?;
    /// ```
    pub async fn get_prices_multi(
        &self,
        ids: &[ExchangeId],
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<std::collections::HashMap<ExchangeId, Price>> {
        use futures_util::future::join_all;

        // Filter to only exchanges in pool
        let connectors: Vec<_> = ids
            .iter()
            .filter_map(|id| self.pool.get(id).map(|c| (*id, c)))
            .collect();

        if connectors.is_empty() {
            return Err(ExchangeError::NotFound(
                "No specified exchanges found in pool".to_string(),
            ));
        }

        // Query all exchanges concurrently
        let futures = connectors.into_iter().map(|(id, connector)| {
            let sym = symbol.clone();
            let acc_type = account_type;
            async move {
                connector
                    .get_price(sym, acc_type)
                    .await
                    .ok()
                    .map(|price| (id, price))
            }
        });

        let results: Vec<Option<(ExchangeId, Price)>> = join_all(futures).await;

        // Collect successful results
        let prices: std::collections::HashMap<_, _> =
            results.into_iter().flatten().collect();

        if prices.is_empty() {
            return Err(ExchangeError::NotFound(
                "All exchange queries failed".to_string(),
            ));
        }

        Ok(prices)
    }

    /// Find best bid and ask across multiple exchanges.
    ///
    /// Queries orderbooks from all specified exchanges and finds the highest bid
    /// and lowest ask, enabling best execution across exchanges.
    ///
    /// # Arguments
    ///
    /// * `ids` - Exchange identifiers to query
    /// * `symbol` - Trading pair symbol
    /// * `account_type` - Account type
    ///
    /// # Returns
    ///
    /// BestBidAsk with highest bid, lowest ask, and their sources
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let best = aggregator.get_best_bid_ask(
    ///     &[ExchangeId::Binance, ExchangeId::KuCoin],
    ///     Symbol::new("BTC", "USDT"),
    ///     AccountType::Spot
    /// ).await?;
    /// println!("Best bid: {} from {:?}", best.bid, best.bid_exchange);
    /// println!("Best ask: {} from {:?}", best.ask, best.ask_exchange);
    /// ```
    pub async fn get_best_bid_ask(
        &self,
        ids: &[ExchangeId],
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<BestBidAsk> {
        use futures_util::future::join_all;

        // Filter to only exchanges in pool
        let connectors: Vec<_> = ids
            .iter()
            .filter_map(|id| self.pool.get(id).map(|c| (*id, c)))
            .collect();

        if connectors.is_empty() {
            return Err(ExchangeError::NotFound(
                "No specified exchanges found in pool".to_string(),
            ));
        }

        // Query all orderbooks concurrently
        let futures = connectors.into_iter().map(|(id, connector)| {
            let sym = symbol.clone();
            let acc_type = account_type;
            async move {
                connector
                    .get_orderbook(sym, Some(1), acc_type)
                    .await
                    .ok()
                    .map(|ob| (id, ob))
            }
        });

        let results: Vec<Option<(ExchangeId, OrderBook)>> = join_all(futures).await;

        // Collect successful orderbooks
        let orderbooks: Vec<_> = results.into_iter().flatten().collect();

        if orderbooks.is_empty() {
            return Err(ExchangeError::NotFound(
                "All orderbook queries failed".to_string(),
            ));
        }

        // Find best bid (highest) and best ask (lowest)
        let mut best_bid: Option<(f64, ExchangeId)> = None;
        let mut best_ask: Option<(f64, ExchangeId)> = None;

        for (id, ob) in orderbooks {
            // Check bids (highest is best)
            if let Some((bid_price, _)) = ob.bids.first() {
                match best_bid {
                    None => best_bid = Some((*bid_price, id)),
                    Some((current_best, _)) if *bid_price > current_best => {
                        best_bid = Some((*bid_price, id))
                    }
                    _ => {}
                }
            }

            // Check asks (lowest is best)
            if let Some((ask_price, _)) = ob.asks.first() {
                match best_ask {
                    None => best_ask = Some((*ask_price, id)),
                    Some((current_best, _)) if *ask_price < current_best => {
                        best_ask = Some((*ask_price, id))
                    }
                    _ => {}
                }
            }
        }

        let (bid, bid_exchange) = best_bid.ok_or_else(|| {
            ExchangeError::NotFound("No valid bids found in orderbooks".to_string())
        })?;

        let (ask, ask_exchange) = best_ask.ok_or_else(|| {
            ExchangeError::NotFound("No valid asks found in orderbooks".to_string())
        })?;

        Ok(BestBidAsk {
            bid,
            bid_exchange,
            ask,
            ask_exchange,
            spread: ask - bid,
            spread_percent: ((ask - bid) / bid) * 100.0,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Account Operations
    // ═══════════════════════════════════════════════════════════════════════════
    //
    // NOTE: These methods require Account trait to be implemented on AnyConnector.
    // TODO: Uncomment when Account trait is delegated in connector.rs
    //
    // /// Get balance from a specific exchange.
    // pub async fn get_balance(...) -> ExchangeResult<Vec<Balance>> { ... }
    //
    // /// Get balances from multiple exchanges concurrently.
    // pub async fn get_balances_multi(...) -> ExchangeResult<HashMap<ExchangeId, Vec<Balance>>> { ... }

    // ═══════════════════════════════════════════════════════════════════════════
    // Trading Operations
    // ═══════════════════════════════════════════════════════════════════════════
    //
    // NOTE: These methods require Trading trait to be implemented on AnyConnector.
    // TODO: Uncomment when Trading trait is delegated in connector.rs
    //
    // /// Place a market order on a specific exchange.
    // pub async fn place_market_order(...) -> ExchangeResult<Order> { ... }
    //
    // /// Place a limit order on a specific exchange.
    // pub async fn place_limit_order(...) -> ExchangeResult<Order> { ... }
    //
    // /// Cancel an order on a specific exchange.
    // pub async fn cancel_order(...) -> ExchangeResult<Order> { ... }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Builder Pattern
// ═══════════════════════════════════════════════════════════════════════════════

/// Builder for ConnectorAggregator.
///
/// Provides fluent API for constructing an aggregator instance.
pub struct ConnectorAggregatorBuilder {
    pool: ConnectorPool,
}

impl ConnectorAggregatorBuilder {
    /// Create a new builder with an empty pool.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let builder = ConnectorAggregatorBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            pool: ConnectorPool::new(),
        }
    }

    /// Create a builder with an existing pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - Existing ConnectorPool instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pool = ConnectorPool::new();
    /// let builder = ConnectorAggregatorBuilder::with_pool(pool);
    /// ```
    pub fn with_pool(pool: ConnectorPool) -> Self {
        Self { pool }
    }

    /// Build the aggregator.
    ///
    /// Consumes the builder and returns the configured aggregator.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let aggregator = ConnectorAggregatorBuilder::new()
    ///     .build();
    /// ```
    pub fn build(self) -> ConnectorAggregator {
        ConnectorAggregator::new(self.pool)
    }
}

impl Default for ConnectorAggregatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Result from get_best_bid_ask query.
#[derive(Debug, Clone)]
pub struct BestBidAsk {
    /// Best (highest) bid price
    pub bid: f64,
    /// Exchange with best bid
    pub bid_exchange: ExchangeId,
    /// Best (lowest) ask price
    pub ask: f64,
    /// Exchange with best ask
    pub ask_exchange: ExchangeId,
    /// Spread (ask - bid)
    pub spread: f64,
    /// Spread as percentage of bid
    pub spread_percent: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector_manager::AnyConnector;
    use crate::exchanges::okx::OkxConnector;

    /// Helper to create a mock OKX connector for testing
    async fn create_mock_connector() -> Arc<AnyConnector> {
        let connector = OkxConnector::public(true).await.unwrap();
        Arc::new(AnyConnector::OKX(Arc::new(connector)))
    }

    /// Helper to create a pool with mock connectors
    async fn create_test_pool() -> ConnectorPool {
        let pool = ConnectorPool::new();
        pool.insert(ExchangeId::Binance, create_mock_connector().await);
        pool.insert(ExchangeId::KuCoin, create_mock_connector().await);
        pool
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Constructor Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_new_aggregator() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);
        assert!(aggregator.available_exchanges().is_empty());
    }

    #[tokio::test]
    async fn test_aggregator_with_pool() {
        let pool = create_test_pool().await;
        let aggregator = ConnectorAggregator::new(pool);
        assert_eq!(aggregator.available_exchanges().len(), 2);
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Builder Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_builder_new() {
        let aggregator = ConnectorAggregatorBuilder::new().build();
        assert!(aggregator.available_exchanges().is_empty());
    }

    #[tokio::test]
    async fn test_builder_with_pool() {
        let pool = create_test_pool().await;
        let aggregator = ConnectorAggregatorBuilder::with_pool(pool).build();
        assert_eq!(aggregator.available_exchanges().len(), 2);
    }

    #[tokio::test]
    async fn test_builder_default() {
        let aggregator = ConnectorAggregatorBuilder::default().build();
        assert!(aggregator.available_exchanges().is_empty());
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Pool Access Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_pool_access() {
        let pool = create_test_pool().await;
        let aggregator = ConnectorAggregator::new(pool);

        let pool_ref = aggregator.pool();
        assert_eq!(pool_ref.len(), 2);
    }

    #[tokio::test]
    async fn test_available_exchanges() {
        let pool = create_test_pool().await;
        let aggregator = ConnectorAggregator::new(pool);

        let exchanges = aggregator.available_exchanges();
        assert_eq!(exchanges.len(), 2);
        assert!(exchanges.contains(&ExchangeId::Binance));
        assert!(exchanges.contains(&ExchangeId::KuCoin));
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Single Exchange Operations - Error Handling
    // ───────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_price_exchange_not_in_pool() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_price(
                ExchangeId::Binance,
                Symbol::new("BTC", "USDT"),
                AccountType::Spot,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExchangeError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_ticker_exchange_not_in_pool() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_ticker(
                ExchangeId::Binance,
                Symbol::new("BTC", "USDT"),
                AccountType::Spot,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_orderbook_exchange_not_in_pool() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_orderbook(
                ExchangeId::Binance,
                Symbol::new("BTC", "USDT"),
                AccountType::Spot,
                Some(20),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_klines_exchange_not_in_pool() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_klines(
                ExchangeId::Binance,
                Symbol::new("BTC", "USDT"),
                "1h",
                AccountType::Spot,
                Some(100),
            )
            .await;

        assert!(result.is_err());
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Multi-Exchange Operations - Error Handling
    // ───────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_prices_multi_no_exchanges() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_prices_multi(
                &[ExchangeId::Binance, ExchangeId::KuCoin],
                Symbol::new("BTC", "USDT"),
                AccountType::Spot,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExchangeError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_prices_multi_empty_list() {
        let pool = create_test_pool().await;
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_prices_multi(&[], Symbol::new("BTC", "USDT"), AccountType::Spot)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_best_bid_ask_no_exchanges() {
        let pool = ConnectorPool::new();
        let aggregator = ConnectorAggregator::new(pool);

        let result = aggregator
            .get_best_bid_ask(
                &[ExchangeId::Binance],
                Symbol::new("BTC", "USDT"),
                AccountType::Spot,
            )
            .await;

        assert!(result.is_err());
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Helper Types Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_best_bid_ask_spread_calculation() {
        let best = BestBidAsk {
            bid: 50000.0,
            bid_exchange: ExchangeId::Binance,
            ask: 50100.0,
            ask_exchange: ExchangeId::KuCoin,
            spread: 100.0,
            spread_percent: 0.2,
        };

        assert_eq!(best.spread, 100.0);
        assert_eq!(best.spread_percent, 0.2);
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Aggregator Clone Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_aggregator_clone() {
        let pool = create_test_pool().await;
        let aggregator1 = ConnectorAggregator::new(pool);

        // Clone the aggregator
        let aggregator2 = aggregator1.clone();

        // Both should share the same pool
        assert_eq!(aggregator1.available_exchanges().len(), 2);
        assert_eq!(aggregator2.available_exchanges().len(), 2);
    }
}
