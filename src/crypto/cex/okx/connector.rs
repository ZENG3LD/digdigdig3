//! # OKX Connector
//!
//! Реализация всех core трейтов для OKX API v5.
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
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{OkxUrls, OkxEndpoint, format_symbol, map_kline_interval, get_inst_type, get_trade_mode};
use super::auth::OkxAuth;
use super::parser::OkxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OKX коннектор
pub struct OkxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<OkxAuth>,
    /// URL'ы (mainnet/testnet)
    urls: OkxUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (10 requests per 2 seconds = 5 rps)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl OkxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            OkxUrls::TESTNET
        } else {
            OkxUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(OkxAuth::new)
            .transpose()?;

        // Initialize rate limiter: 20 requests per 2 seconds (OKX public endpoint limit)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(20, Duration::from_secs(2))
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

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock()
                    .expect("Rate limiter mutex poisoned");
                if limiter.try_acquire() {
                    return;
                }
                limiter.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: OkxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

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
        let full_path = format!("{}{}", path, query);

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            if self.testnet {
                auth.sign_request_testnet("GET", &full_path, "")
            } else {
                auth.sign_request("GET", &full_path, "")
            }
        } else {
            HashMap::new()
        };

        self.http.get_with_headers(&url, &HashMap::new(), &headers).await
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: OkxEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = if self.testnet {
            auth.sign_request_testnet("POST", path, &body_str)
        } else {
            auth.sign_request("POST", path, &body_str)
        };

        self.http.post(&url, &body, &headers).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (OKX-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить все тикеры для определенного типа инструментов
    pub async fn get_all_tickers(&self, account_type: AccountType) -> ExchangeResult<Vec<Ticker>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        let response = self.get(OkxEndpoint::AllTickers, params).await?;
        // TODO: implement parse_all_tickers in parser
        let _ = response;
        Ok(vec![])
    }

    /// Получить информацию о инструментах/символах
    pub async fn get_instruments(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        let response = self.get(OkxEndpoint::Instruments, params).await?;
        OkxParser::parse_symbols(&response)
    }

    /// Получить список символов (алиас для get_instruments для совместимости с тестами)
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        self.get_instruments(account_type).await
    }

    /// Получить server time
    pub async fn get_server_time(&self) -> ExchangeResult<i64> {
        let response = self.get(OkxEndpoint::ServerTime, HashMap::new()).await?;
        let data = OkxParser::extract_first_data(&response)?;
        OkxParser::parse_i64(data.get("ts").ok_or_else(|| ExchangeError::Parse("Missing 'ts'".to_string()))?)
            .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for OkxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::OKX
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
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
        self.testnet
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
impl MarketData for OkxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::Ticker, params).await?;
        let ticker = OkxParser::parse_ticker(&response)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("sz".to_string(), d.to_string());
        }

        let response = self.get(OkxEndpoint::Orderbook, params).await?;
        OkxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("bar".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(300).to_string());
        }

        // OKX naming is inverted: "after" = older-than (paginate backward).
        // /market/candles has ~1440 bar depth limit on 1m.
        // /market/history-candles has full depth — use it for pagination.
        let endpoint = if end_time.is_some() {
            OkxEndpoint::HistoryKlines
        } else {
            OkxEndpoint::Klines
        };

        if let Some(et) = end_time {
            params.insert("after".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params).await?;
        OkxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::Ticker, params).await?;
        OkxParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        self.get(OkxEndpoint::ServerTime, HashMap::new()).await?;
        Ok(())
    }

    /// Получить информацию о всех торговых символах биржи
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        self.get_instruments(account_type).await
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


