//! # Finnhub Connector
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

use super::endpoints::{FinnhubUrls, FinnhubEndpoint, format_symbol, map_resolution};
use super::auth::FinnhubAuth;
use super::parser::FinnhubParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Finnhub connector
pub struct FinnhubConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: FinnhubAuth,
    /// URLs
    urls: FinnhubUrls,
    /// Rate limiter (60 req/min for free tier, 30 req/sec hard cap)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl FinnhubConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `credentials` - API credentials (requires api_key)
    pub async fn new(credentials: Credentials) -> ExchangeResult<Self> {
        let auth = FinnhubAuth::new(&credentials)?;
        let urls = FinnhubUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Initialize rate limiter: 60 req/min for free tier
        // Note: There's a hard cap of 30 req/sec across all tiers
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(60, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
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
        endpoint: FinnhubEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();

        // Build query string with auth
        let mut all_params = params;
        self.auth.add_to_params(&mut all_params);

        let query = if all_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = all_params.iter()
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
    // EXTENDED METHODS — ETF Data
    // ═══════════════════════════════════════════════════════════════════════════

    /// ETF holdings — `GET /api/v1/etf/holdings`
    ///
    /// `symbol` — ETF ticker (e.g. `"SPY"`); optional `date` in `YYYY-MM-DD` format.
    pub async fn get_etf_holdings(
        &self,
        symbol: &str,
        date: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(d) = date {
            params.insert("date".to_string(), d.to_string());
        }
        self.get(FinnhubEndpoint::EtfHoldings, params).await
    }

    /// ETF profile — `GET /api/v1/etf/profile`
    ///
    /// `symbol` — ETF ticker.
    pub async fn get_etf_profile(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        self.get(FinnhubEndpoint::EtfProfile, params).await
    }

    /// ETF country exposure — `GET /api/v1/etf/country`
    ///
    /// `symbol` — ETF ticker.
    pub async fn get_etf_country_exposure(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        self.get(FinnhubEndpoint::EtfCountryExposure, params).await
    }

    /// ETF sector exposure — `GET /api/v1/etf/sector`
    ///
    /// `symbol` — ETF ticker.
    pub async fn get_etf_sector_exposure(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        self.get(FinnhubEndpoint::EtfSectorExposure, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — IPO & Earnings
    // ═══════════════════════════════════════════════════════════════════════════

    /// IPO calendar — `GET /api/v1/calendar/ipo`
    ///
    /// `from` / `to` — date range in `YYYY-MM-DD` format.
    pub async fn get_ipo_calendar(&self, from: &str, to: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());
        self.get(FinnhubEndpoint::IpoCalendar, params).await
    }

    /// Earnings surprise — `GET /api/v1/stock/earnings`
    ///
    /// `symbol` — stock ticker; optional `limit` (number of quarters, default 4).
    pub async fn get_earnings_surprise(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(FinnhubEndpoint::EarningsSurprise, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Social Sentiment
    // ═══════════════════════════════════════════════════════════════════════════

    /// Social sentiment — `GET /api/v1/stock/social-sentiment`
    ///
    /// `symbol` — stock ticker; optional `from` / `to` in `YYYY-MM-DD` format.
    pub async fn get_social_sentiment(
        &self,
        symbol: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(f) = from {
            params.insert("from".to_string(), f.to_string());
        }
        if let Some(t) = to {
            params.insert("to".to_string(), t.to_string());
        }
        self.get(FinnhubEndpoint::SocialSentiment, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Crypto Profile
    // ═══════════════════════════════════════════════════════════════════════════

    /// Crypto profile — `GET /api/v1/crypto/profile`
    ///
    /// `symbol` — crypto symbol (e.g. `"BINANCE:BTCUSDT"`).
    pub async fn get_crypto_profile(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.get(FinnhubEndpoint::CryptoProfile, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for FinnhubConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Finnhub
    }

    fn is_testnet(&self) -> bool {
        false // Finnhub doesn't have testnet
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
impl MarketData for FinnhubConnector {
    /// Get current price
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), ticker_symbol);

        let response = self.get(
            FinnhubEndpoint::Quote,
            params,
        ).await?;

        FinnhubParser::parse_price(&response)
    }

    /// Get orderbook (only best bid/ask available on premium tier)
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), ticker_symbol);

        let response = self.get(
            FinnhubEndpoint::BidAsk,
            params,
        ).await?;

        FinnhubParser::parse_orderbook(&response)
    }

    /// Get klines (OHLC candles)
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
        let resolution = map_resolution(interval);

        // Calculate date range
        // Finnhub requires UNIX timestamps (seconds, not milliseconds)
        let to = chrono::Utc::now().timestamp();
        let from = if let Some(lim) = limit {
            // Calculate from timestamp based on interval and limit
            let seconds_per_candle = match resolution {
                "1" => 60,           // 1 minute
                "5" => 300,          // 5 minutes
                "15" => 900,         // 15 minutes
                "30" => 1800,        // 30 minutes
                "60" => 3600,        // 1 hour
                "D" => 86400,        // 1 day
                "W" => 604800,       // 1 week
                "M" => 2592000,      // ~30 days
                _ => 86400,
            };
            to - (lim as i64 * seconds_per_candle)
        } else {
            // Default: last 30 days
            to - (30 * 86400)
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), ticker_symbol);
        params.insert("resolution".to_string(), resolution.to_string());
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        let response = self.get(
            FinnhubEndpoint::StockCandles,
            params,
        ).await?;

        FinnhubParser::parse_klines(&response)
    }

    /// Get 24h ticker
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Use only base symbol (ticker) for stocks
        let ticker_symbol = format_symbol(&symbol.base);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), ticker_symbol.clone());

        let response = self.get(
            FinnhubEndpoint::Quote,
            params,
        ).await?;

        let mut ticker = FinnhubParser::parse_ticker(&response)?;
        ticker.symbol = ticker_symbol;
        Ok(ticker)
    }

    /// Ping (check connection)
    async fn ping(&self) -> ExchangeResult<()> {
        // Use a lightweight endpoint to check connection
        // Market status is a good choice as it doesn't require a symbol
        let mut params = HashMap::new();
        params.insert("exchange".to_string(), "US".to_string());

        let response = self.get(
            FinnhubEndpoint::MarketStatus,
            params,
        ).await?;

        // If we got a response without error, connection is OK
        FinnhubParser::check_error(&response)?;
        Ok(())
    }

    /// Get exchange info — returns US stock symbols from Finnhub
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("exchange".to_string(), "US".to_string());

        let response = self.get(FinnhubEndpoint::StockSymbols, params).await?;

        // Response is an array of symbol objects
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of symbols".to_string()))?;

        let infos = arr.iter().filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let currency = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_uppercase();

            Some(SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: currency,
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: Some(1.0),
                max_quantity: None,
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
impl Trading for FinnhubConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Trading is not supported.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for FinnhubConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Account operations are not supported.".to_string()
        ))
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Account operations are not supported.".to_string()
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Account operations are not supported.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for FinnhubConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Position operations are not supported.".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Position operations are not supported.".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Finnhub is a data provider, not an exchange. Position operations are not supported.".to_string()
        ))
    }
}
