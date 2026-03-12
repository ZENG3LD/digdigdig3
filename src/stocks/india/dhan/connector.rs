//! # Dhan Connector
//!
//! Implementation of all core traits for Dhan.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - position management (for F&O)
//!
//! ## Extended Methods
//! Additional Dhan-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

use async_trait::async_trait;
use serde_json::{json, Value};
use reqwest;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderStatus, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::SymbolInfo;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{DhanUrls, DhanEndpoint, DhanExchangeSegment, map_interval, map_product_type};
use super::auth::DhanAuth;
use super::parser::DhanParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Dhan connector
pub struct DhanConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: Arc<Mutex<DhanAuth>>,
    /// URLs (mainnet/testnet)
    urls: DhanUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter for order APIs (25/sec)
    order_limiter: Arc<StdMutex<SimpleRateLimiter>>,
    /// Rate limiter for data APIs (5/sec)
    data_limiter: Arc<StdMutex<SimpleRateLimiter>>,
    /// Rate limiter for quote APIs (1/sec)
    quote_limiter: Arc<StdMutex<SimpleRateLimiter>>,
    /// Rate limiter for non-trading APIs (20/sec)
    general_limiter: Arc<StdMutex<SimpleRateLimiter>>,
}

impl DhanConnector {
    /// Create new connector
    pub async fn new(credentials: Credentials, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            DhanUrls::TESTNET
        } else {
            DhanUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = Arc::new(Mutex::new(DhanAuth::new(&credentials)?));

        // Initialize rate limiters per Dhan's documented limits
        let order_limiter = Arc::new(StdMutex::new(
            SimpleRateLimiter::new(25, Duration::from_secs(1)) // 25 orders/sec
        ));
        let data_limiter = Arc::new(StdMutex::new(
            SimpleRateLimiter::new(5, Duration::from_secs(1)) // 5 data requests/sec
        ));
        let quote_limiter = Arc::new(StdMutex::new(
            SimpleRateLimiter::new(1, Duration::from_secs(1)) // 1 quote request/sec
        ));
        let general_limiter = Arc::new(StdMutex::new(
            SimpleRateLimiter::new(20, Duration::from_secs(1)) // 20 general requests/sec
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            order_limiter,
            data_limiter,
            quote_limiter,
            general_limiter,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit based on endpoint type
    async fn rate_limit_wait(&self, endpoint: DhanEndpoint) {
        let limiter = match endpoint {
            // Order APIs
            DhanEndpoint::PlaceOrder
            | DhanEndpoint::ModifyOrder
            | DhanEndpoint::CancelOrder
            | DhanEndpoint::PlaceSlicedOrder
            | DhanEndpoint::PlaceSuperOrder
            | DhanEndpoint::ModifySuperOrder
            | DhanEndpoint::CancelSuperOrder
            | DhanEndpoint::PlaceForeverOrder
            | DhanEndpoint::ModifyForeverOrder
            | DhanEndpoint::CancelForeverOrder => &self.order_limiter,

            // Data APIs
            DhanEndpoint::HistoricalDaily | DhanEndpoint::HistoricalIntraday => &self.data_limiter,

            // Quote APIs
            DhanEndpoint::LTP | DhanEndpoint::OHLC | DhanEndpoint::Quote | DhanEndpoint::OptionChain => &self.quote_limiter,

            // Everything else
            _ => &self.general_limiter,
        };

        loop {
            let wait_time = {
                let mut rate_limiter = limiter.lock().expect("Mutex poisoned");
                if rate_limiter.try_acquire() {
                    return;
                }
                rate_limiter.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(&self, endpoint: DhanEndpoint, params: HashMap<String, String>) -> ExchangeResult<Value> {
        self.rate_limit_wait(endpoint).await;

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

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let base_url_owned = base_url.to_string();
            let mut auth = self.auth.lock().await;
            auth.build_headers(&base_url_owned, &self.http).await?
        } else {
            let auth = self.auth.lock().await;
            auth.build_public_headers()
        };

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// POST request
    async fn post(&self, endpoint: DhanEndpoint, body: Value) -> ExchangeResult<Value> {
        self.rate_limit_wait(endpoint).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let headers = if endpoint.requires_auth() {
            let base_url_owned = base_url.to_string();
            let mut auth = self.auth.lock().await;
            auth.build_headers(&base_url_owned, &self.http).await?
        } else {
            let auth = self.auth.lock().await;
            auth.build_public_headers()
        };

        let response = self.http.post(&url, &body, &headers).await?;
        Ok(response)
    }

    /// PUT request
    async fn _put(&self, endpoint: DhanEndpoint, path_params: &[(&str, &str)], body: Value) -> ExchangeResult<Value> {
        self.rate_limit_wait(endpoint).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        let base_url_owned = base_url.to_string();
        let mut auth = self.auth.lock().await;
        let headers = auth.build_headers(&base_url_owned, &self.http).await?;
        drop(auth); // Explicitly drop to release lock

        let response = self.http.put(&url, &body, &headers).await?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(&self, endpoint: DhanEndpoint, path_params: &[(&str, &str)]) -> ExchangeResult<Value> {
        self.rate_limit_wait(endpoint).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        let base_url_owned = base_url.to_string();
        let mut auth = self.auth.lock().await;
        let headers = auth.build_headers(&base_url_owned, &self.http).await?;
        drop(auth); // Explicitly drop to release lock

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get security ID from symbol (assumes symbol is already the security ID)
    fn get_security_id(&self, symbol: &Symbol) -> String {
        // In production, you'd look this up from instrument CSV
        // For now, assume symbol contains the security ID
        symbol.base.clone()
    }

    /// Get exchange segment from account type
    fn get_exchange_segment(&self, _account_type: AccountType) -> DhanExchangeSegment {
        // Default to NSE Equity
        // In production, this should be configurable or derived from symbol
        DhanExchangeSegment::NseEq
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ExchangeIdentity for DhanConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Dhan
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }
}

#[async_trait]
impl MarketData for DhanConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        let security_id = self.get_security_id(&symbol);
        let segment = self.get_exchange_segment(_account_type);

        let body = json!({
            segment.as_str(): [security_id.clone()]
        });

        let response = self.post(DhanEndpoint::LTP, body).await?;
        DhanParser::parse_ltp(&response, &security_id)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let security_id = self.get_security_id(&symbol);
        let segment = self.get_exchange_segment(_account_type);

        let body = json!({
            segment.as_str(): [security_id.clone()]
        });

        let response = self.post(DhanEndpoint::Quote, body).await?;
        DhanParser::parse_quote(&response, &security_id)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let security_id = self.get_security_id(&symbol);
        let segment = self.get_exchange_segment(_account_type);

        // Calculate date range (default to last 90 days for intraday)
        let to_date = chrono::Utc::now();
        let from_date = to_date - chrono::Duration::days(90);

        let body = json!({
            "securityId": security_id,
            "exchangeSegment": segment.as_str(),
            "instrument": "EQUITY",
            "interval": map_interval(interval),
            "fromDate": from_date.format("%Y-%m-%d").to_string(),
            "toDate": to_date.format("%Y-%m-%d").to_string(),
        });

        let response = self.post(DhanEndpoint::HistoricalIntraday, body).await?;
        let mut klines = DhanParser::parse_historical_intraday(&response)?;

        // Apply limit if specified
        if let Some(limit) = limit {
            klines.truncate(limit as usize);
        }

        Ok(klines)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let security_id = self.get_security_id(&symbol);
        let segment = self.get_exchange_segment(_account_type);

        let body = json!({
            segment.as_str(): [security_id.clone()]
        });

        let response = self.post(DhanEndpoint::OHLC, body).await?;
        DhanParser::parse_ticker(&response, &security_id)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Dhan doesn't have a dedicated ping endpoint
        // Use token generation as health check
        let base_url = self.urls.rest_url().to_string();
        let mut auth = self.auth.lock().await;
        let result = auth.get_token(&base_url, &self.http).await?;
        drop(auth);
        drop(result);
        Ok(())
    }

    /// Get exchange info — returns NSE equity instruments from Dhan
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Dhan's InstrumentList endpoint: /v2/instrument/{exchangeSegment}
        // Returns CSV with columns: SEM_EXM_EXCH_ID, SEM_SEGMENT, SEM_SMST_SECURITY_ID, SEM_INSTRUMENT_NAME, SEM_CUSTOM_SYMBOL, ...
        let base_url = self.urls.rest_url();
        let url = format!("{}/v2/instrument/NSE_EQ", base_url);

        let base_url_owned = base_url.to_string();
        let mut auth = self.auth.lock().await;
        let headers = auth.build_headers(&base_url_owned, &self.http).await?;
        drop(auth);

        // Use reqwest directly for text response with auth headers
        let client = reqwest::Client::new();
        let mut req = client.get(&url);
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let response = req.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let csv_text = response.text().await
            .map_err(|e| ExchangeError::Network(format!("Failed to read text: {}", e)))?;

        let mut infos = Vec::new();
        for (i, line) in csv_text.lines().enumerate() {
            if i == 0 {
                continue; // skip header
            }
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 5 {
                continue;
            }

            let symbol = cols[4].trim().trim_matches('"').to_string();
            if symbol.is_empty() {
                continue;
            }

            infos.push(SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: "INR".to_string(),
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: Some(1.0),
                max_quantity: None,
                step_size: Some(1.0),
                min_notional: None,
            });
        }

        Ok(infos)
    }
}







#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("1000000123");

        let result = DhanConnector::new(credentials, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_exchange_identity() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("1000000123");

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let connector = runtime.block_on(DhanConnector::new(credentials, true)).unwrap();

        assert_eq!(connector.exchange_id(), ExchangeId::Dhan);
        assert!(connector.is_testnet());
    }
}
