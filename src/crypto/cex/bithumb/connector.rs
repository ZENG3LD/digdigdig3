//! # Bithumb Connector
//!
//! Реализация всех core трейтов для Bithumb Pro.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции (limited support)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::http::RetryConfig;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BithumbUrls, BithumbEndpoint, format_symbol, map_kline_interval};
use super::auth::BithumbAuth;
use super::parser::BithumbParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bithumb коннектор
pub struct BithumbConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BithumbAuth>,
    /// URL'ы (mainnet/testnet)
    urls: BithumbUrls,
    /// Testnet mode (note: Bithumb Pro doesn't have testnet)
    testnet: bool,
    /// Rate limiter для всех запросов (2 req/s - очень консервативно из-за проблем с инфраструктурой)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BithumbConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            BithumbUrls::TESTNET
        } else {
            BithumbUrls::MAINNET
        };

        // Bithumb API имеет известные проблемы с инфраструктурой (~20% запросов получают 504 Gateway Timeout)
        // Используем специальную конфигурацию retry с:
        // - 7 попыток (вместо 3)
        // - Более короткий таймаут (10s вместо 30s) с более быстрым exponential backoff
        // - Jitter для избежания thundering herd
        let retry_config = RetryConfig::unreliable_api();
        let http = HttpClient::with_config(10_000, retry_config)?; // 10 sec timeout

        let auth = credentials
            .as_ref()
            .map(BithumbAuth::new)
            .transpose()?;

        // Bithumb has poor documentation and flaky infrastructure
        // Use VERY conservative rate limit: 120 requests per 60 seconds
        // This prevents overwhelming their servers and triggering 504 errors
        // Slower requests = fewer retries = faster overall
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(120, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self) {
        let wait_time = {
            let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
            if !limiter.try_acquire() {
                limiter.time_until_ready()
            } else {
                Duration::ZERO
            }
        };

        if !wait_time.is_zero() {
            tokio::time::sleep(wait_time).await;
            // Try again after waiting
            let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
            limiter.try_acquire();
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: BithumbEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Apply rate limiting BEFORE making the request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth params if needed
        if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            params = auth.sign_request(&mut params);
        }

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

        let response = self.http.get(&url, &HashMap::new()).await?;
        BithumbParser::check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BithumbEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Apply rate limiting BEFORE making the request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth params
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let signed_params = auth.sign_request(&mut params);

        // Convert to JSON
        let body = json!(signed_params);

        let response = self.http.post(&url, &body, &HashMap::new()).await?;
        BithumbParser::check_response(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Bithumb-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить информацию о символах
    pub async fn get_config(&self) -> ExchangeResult<Value> {
        self.get(BithumbEndpoint::SpotConfig, HashMap::new(), AccountType::Spot).await
    }

    /// Получить server time
    pub async fn get_server_time(&self) -> ExchangeResult<i64> {
        let response = self.get(BithumbEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        let data = BithumbParser::extract_data(&response)?;
        data.as_i64()
            .ok_or_else(|| ExchangeError::Parse("Invalid server time".to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BithumbConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bithumb
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::FuturesCross,
            // Bithumb has separate platforms:
            // - Bithumb Pro: spot trading
            // - Bithumb Futures: perpetual futures
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
impl MarketData for BithumbConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => BithumbEndpoint::FuturesTicker,
            _ => BithumbEndpoint::SpotTicker,
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => BithumbEndpoint::FuturesOrderbook,
            _ => BithumbEndpoint::SpotOrderbook,
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                // Bithumb Futures uses "interval" parameter
                params.insert("interval".to_string(), map_kline_interval(interval, account_type));
                BithumbEndpoint::FuturesKlines
            }
            _ => {
                // Bithumb Pro Spot uses "type" parameter
                params.insert("type".to_string(), map_kline_interval(interval, account_type));

                // Bithumb Pro requires start and end timestamps
                // Use last 24 hours as default
                let end = crate::core::timestamp_millis() / 1000; // seconds
                let start = end - 86400; // 24 hours ago
                params.insert("start".to_string(), start.to_string());
                params.insert("end".to_string(), end.to_string());

                BithumbEndpoint::SpotKlines
            }
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => BithumbEndpoint::FuturesTicker,
            _ => BithumbEndpoint::SpotTicker,
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _ = self.get_server_time().await?;
        Ok(())
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


