//! # Bitget Connector
//!
//! Реализация всех core трейтов для Bitget.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{
    BitgetUrls, BitgetEndpoint, format_symbol, map_kline_interval,
    map_futures_granularity, get_product_type
};
use super::auth::BitgetAuth;
use super::parser::BitgetParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitget коннектор
pub struct BitgetConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BitgetAuth>,
    /// URL'ы (mainnet only for Bitget)
    urls: BitgetUrls,
    /// Rate limiter для market data (20 req/sec)
    market_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Rate limiter для trading (10 req/sec)
    trading_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BitgetConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, _testnet: bool) -> ExchangeResult<Self> {
        let urls = BitgetUrls::MAINNET;

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials
            .as_ref()
            .map(BitgetAuth::new)
            .transpose()?;

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = urls.rest_url(AccountType::Spot);
            let url = format!("{}/api/v2/public/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(data) = response.get("data") {
                    if let Some(server_time_str) = data.get("serverTime").and_then(|t| t.as_str()) {
                        if let Ok(server_time) = server_time_str.parse::<i64>() {
                            if let Some(ref mut a) = auth {
                                a.sync_time(server_time);
                            }
                        }
                    }
                }
            }
        }

        // Bitget rate limits: market 20/s, trading 10/s
        let market_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(20, Duration::from_secs(1))
        ));
        let trading_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(10, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            market_limiter,
            trading_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None, false).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse rate limit headers from Bitget response and update the appropriate limiter.
    ///
    /// Bitget reports: `x-mbx-used-remain-limit` = remaining requests in the current second.
    fn update_rate_from_headers(&self, headers: &HeaderMap, is_private: bool) {
        if let Some(remaining) = headers
            .get("x-mbx-used-remain-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            let limiter = if is_private { &self.trading_limiter } else { &self.market_limiter };
            if let Ok(mut lim) = limiter.lock() {
                lim.update_from_server(remaining);
            }
        }
    }

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self, is_private: bool) {
        let limiter = if is_private { &self.trading_limiter } else { &self.market_limiter };
        loop {
            let wait_time = {
                let mut l = limiter.lock().expect("lock");
                if l.try_acquire() {
                    return;
                }
                l.time_until_ready()
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: BitgetEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit based on endpoint type
        self.rate_limit_wait(endpoint.requires_auth()).await;

        let base_url = self.urls.rest_url(account_type);
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
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", path, &query, "")
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, endpoint.requires_auth());
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BitgetEndpoint,
        body: Value,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // POST endpoints are always trading-related
        self.rate_limit_wait(true).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", path, "", &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers, true);
        self.check_response(&response)?;
        Ok(response)
    }

    /// Проверить response на ошибки
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("00000");

        if code != "00000" {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code.parse().unwrap_or(-1),
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Bitget-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить информацию о символах
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Value> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotSymbols,
            _ => BitgetEndpoint::FuturesContracts,
        };

        let mut params = HashMap::new();

        // Futures requires productType parameter
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), "USDT-FUTURES".to_string());
        }

        self.get(endpoint, params, account_type).await
    }

}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitgetConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitget
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.market_limiter.lock() {
            (lim.current_count(), lim.max_requests())
        } else {
            (0, 0)
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups: Vec::new(),
            ws_ping_rtt_ms: 0,
        }
    }

    fn is_testnet(&self) -> bool {
        false // Bitget doesn't have testnet in this implementation
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
            AccountType::FuturesIsolated,
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BitgetConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotPrice,
            _ => BitgetEndpoint::FuturesPrice,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Futures requires productType
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotOrderbook,
            _ => BitgetEndpoint::FuturesOrderbook,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Bitget spot uses "type" and "limit", futures uses just "limit"
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                params.insert("type".to_string(), "step0".to_string());
                params.insert("limit".to_string(), depth.unwrap_or(100).to_string());
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
                let limit = match depth.unwrap_or(100) {
                    0..=5 => 5,
                    6..=15 => 15,
                    16..=50 => 50,
                    _ => 100,
                };
                params.insert("limit".to_string(), limit.to_string());
            }
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotKlines,
            _ => BitgetEndpoint::FuturesKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // V2 API uses `granularity` for both Spot and Futures
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                // V2 Spot uses "granularity" with format: "1min", "1h", "1day"
                params.insert("granularity".to_string(), map_kline_interval(interval).to_string());
                params.insert("limit".to_string(), limit.unwrap_or(1000).min(1000).to_string());
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
                // V2 Futures uses "granularity" with format: "1m", "1H", "1D"
                params.insert("granularity".to_string(), map_futures_granularity(interval).to_string());
                params.insert("limit".to_string(), limit.unwrap_or(200).min(1000).to_string());
            }
        }

        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotTicker,
            _ => BitgetEndpoint::FuturesTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Futures requires productType
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(BitgetEndpoint::Timestamp, HashMap::new(), AccountType::Spot).await?;
        self.check_response(&response)
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols(account_type).await?;
        BitgetParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════


