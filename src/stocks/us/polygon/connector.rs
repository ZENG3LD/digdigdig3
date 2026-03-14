//! # Polygon.io Connector
//!
//! Main connector implementation with trait implementations.
//!
//! ## Trait Implementation Status
//! - `ExchangeIdentity`: Yes (basic identification)
//! - `MarketData`: Yes (full implementation)
//! - `Trading`: No (returns UnsupportedOperation - data provider only)
//! - `Account`: No (returns UnsupportedOperation - data provider only)
//! - `Positions`: No (returns UnsupportedOperation - data provider only)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, Balance, AccountInfo, Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::WeightRateLimiter;
use crate::core::types::SymbolInfo;

use super::endpoints::{PolygonUrls, PolygonEndpoint, format_symbol, map_timespan, extract_multiplier};
use super::auth::PolygonAuth;
use super::parser::PolygonParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Polygon.io connector
pub struct PolygonConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: PolygonAuth,
    /// URLs
    urls: PolygonUrls,
    /// Use real-time WebSocket (true) or delayed (false)
    _realtime: bool,
    /// Rate limiter (5 req/min for free tier, higher for paid)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl PolygonConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `credentials` - API credentials (requires api_key)
    /// * `realtime` - Use real-time data (requires Advanced+ plan)
    pub async fn new(credentials: Credentials, realtime: bool) -> ExchangeResult<Self> {
        let auth = PolygonAuth::new(&credentials)?;
        let urls = PolygonUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Initialize rate limiter: 5 req/min for free tier
        // Note: Paid tiers have higher limits (100+/min)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(5, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            _realtime: realtime,
            rate_limiter,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return;
                }
                limiter.time_until_ready(weight)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request with authentication
    async fn get(
        &self,
        endpoint: PolygonEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();

        // Add API key to params
        self.auth.add_to_params(&mut params);

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Make request
        let response = self.http.get(&url, &HashMap::new()).await?;

        Ok(response)
    }

    /// Build URL with path parameters
    fn build_path(&self, endpoint: PolygonEndpoint, path_params: &HashMap<&str, String>) -> String {
        let mut path = endpoint.path().to_string();

        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        path
    }

    /// GET request with path and query parameters
    async fn get_with_path(
        &self,
        endpoint: PolygonEndpoint,
        path_params: HashMap<&str, String>,
        mut query_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest_url();
        let path = self.build_path(endpoint, &path_params);

        // Add API key to params
        self.auth.add_to_params(&mut query_params);

        // Build query string
        let query = if query_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Make request
        let response = self.http.get(&url, &HashMap::new()).await?;

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Options
    // ═══════════════════════════════════════════════════════════════════════════

    /// Options contracts reference data — `GET /v3/reference/options/contracts`
    ///
    /// Optional params: `underlying_ticker`, `contract_type` ("call"/"put"),
    /// `expiration_date`, `strike_price`, `order`, `limit`, `sort`.
    pub async fn get_options_contracts(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(PolygonEndpoint::OptionsContracts, params).await
    }

    /// Options chain snapshot — `GET /v3/snapshot/options/{underlyingAsset}`
    ///
    /// `underlying_asset` — ticker of the underlying (e.g. `"AAPL"`).
    /// Optional params: `contract_type`, `expiration_date`, `strike_price`, `limit`.
    pub async fn get_options_chain(
        &self,
        underlying_asset: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let path_params = {
            let mut m = HashMap::new();
            m.insert("underlyingAsset", underlying_asset.to_uppercase());
            m
        };
        self.get_with_path(PolygonEndpoint::OptionsChain, path_params, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Indices
    // ═══════════════════════════════════════════════════════════════════════════

    /// Indices snapshot — `GET /v3/snapshot/indices`
    ///
    /// Optional params: `ticker` (comma-separated), `order`, `limit`, `sort`.
    pub async fn get_indices_snapshot(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(PolygonEndpoint::IndicesSnapshot, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Forex
    // ═══════════════════════════════════════════════════════════════════════════

    /// Forex last quote — `GET /v1/last_quote/currencies/{from}/{to}`
    ///
    /// `from_currency` / `to_currency` — ISO currency codes (e.g. `"USD"`, `"EUR"`).
    pub async fn get_forex_quote(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> ExchangeResult<Value> {
        let path_params = {
            let mut m = HashMap::new();
            m.insert("from", from_currency.to_uppercase());
            m.insert("to", to_currency.to_uppercase());
            m
        };
        self.get_with_path(PolygonEndpoint::ForexQuote, path_params, HashMap::new()).await
    }

    /// Forex OHLCV aggregates — `GET /v2/aggs/ticker/{ticker}/range/{mul}/{res}/{from}/{to}`
    ///
    /// `ticker` — Forex ticker (e.g. `"C:EURUSD"`).
    /// `multiplier` — size of the aggregate (e.g. 1, 5, 15).
    /// `timespan` — "minute", "hour", "day", "week", "month", "quarter", "year".
    /// `from` / `to` — dates in `YYYY-MM-DD` or millisecond epoch format.
    pub async fn get_forex_aggregates(
        &self,
        ticker: &str,
        multiplier: u32,
        timespan: &str,
        from: &str,
        to: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let path_params = {
            let mut m = HashMap::new();
            m.insert("ticker", ticker.to_uppercase());
            m.insert("multiplier", multiplier.to_string());
            m.insert("timespan", timespan.to_string());
            m.insert("from", from.to_string());
            m.insert("to", to.to_string());
            m
        };
        self.get_with_path(PolygonEndpoint::ForexAggregates, path_params, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Crypto Snapshot
    // ═══════════════════════════════════════════════════════════════════════════

    /// Crypto snapshot (all tickers) — `GET /v2/snapshot/locale/global/markets/crypto/tickers`
    ///
    /// Optional params: `tickers` (comma-separated list to filter).
    pub async fn get_crypto_snapshot(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(PolygonEndpoint::CryptoSnapshot, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Reference Data
    // ═══════════════════════════════════════════════════════════════════════════

    /// Trade conditions reference — `GET /v3/reference/conditions`
    ///
    /// Optional params: `asset_class`, `data_type`, `id`, `sip`, `order`, `limit`, `sort`.
    pub async fn get_reference_conditions(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(PolygonEndpoint::ReferenceConditions, params).await
    }

    /// Exchanges reference — `GET /v3/reference/exchanges`
    ///
    /// Optional params: `asset_class`, `locale`, `order`, `limit`, `sort`.
    pub async fn get_reference_exchanges(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(PolygonEndpoint::ReferenceExchanges, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for PolygonConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Polygon
    }

    fn is_testnet(&self) -> bool {
        false // Polygon doesn't have testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Data provider only, but we use Spot as default for compatibility
        vec![AccountType::Spot]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex // Data provider
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for PolygonConnector {
    /// Get current price
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);

        let path_params = vec![("ticker", ticker_symbol)]
            .into_iter()
            .collect();

        // Use PreviousClose endpoint (free tier) instead of SingleSnapshot
        // This gives us yesterday's close price, which is better than no data
        let response = self.get_with_path(
            PolygonEndpoint::PreviousClose,
            path_params,
            HashMap::new(),
        ).await?;

        // PreviousClose returns aggregates format in an array
        // Extract close price from results array
        let results = PolygonParser::extract_results(&response)?;
        if let Some(arr) = results.as_array() {
            if let Some(first) = arr.first() {
                if let Some(close) = first.get("c").and_then(|v| v.as_f64()) {
                    return Ok(close);
                }
            }
        }

        Err(ExchangeError::Parse("Could not extract close price".to_string()))
    }

    /// Get orderbook (only best bid/ask available)
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);

        let path_params = vec![("ticker", ticker_symbol)]
            .into_iter()
            .collect();

        let response = self.get_with_path(
            PolygonEndpoint::SingleSnapshot,
            path_params,
            HashMap::new(),
        ).await?;

        PolygonParser::parse_orderbook(&response)
    }

    /// Get klines (OHLC aggregates)
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);
        let timespan = map_timespan(interval);
        let multiplier = extract_multiplier(interval);

        // Calculate date range (use last 30 days for default)
        let to = chrono::Utc::now();
        let from = to - chrono::Duration::days(30);

        let path_params = vec![
            ("ticker", ticker_symbol),
            ("multiplier", multiplier.to_string()),
            ("timespan", timespan.to_string()),
            ("from", from.format("%Y-%m-%d").to_string()),
            ("to", to.format("%Y-%m-%d").to_string()),
        ]
        .into_iter()
        .collect();

        let mut query_params = HashMap::new();
        query_params.insert("adjusted".to_string(), "true".to_string());
        query_params.insert("sort".to_string(), "asc".to_string());

        if let Some(lim) = limit {
            query_params.insert("limit".to_string(), lim.to_string());
        } else {
            query_params.insert("limit".to_string(), "5000".to_string());
        }

        let response = self.get_with_path(
            PolygonEndpoint::Aggregates,
            path_params,
            query_params,
        ).await?;

        PolygonParser::parse_klines(&response)
    }

    /// Get 24h ticker
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);

        let path_params = vec![("ticker", ticker_symbol)]
            .into_iter()
            .collect();

        let response = self.get_with_path(
            PolygonEndpoint::SingleSnapshot,
            path_params,
            HashMap::new(),
        ).await?;

        PolygonParser::parse_ticker(&response)
    }

    /// Ping (check connection)
    async fn ping(&self) -> ExchangeResult<()> {
        // Use market status endpoint as ping
        let response = self.get(
            PolygonEndpoint::MarketStatus,
            HashMap::new(),
        ).await?;

        if response.get("status").and_then(|s| s.as_str()) == Some("OK") {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }

    /// Get exchange info — returns a paginated list of US stock tickers
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("market".to_string(), "stocks".to_string());
        params.insert("active".to_string(), "true".to_string());
        params.insert("limit".to_string(), "1000".to_string());

        let response = self.get(PolygonEndpoint::Tickers, params).await?;

        let results = response.get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing results array".to_string()))?;

        let infos = results.iter().filter_map(|item| {
            let ticker = item.get("ticker")?.as_str()?.to_string();
            let _name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let currency = item.get("currency_name")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_uppercase();

            Some(SymbolInfo {
                symbol: ticker.clone(),
                base_asset: ticker,
                quote_asset: currency,
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: Some(1.0),
                max_quantity: None,
                tick_size: None,
                step_size: Some(1.0),
                min_notional: None,
            })
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for PolygonConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for PolygonConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Account operations are not supported.".to_string()
        ))
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Account operations are not supported.".to_string()
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Account operations are not supported.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for PolygonConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Position operations are not supported.".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Position operations are not supported.".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Position operations are not supported.".to_string()
        ))
    }
}
