//! # Coinglass Connector
//!
//! Реализация коннектора для Coinglass API V4.
//!
//! ## Important Notes
//!
//! Coinglass is a DERIVATIVES ANALYTICS provider, not a trading exchange:
//! - NO standard price/OHLC data (use exchanges for that)
//! - NO trading operations (Trading trait returns UnsupportedOperation)
//! - NO account balances (Account trait returns UnsupportedOperation)
//! - Focus: Liquidations, Open Interest, Funding Rates, Long/Short Ratios
//!
//! ## Custom Methods
//!
//! Since Coinglass doesn't fit standard MarketData/Trading/Account patterns,
//! custom methods are provided as direct connector methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, Balance, AccountInfo,
    Position, FundingRate, SymbolInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{CoinglassUrls, CoinglassEndpoint};
use super::auth::CoinglassAuth;
use super::parser::{
    CoinglassParser,
    LiquidationData, OpenInterestOhlc, FundingRateData, LongShortRatio,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Coinglass коннектор
pub struct CoinglassConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация
    auth: CoinglassAuth,
    /// URL'ы
    urls: CoinglassUrls,
    /// Rate limiter (varies by subscription tier)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl CoinglassConnector {
    /// Создать новый коннектор
    ///
    /// # Arguments
    /// * `credentials` - API credentials (requires api_key)
    /// * `rate_limit_per_min` - Rate limit (30 for Hobbyist, 80 for Startup, etc.)
    pub async fn new(credentials: Credentials, rate_limit_per_min: u32) -> ExchangeResult<Self> {
        let auth = CoinglassAuth::new(&credentials)?;
        let urls = CoinglassUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Initialize rate limiter: rate_limit requests per 60 seconds
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(rate_limit_per_min, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
        })
    }

    /// Create connector with default rate limit (30 req/min - Hobbyist tier)
    pub async fn new_with_default_limit(credentials: Credentials) -> ExchangeResult<Self> {
        Self::new(credentials, 30).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready(weight)
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: CoinglassEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Add auth headers
        let headers = self.auth.get_headers();

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;

        // Check for API errors
        if !CoinglassParser::is_success(&response) {
            let error_msg = CoinglassParser::extract_error(&response);
            let error_code = response.get("code")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            return Err(ExchangeError::Api {
                code: error_code,
                message: error_msg,
            });
        }

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - MARKET DISCOVERY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get list of supported coins
    pub async fn get_supported_coins(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(CoinglassEndpoint::SupportedCoins, HashMap::new()).await?;
        CoinglassParser::parse_supported_coins(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - LIQUIDATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get liquidation history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval ("1m", "5m", "15m", "1h", "4h", "12h", "1d")
    /// * `limit` - Number of data points (optional)
    pub async fn get_liquidation_history(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LiquidationData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::LiquidationHistory, params).await?;
        CoinglassParser::parse_liquidations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - OPEN INTEREST
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get Open Interest OHLC aggregated history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval ("1m", "5m", "15m", "1h", "4h", "12h", "1d")
    /// * `limit` - Number of data points (optional)
    pub async fn get_open_interest_ohlc(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenInterestOhlc>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::OpenInterestOhlc, params).await?;
        CoinglassParser::parse_oi_ohlc(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - FUNDING RATES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get funding rate history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `exchange` - Exchange name (optional, e.g., "Binance")
    /// * `limit` - Number of data points (optional)
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<FundingRateData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::FundingRateHistory, params).await?;
        CoinglassParser::parse_funding_rates(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - LONG/SHORT RATIOS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get long/short ratio history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval ("1m", "5m", "15m", "1h", "4h", "12h", "1d")
    /// * `limit` - Number of data points (optional)
    pub async fn get_long_short_ratio(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LongShortRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::LongShortRateHistory, params).await?;
        CoinglassParser::parse_long_short_ratio(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for CoinglassConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Coinglass
    }

    fn is_testnet(&self) -> bool {
        false // Coinglass only has mainnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Coinglass is a data provider, doesn't support traditional account types
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNSUPPORTED TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for CoinglassConnector {
    async fn ping(&self) -> ExchangeResult<()> {
        // Test with supported-coins endpoint (simplest endpoint)
        match self.get(CoinglassEndpoint::SupportedCoins, HashMap::new()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let coins = self.get_supported_coins().await?;

        let infos = coins
            .into_iter()
            .map(|coin| SymbolInfo {
                symbol: coin.clone(),
                base_asset: coin,
                quote_asset: "USD".to_string(), // Coinglass tracks derivatives quoted in USD
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
                min_notional: None,
            })
            .collect();

        Ok(infos)
    }

    async fn get_price(&self, _symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide standard price data. Use get_open_interest_ohlc() or other custom methods.".to_string()
        ))
    }

    async fn get_orderbook(&self, _symbol: Symbol, _depth: Option<u16>, _account_type: AccountType) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide orderbook data.".to_string()
        ))
    }

    async fn get_klines(&self, _symbol: Symbol, _interval: &str, _limit: Option<u16>, _account_type: AccountType, _end_time: Option<i64>) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide standard klines. Use get_open_interest_ohlc() for OI OHLC data.".to_string()
        ))
    }

    async fn get_ticker(&self, _symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide ticker data.".to_string()
        ))
    }
}






