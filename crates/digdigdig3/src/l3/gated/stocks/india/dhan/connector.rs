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

use serde_json::{json, Value};
use reqwest;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, OrderStatus, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    AmendRequest,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions, AmendOrder,
};
use crate::core::types::{SymbolInfo, SymbolInput};
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
    async fn put(&self, endpoint: DhanEndpoint, path_params: &[(&str, &str)], body: Value) -> ExchangeResult<Value> {
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
    fn get_security_id(&self, symbol: &str) -> String {
        // In production, you'd look this up from instrument CSV
        // For now, assume symbol contains the security ID
        symbol.to_string()
    }

    /// Get exchange segment from account type
    fn get_exchange_segment(&self, _account_type: AccountType) -> DhanExchangeSegment {
        // Default to NSE Equity
        // In production, this should be configurable or derived from symbol
        DhanExchangeSegment::NseEq
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Options Chain
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get options chain — `GET /v2/optionchain`
    ///
    /// `underlying_scrip_id` — security ID of the underlying instrument.
    /// `expiry_date` — option expiry date in `YYYY-MM-DD` format.
    pub async fn get_options_chain(
        &self,
        underlying_scrip_id: &str,
        expiry_date: &str,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("UnderlyingScripId".to_string(), underlying_scrip_id.to_string());
        params.insert("ExpiryDate".to_string(), expiry_date.to_string());
        self.get(DhanEndpoint::OptionChain, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Kill Switch
    // ═══════════════════════════════════════════════════════════════════════════

    /// Activate or deactivate kill switch — `POST /v2/killswitch`
    ///
    /// `kill_switch_status` — `"ACTIVATE"` to halt all trading, `"DEACTIVATE"` to resume.
    pub async fn kill_switch(&self, kill_switch_status: &str) -> ExchangeResult<Value> {
        let body = json!({
            "killSwitchStatus": kill_switch_status,
        });
        self.post(DhanEndpoint::KillSwitch, body).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
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

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl MarketData for DhanConnector {
    async fn get_price(&self, symbol: SymbolInput<'_>, _account_type: AccountType) -> ExchangeResult<Price> {
        let sym_str: String = match symbol { SymbolInput::Raw(s) => s.to_string(), SymbolInput::Canonical(c) => c.to_concat() };
        let security_id = self.get_security_id(&sym_str);
        let segment = self.get_exchange_segment(_account_type);

        let body = json!({
            segment.as_str(): [security_id.clone()]
        });

        let response = self.post(DhanEndpoint::LTP, body).await?;
        DhanParser::parse_ltp(&response, &security_id)
    }

    async fn get_orderbook(
        &self,
        symbol: SymbolInput<'_>,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let sym_str: String = match symbol { SymbolInput::Raw(s) => s.to_string(), SymbolInput::Canonical(c) => c.to_concat() };
        let security_id = self.get_security_id(&sym_str);
        let segment = self.get_exchange_segment(_account_type);

        let body = json!({
            segment.as_str(): [security_id.clone()]
        });

        let response = self.post(DhanEndpoint::Quote, body).await?;
        DhanParser::parse_quote(&response, &security_id)
    }

    async fn get_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let sym_str: String = match symbol { SymbolInput::Raw(s) => s.to_string(), SymbolInput::Canonical(c) => c.to_concat() };
        let security_id = self.get_security_id(&sym_str);
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

    async fn get_ticker(&self, symbol: SymbolInput<'_>, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let sym_str: String = match symbol { SymbolInput::Raw(s) => s.to_string(), SymbolInput::Canonical(c) => c.to_concat() };
        let security_id = self.get_security_id(&sym_str);
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

    /// Get exchange info — full Dhan instrument master (all segments, no filter).
    ///
    /// Uses the public Dhan scrip-master CSV which covers all exchange segments:
    /// NSE_EQ, NSE_FNO, NSE_CURRENCY, BSE_EQ, BSE_FNO, MCX_COMM, etc.
    /// No active-only filter — every row is returned verbatim.
    ///
    /// CSV columns (0-indexed):
    ///   0 SEM_EXM_EXCH_ID  (exchange, e.g. "NSE"/"BSE"/"MCX")
    ///   1 SEM_SEGMENT       (segment token, e.g. "NSE_EQ"/"NSE_FNO" → instrument_type)
    ///   2 SEM_SMST_SECURITY_ID  (numeric security ID)
    ///   3 SEM_INSTRUMENT_NAME   (instrument class, e.g. "EQUITY","FUTIDX","OPTIDX")
    ///   4 SEM_CUSTOM_SYMBOL     (display symbol used in order API)
    ///   5 SEM_EXPIRY_FLAG
    ///   6 SM_SYMBOL_NAME
    ///   7 SEM_SERIES
    ///   8 SEM_STRIKE_PRICE
    ///   9 SEM_OPTION_TYPE
    ///  10 SEM_FUT_FLAG
    ///  11 SEM_LOT_UNITS
    ///  12 SEM_CUSTOM_SYMBOL (alias / display form)
    ///  13 SEM_EXPIRY_DATE
    ///  14 SEM_TICK_SIZE
    ///  15 SEM_TRADING_SYMBOL
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Public master CSV — no auth required, all segments in one file.
        let url = "https://images.dhan.co/api-data/api-scrip-master.csv";

        let csv_text = reqwest::Client::new()
            .get(url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Dhan scrip-master fetch failed: {}", e)))?
            .text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Dhan scrip-master read failed: {}", e)))?;

        // Parse header row to build a name→index map for robust column access.
        let mut lines = csv_text.lines();
        let header_line = match lines.next() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let headers: Vec<&str> = header_line.split(',')
            .map(|s| s.trim().trim_matches('"'))
            .collect();

        let col = |name: &str| -> Option<usize> {
            headers.iter().position(|&h| h.eq_ignore_ascii_case(name))
        };

        // Column indices — fall back to positional defaults when CSV evolves.
        let i_exch   = col("SEM_EXM_EXCH_ID").unwrap_or(0);
        let i_seg    = col("SEM_SEGMENT").unwrap_or(1);
        let i_sec_id = col("SEM_SMST_SECURITY_ID").unwrap_or(2);
        let i_inst   = col("SEM_INSTRUMENT_NAME").unwrap_or(3);
        let i_sym    = col("SEM_CUSTOM_SYMBOL").unwrap_or(4);
        let i_lot    = col("SEM_LOT_UNITS").unwrap_or(11);
        let i_tick   = col("SEM_TICK_SIZE").unwrap_or(14);

        let mut infos = Vec::new();
        for line in lines {
            let cols: Vec<&str> = line.split(',').collect();
            let ncols = cols.len();
            if ncols < 5 {
                continue;
            }

            let get = |i: usize| -> &str {
                if i < ncols { cols[i].trim().trim_matches('"') } else { "" }
            };

            let symbol = get(i_sym);
            if symbol.is_empty() {
                continue;
            }

            // Native status: Dhan scrip-master has no status column.
            let status = String::new();

            // instrument_type: segment token verbatim (e.g. "NSE_EQ", "NSE_FNO",
            // "BSE_EQ", "MCX_COMM").  If SEM_INSTRUMENT_NAME is non-empty and
            // distinct, combine: "NSE_FNO/FUTIDX".
            let seg  = get(i_seg);
            let inst = get(i_inst);
            let instrument_type = if inst.is_empty() || inst == seg {
                if seg.is_empty() { None } else { Some(seg.to_string()) }
            } else {
                Some(format!("{}/{}", seg, inst))
            };

            let tick_size = get(i_tick).parse::<f64>().ok().filter(|&v| v > 0.0);
            let lot_size  = get(i_lot).parse::<f64>().ok().filter(|&v| v > 0.0);

            // Build the raw JSON object from all columns keyed by header name.
            let mut obj = serde_json::Map::with_capacity(ncols);
            for (idx, &hdr) in headers.iter().enumerate() {
                let val = if idx < ncols {
                    serde_json::Value::String(cols[idx].trim().trim_matches('"').to_string())
                } else {
                    serde_json::Value::Null
                };
                obj.insert(hdr.to_string(), val);
            }
            let extra = serde_json::Value::Object(obj);

            infos.push(SymbolInfo {
                symbol: symbol.to_string(),
                base_asset: get(i_exch).to_string(),
                quote_asset: "INR".to_string(),
                status,
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: lot_size.or(Some(1.0)),
                max_quantity: None,
                tick_size,
                step_size: lot_size.or(Some(1.0)),
                min_notional: None,
                account_type,
                instrument_type,
                extra,
            });

            // Use `i_sec_id` as the security ID in `base_asset` instead of exchange.
            // Patch the already-pushed entry.
            let last = infos.last_mut().unwrap();
            let sec_id = get(i_sec_id).to_string();
            if !sec_id.is_empty() {
                last.base_asset = sec_id;
            }
        }

        Ok(infos)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Trading for DhanConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let transaction_type = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        let security_id = self.get_security_id(&symbol.base);
        let segment = self.get_exchange_segment(account_type);
        let product_type = map_product_type(account_type);
        let client_id = {
            let auth = self.auth.lock().await;
            auth.client_id().to_string()
        };

        // Determine validity from time_in_force
        let validity = match req.time_in_force {
            crate::core::TimeInForce::Ioc => "IOC",
            _ => "DAY",
        };

        match req.order_type {
            OrderType::Market => {
                let body = json!({
                    "dhanClientId": client_id,
                    "transactionType": transaction_type,
                    "exchangeSegment": segment.as_str(),
                    "productType": product_type,
                    "orderType": "MARKET",
                    "validity": validity,
                    "securityId": security_id,
                    "quantity": quantity as i64,
                    "price": 0,
                    "disclosedQuantity": 0,
                    "afterMarketOrder": false,
                });
                let response = self.post(DhanEndpoint::PlaceOrder, body).await?;
                DhanParser::parse_order_placement(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let body = json!({
                    "dhanClientId": client_id,
                    "transactionType": transaction_type,
                    "exchangeSegment": segment.as_str(),
                    "productType": product_type,
                    "orderType": "LIMIT",
                    "validity": validity,
                    "securityId": security_id,
                    "quantity": quantity as i64,
                    "price": price,
                    "disclosedQuantity": 0,
                    "afterMarketOrder": false,
                });
                let response = self.post(DhanEndpoint::PlaceOrder, body).await?;
                DhanParser::parse_order_placement(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                // Dhan STOP_LOSS_MARKET: triggerPrice required, price = 0
                let body = json!({
                    "dhanClientId": client_id,
                    "transactionType": transaction_type,
                    "exchangeSegment": segment.as_str(),
                    "productType": product_type,
                    "orderType": "STOP_LOSS_MARKET",
                    "validity": validity,
                    "securityId": security_id,
                    "quantity": quantity as i64,
                    "price": 0,
                    "triggerPrice": stop_price,
                    "disclosedQuantity": 0,
                    "afterMarketOrder": false,
                });
                let response = self.post(DhanEndpoint::PlaceOrder, body).await?;
                DhanParser::parse_order_placement(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Dhan STOP_LOSS: both price and triggerPrice required
                let body = json!({
                    "dhanClientId": client_id,
                    "transactionType": transaction_type,
                    "exchangeSegment": segment.as_str(),
                    "productType": product_type,
                    "orderType": "STOP_LOSS",
                    "validity": validity,
                    "securityId": security_id,
                    "quantity": quantity as i64,
                    "price": limit_price,
                    "triggerPrice": stop_price,
                    "disclosedQuantity": 0,
                    "afterMarketOrder": false,
                });
                let response = self.post(DhanEndpoint::PlaceOrder, body).await?;
                DhanParser::parse_order_placement(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                // Dhan Super Order (Bracket) — POST /v2/orders/super
                // Native bracket: legName, price, targetPrice, stopLossPrice, trailingJump
                let entry_price = price.unwrap_or(0.0);
                let order_type_str = if entry_price > 0.0 { "LIMIT" } else { "MARKET" };

                let body = json!({
                    "dhanClientId": client_id,
                    "transactionType": transaction_type,
                    "exchangeSegment": segment.as_str(),
                    "productType": "BO",
                    "orderType": order_type_str,
                    "validity": "DAY",
                    "securityId": security_id,
                    "quantity": quantity as i64,
                    "price": entry_price,
                    "disclosedQuantity": 0,
                    "afterMarketOrder": false,
                    "boProfitValue": take_profit,
                    "boStopLossValue": stop_loss,
                });
                let response = self.post(DhanEndpoint::PlaceSuperOrder, body).await?;
                DhanParser::parse_order_placement(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                // IOC: validity = IOC, LIMIT if price given, MARKET otherwise
                let (order_type_str, price_val) = match price {
                    Some(p) => ("LIMIT", p),
                    None => ("MARKET", 0.0),
                };
                let body = json!({
                    "dhanClientId": client_id,
                    "transactionType": transaction_type,
                    "exchangeSegment": segment.as_str(),
                    "productType": product_type,
                    "orderType": order_type_str,
                    "validity": "IOC",
                    "securityId": security_id,
                    "quantity": quantity as i64,
                    "price": price_val,
                    "disclosedQuantity": 0,
                    "afterMarketOrder": false,
                });
                let response = self.post(DhanEndpoint::PlaceOrder, body).await?;
                DhanParser::parse_order_placement(&response).map(PlaceOrderResponse::Simple)
            }

            _ => Err(ExchangeError::NotImplemented(
                format!("{:?} order type not supported on Dhan", req.order_type)
            )),
        }
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Dhan /v2/orders returns all orders (open + closed)
        let response = self.get(DhanEndpoint::GetOrderBook, HashMap::new()).await?;
        let all_orders = DhanParser::parse_orders(&response)?;

        // Filter for closed/filled/cancelled orders (history)
        Ok(all_orders
            .into_iter()
            .filter(|o| matches!(
                o.status,
                OrderStatus::Filled | OrderStatus::Canceled | OrderStatus::Rejected | OrderStatus::Expired
            ))
            .collect())
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let response = self.delete(DhanEndpoint::CancelOrder, &[("orderId", order_id)]).await?;
            DhanParser::parse_order_placement(&response)
    
            }
            _ => Err(ExchangeError::NotImplemented(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let mut path = DhanEndpoint::GetOrder.path().to_string();
        path = path.replace("{orderId}", order_id);

        let base_url = self.urls.rest_url();
        let url = format!("{}{}", base_url, path);

        let base_url_owned = base_url.to_string();
        let mut auth = self.auth.lock().await;
        let headers = auth.build_headers(&base_url_owned, &self.http).await?;
        drop(auth); // Explicitly drop to release lock

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        DhanParser::parse_order(&response)
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let _symbol_str = _symbol;
        let _symbol: Option<crate::core::Symbol> = _symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let response = self.get(DhanEndpoint::GetOrderBook, HashMap::new()).await?;
        let all_orders = DhanParser::parse_orders(&response)?;

        // Filter for open orders only
        Ok(all_orders
            .into_iter()
            .filter(|o| matches!(o.status, OrderStatus::New | OrderStatus::Open | OrderStatus::PartiallyFilled))
            .collect())
    
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Account for DhanConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset;
        let _account_type = query.account_type;
        // /v2/fundlimit returns available cash/margin balance
        let response = self.get(DhanEndpoint::GetFunds, HashMap::new()).await?;
        DhanParser::parse_balance(&response)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Dhan has no profile endpoint; return info derived from fund limit
        let response = self.get(DhanEndpoint::GetFunds, HashMap::new()).await?;
        DhanParser::parse_funds(&response)
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Dhan does not expose a fee schedule API
        Err(ExchangeError::NotImplemented(
            "Fee schedule endpoint not available on Dhan".to_string()
        ))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Positions for DhanConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let response = self.get(DhanEndpoint::GetPositions, HashMap::new()).await?;
        DhanParser::parse_positions(&response)
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let _symbol_str = _symbol;
        let _symbol = {
            let parts: Vec<&str> = _symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: _symbol_str.to_string(), quote: String::new(), raw: Some(_symbol_str.to_string()) }
            }
        };

        // Dhan doesn't have funding rates (equity derivatives don't have funding)
        Err(ExchangeError::NotImplemented(
            "Funding rates not available for equity derivatives".to_string(),
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Dhan uses fixed margin requirements, leverage not directly settable
                Err(ExchangeError::NotImplemented(
                "Leverage setting not supported (uses fixed margin requirements)".to_string(),
                ))
    
            }
            _ => Err(ExchangeError::NotImplemented(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait — Dhan supports PUT /v2/orders/{orderId})
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl AmendOrder for DhanConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let order_id = req.order_id.clone();

        // At least one field must be Some
        if req.fields.price.is_none()
            && req.fields.quantity.is_none()
            && req.fields.trigger_price.is_none()
        {
            return Err(ExchangeError::InvalidRequest(
                "At least one field (price, quantity, trigger_price) must be provided".to_string(),
            ));
        }

        // Fetch current order to fill in unchanged fields (Dhan PUT requires all fields)
        let current = self.get_order("", &order_id, req.account_type).await?;

        let new_price = req.fields.price.or(current.price).unwrap_or(0.0);
        let new_quantity = req.fields.quantity.unwrap_or(current.quantity);
        let new_trigger = req.fields.trigger_price.or(current.stop_price).unwrap_or(0.0);

        // Determine orderType string from current order
        let order_type_str = match &current.order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit { .. } => "LIMIT",
            OrderType::StopMarket { .. } => "STOP_LOSS_MARKET",
            OrderType::StopLimit { .. } => "STOP_LOSS",
            _ => "LIMIT",
        };

        let validity = match current.time_in_force {
            crate::core::TimeInForce::Ioc => "IOC",
            _ => "DAY",
        };

        let client_id = {
            let auth = self.auth.lock().await;
            auth.client_id().to_string()
        };

        let body = json!({
            "dhanClientId": client_id,
            "orderId": order_id,
            "orderType": order_type_str,
            "validity": validity,
            "quantity": new_quantity as i64,
            "price": new_price,
            "disclosedQuantity": 0,
            "triggerPrice": new_trigger,
        });

        let response = self.put(DhanEndpoint::ModifyOrder, &[("orderId", &order_id)], body).await?;
        DhanParser::parse_order_placement(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS — Trade history addition
// ═══════════════════════════════════════════════════════════════════════════════

impl DhanConnector {
    /// Recent trade history — `GET /v2/trades` (signed)
    ///
    /// Returns the latest trades executed on the account without requiring a date
    /// range. Use `get_trade_history_paginated` for date-filtered paginated results.
    pub async fn get_trade_history(&self) -> ExchangeResult<Value> {
        self.get(DhanEndpoint::GetRecentTrades, HashMap::new()).await
    }

    /// Paginated trade history — `GET /v2/trades/{fromDate}/{toDate}/{page}` (signed)
    ///
    /// Parameters:
    /// - `from_date`: start date in `YYYY-MM-DD` format
    /// - `to_date`: end date in `YYYY-MM-DD` format
    /// - `page`: page number (0-indexed)
    pub async fn get_trade_history_paginated(
        &self,
        from_date: &str,
        to_date: &str,
        page: u32,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url();
        let path = format!("/v2/trades/{}/{}/{}", from_date, to_date, page);
        let url = format!("{}{}", base_url, path);

        let base_url_owned = base_url.to_string();
        let mut auth = self.auth.lock().await;
        let headers = auth.build_headers(&base_url_owned, &self.http).await?;
        drop(auth);

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }
}

