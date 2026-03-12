//! # Tiingo Connector
//!
//! Main connector implementation with trait implementations.
//!
//! ## Trait Implementation Status
//! - `ExchangeIdentity`: Yes (basic identification)
//! - `MarketData`: Yes (full implementation for stocks/crypto/forex)
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
    ExchangeId, ExchangeType, AccountType, Symbol, Asset,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, Balance, AccountInfo, Position, FundingRate,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::WeightRateLimiter;
use crate::core::types::SymbolInfo;

use super::endpoints::{
    TiingoUrls, TiingoEndpoint,
    format_stock_symbol, format_crypto_symbol, format_forex_symbol,
    map_interval,
};
use super::auth::TiingoAuth;
use super::parser::TiingoParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Tiingo connector for multi-asset data
pub struct TiingoConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: TiingoAuth,
    /// URLs
    urls: TiingoUrls,
    /// Rate limiter (5 req/min for free tier, higher for paid)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl TiingoConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `credentials` - API credentials (requires api_key as API token)
    pub async fn new(credentials: Credentials) -> ExchangeResult<Self> {
        let auth = TiingoAuth::new(&credentials)?;
        let urls = TiingoUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Initialize rate limiter: 5 req/min for free tier
        // Note: Paid tiers have higher limits (up to 1200/min)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(5, Duration::from_secs(60))
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
        endpoint: TiingoEndpoint,
        ticker: Option<&str>,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest_url();
        let url = endpoint.build_url(base_url, ticker);

        // Get auth headers
        let headers = self.auth.get_auth_header();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let full_url = format!("{}{}", url, query);

        // Make request
        let response = self.http.get(&full_url, &headers).await?;

        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for TiingoConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Tiingo
    }

    fn is_testnet(&self) -> bool {
        false // Tiingo doesn't have testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Data provider only, but we use Spot as default for compatibility
        vec![AccountType::Spot]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::DataProvider
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for TiingoConnector {
    /// Get current price (uses IEX endpoint for stocks)
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Use stock symbol format for IEX prices
        let ticker_symbol = format_stock_symbol(&symbol.base);

        let mut params = HashMap::new();
        params.insert("columns".to_string(), "close".to_string());

        let response = self.get(
            TiingoEndpoint::IexPrices,
            Some(&ticker_symbol),
            params,
        ).await?;

        // Parse IEX prices and get latest
        let klines = TiingoParser::parse_iex_prices(&response)?;
        let latest = klines.last()
            .ok_or_else(|| ExchangeError::Parse("No price data available".to_string()))?;

        Ok(latest.close)
    }

    /// Get orderbook (NOT SUPPORTED - data provider doesn't offer orderbook)
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _limit: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Tiingo does not provide orderbook data - market data provider only".to_string()
        ))
    }

    /// Get klines/candles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        // Use IEX intraday endpoint for stocks
        let ticker_symbol = format_stock_symbol(&symbol.base);
        let resample_freq = map_interval(interval);

        let mut params = HashMap::new();
        params.insert("resampleFreq".to_string(), resample_freq.to_string());

        if let Some(_lim) = limit {
            // Tiingo doesn't have a limit parameter, but we can filter after fetching
            // For now, we'll fetch recent data and limit client-side
        }

        let response = self.get(
            TiingoEndpoint::IexPrices,
            Some(&ticker_symbol),
            params,
        ).await?;

        let mut klines = TiingoParser::parse_iex_prices(&response)?;

        // Apply limit client-side if specified
        if let Some(lim) = limit {
            let start = klines.len().saturating_sub(lim as usize);
            klines = klines[start..].to_vec();
        }

        Ok(klines)
    }

    /// Get 24h ticker
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Use crypto top-of-book for ticker-like data
        // Note: For stocks, Tiingo doesn't have a direct "ticker" endpoint
        // We'll use IEX prices to construct a basic ticker

        let ticker_symbol = format_stock_symbol(&symbol.base);

        let response = self.get(
            TiingoEndpoint::IexPrices,
            Some(&ticker_symbol),
            HashMap::new(),
        ).await?;

        let klines = TiingoParser::parse_iex_prices(&response)?;

        if klines.is_empty() {
            return Err(ExchangeError::Parse("No ticker data available".to_string()));
        }

        // Construct ticker from recent klines
        let latest = klines.last().expect("Klines should not be empty");
        let high = klines.iter().map(|k| k.high).fold(f64::NEG_INFINITY, f64::max);
        let low = klines.iter().map(|k| k.low).fold(f64::INFINITY, f64::min);
        let volume: f64 = klines.iter().map(|k| k.volume).sum();

        Ok(Ticker {
            symbol: symbol.base.clone(),
            last_price: latest.close,
            bid_price: None,
            ask_price: None,
            high_24h: Some(high),
            low_24h: Some(low),
            volume_24h: Some(volume),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: latest.open_time,
        })
    }

    /// Ping (check connection)
    async fn ping(&self) -> ExchangeResult<()> {
        // Use fundamentals definitions endpoint as ping (lightweight)
        let response = self.get(
            TiingoEndpoint::FundamentalsDefinitions,
            None,
            HashMap::new(),
        ).await?;

        if response.is_array() || response.is_object() {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }

    /// Get exchange info — returns supported crypto tickers from Tiingo
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Tiingo doesn't have a bulk stock listing endpoint (requires ticker per request).
        // CryptoMeta returns all supported crypto tickers without pagination.
        let response = self.get(TiingoEndpoint::CryptoMeta, None, HashMap::new()).await?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of crypto tickers".to_string()))?;

        let infos = arr.iter().filter_map(|item| {
            let ticker = item.get("ticker")?.as_str()?.to_string();
            let base = item.get("baseCurrency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            let quote = item.get("quoteCurrency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_uppercase();

            Some(SymbolInfo {
                symbol: ticker,
                base_asset: base,
                quote_asset: quote,
                status: "TRADING".to_string(),
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
                min_notional: None,
            })
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UNSUPPORTED - Data Provider Only)
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Provider-Specific)
// ═══════════════════════════════════════════════════════════════════════════════

impl TiingoConnector {
    /// Get daily EOD prices for stocks
    pub async fn get_daily_prices(
        &self,
        ticker: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();

        if let Some(start) = start_date {
            params.insert("startDate".to_string(), start.to_string());
        }
        if let Some(end) = end_date {
            params.insert("endDate".to_string(), end.to_string());
        }

        let response = self.get(
            TiingoEndpoint::DailyPrices,
            Some(ticker),
            params,
        ).await?;

        TiingoParser::parse_daily_prices(&response)
    }

    /// Get crypto top-of-book quote
    pub async fn get_crypto_top(
        &self,
        symbol: &Symbol,
    ) -> ExchangeResult<Ticker> {
        let ticker_symbol = format_crypto_symbol(symbol);

        let mut params = HashMap::new();
        params.insert("tickers".to_string(), ticker_symbol);

        let response = self.get(
            TiingoEndpoint::CryptoTop,
            None,
            params,
        ).await?;

        TiingoParser::parse_crypto_top(&response)
    }

    /// Get crypto historical prices
    pub async fn get_crypto_prices(
        &self,
        symbol: &Symbol,
        start_date: Option<&str>,
        interval: &str,
    ) -> ExchangeResult<Vec<Kline>> {
        let ticker_symbol = format_crypto_symbol(symbol);
        let resample_freq = map_interval(interval);

        let mut params = HashMap::new();
        params.insert("tickers".to_string(), ticker_symbol);
        params.insert("resampleFreq".to_string(), resample_freq.to_string());

        if let Some(start) = start_date {
            params.insert("startDate".to_string(), start.to_string());
        }

        let response = self.get(
            TiingoEndpoint::CryptoPrices,
            None,
            params,
        ).await?;

        TiingoParser::parse_crypto_prices(&response)
    }

    /// Get forex top-of-book quote
    pub async fn get_forex_top(
        &self,
        symbol: &Symbol,
    ) -> ExchangeResult<Ticker> {
        let ticker_symbol = format_forex_symbol(symbol);

        let response = self.get(
            TiingoEndpoint::ForexTop,
            Some(&ticker_symbol),
            HashMap::new(),
        ).await?;

        TiingoParser::parse_forex_top(&response)
    }

    /// Get forex historical prices
    pub async fn get_forex_prices(
        &self,
        symbol: &Symbol,
        start_date: Option<&str>,
        interval: &str,
    ) -> ExchangeResult<Vec<Kline>> {
        let ticker_symbol = format_forex_symbol(symbol);
        let resample_freq = map_interval(interval);

        let mut params = HashMap::new();
        params.insert("resampleFreq".to_string(), resample_freq.to_string());

        if let Some(start) = start_date {
            params.insert("startDate".to_string(), start.to_string());
        }

        let response = self.get(
            TiingoEndpoint::ForexPrices,
            Some(&ticker_symbol),
            params,
        ).await?;

        TiingoParser::parse_forex_prices(&response)
    }
}
