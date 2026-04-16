//! # BingX Connector
//!
//! Реализация всех core трейтов для BingX.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Optional трейты
//! - `CancelAll` - отмена всех ордеров
//! - `AmendOrder` - изменение ордера (swap only)

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
    Position, FundingRate, MarginType,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    TimeInForce, AmendRequest,
    UserTrade, UserTradeFilter,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::{CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts};
use crate::core::types::{ConnectorStats, CancelAllResponse, OrderResult};
use crate::core::types::{
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord, FundsHistoryFilter, FundsRecordType,
    SubAccountOperation, SubAccountResult, SubAccount,
};
use crate::core::utils::SimpleRateLimiter;
use crate::core::utils::PrecisionCache;

use super::endpoints::{BingxUrls, BingxEndpoint, format_symbol, map_kline_interval};
use super::auth::BingxAuth;
use super::parser::BingxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// BingX коннектор
pub struct BingxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BingxAuth>,
    /// URL'ы
    urls: BingxUrls,
    /// Testnet / VST mode flag.
    /// BingX has no separate testnet URLs; VST (Virtual Simulated Trading) uses
    /// the same mainnet endpoints with "-VST" pair suffixes (e.g., BTC-USDT-VST).
    /// Stored here for future VST pair routing support.
    testnet: bool,
    /// Rate limiter для market data (100 req/10s)
    market_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl BingxConnector {
    /// Создать новый коннектор
    ///
    /// Note: BingX has no separate testnet URLs. When `testnet` is `true` the
    /// connector still connects to the same mainnet endpoints; VST
    /// (Virtual Simulated Trading) pairs use a "-VST" suffix on the symbol
    /// level (e.g. "BTC-USDT-VST") rather than a different base URL.
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        // BingX uses the same base URL for both mainnet and VST mode
        let urls = BingxUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(BingxAuth::new)
            .transpose()?;

        // BingX rate limit: 100 requests per 10 seconds (shared pool)
        let market_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            market_limiter,
            precision: PrecisionCache::new(),
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
        loop {
            let wait_time = {
                let mut limiter = self.market_limiter.lock().expect("Mutex poisoned");
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
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request(&mut params)
        } else {
            HashMap::new()
        };

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

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        // Build form body
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        let response = self.http.post(&url, &json!({}), &headers).await?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        // Build query string
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// Check BingX response for API errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
        if code != 0 {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code as i32,
                message: msg.to_string(),
            });
        }
        Ok(())
    }

    /// Build a minimal returned Order after placing
    fn build_placed_order(
        order_id: String,
        client_order_id: Option<String>,
        symbol: &Symbol,
        side: OrderSide,
        order_type: OrderType,
        price: Option<Price>,
        quantity: Quantity,
    ) -> Order {
        Order {
            id: order_id,
            client_order_id,
            symbol: symbol.to_string(),
            side,
            order_type,
            status: crate::core::OrderStatus::New,
            price,
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BingxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::BingX
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
        // BingX has no separate testnet URLs; this flag enables VST pair routing
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
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
impl MarketData for BingxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Use get_ticker and extract the last_price
        let ticker = self.get_ticker(symbol, account_type).await?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotDepth,
            _ => BingxEndpoint::SwapDepth,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("limit".to_string(), d.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_orderbook(&response)
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
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotKlines,
            _ => BingxEndpoint::SwapKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("interval".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1440).to_string());
        }

        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotTickerBookTicker,
            _ => BingxEndpoint::SwapTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // BingX doesn't have dedicated ping endpoint, use symbols as health check
        let response = self.get(BingxEndpoint::SpotSymbols, HashMap::new(), AccountType::Spot).await?;

        // Check response is valid
        if response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) == 0 {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let info = match account_type {
            AccountType::Spot | AccountType::Margin => {
                let response = self.get(BingxEndpoint::SpotSymbols, HashMap::new(), AccountType::Spot).await?;
                BingxParser::parse_spot_exchange_info(&response, account_type)?
            }
            _ => {
                let response = self.get(BingxEndpoint::SwapContracts, HashMap::new(), AccountType::FuturesCross).await?;
                BingxParser::parse_swap_exchange_info(&response, account_type)?
            }
        };
        self.precision.load_from_symbols(&info);
        Ok(info)
    }

    fn market_data_capabilities(&self) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            // SpotTrades / SwapTrades endpoints exist but get_recent_trades is not
            // implemented in the MarketData trait impl for this connector.
            has_recent_trades: false,
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m",
                "1h", "2h", "4h", "6h", "8h", "12h",
                "1d", "3d", "1w", "1M",
            ],
            // get_klines enforces .min(1440) before sending to BingX
            max_kline_limit: Some(1440),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BingxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        let endpoint = if is_futures { BingxEndpoint::SwapOrder } else { BingxEndpoint::SpotOrder };
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), formatted_symbol.clone());
        params.insert("side".to_string(), side_str.to_string());

        match req.order_type {
            OrderType::Market => {
                params.insert("type".to_string(), "MARKET".to_string());
                if is_futures {
                    params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                } else {
                    // BingX Spot: buy uses quoteOrderQty, sell uses quantity
                    match side {
                        OrderSide::Buy => {
                            params.insert("quoteOrderQty".to_string(), self.precision.qty(&formatted_symbol, quantity));
                        }
                        OrderSide::Sell => {
                            params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                        }
                    }
                }
            }

            OrderType::Limit { price } => {
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                params.insert("price".to_string(), self.precision.price(&formatted_symbol, price));
                if is_futures {
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                }
            }

            OrderType::PostOnly { price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "PostOnly is not documented for BingX Spot".to_string()
                    ));
                }
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                params.insert("price".to_string(), self.precision.price(&formatted_symbol, price));
                params.insert("timeInForce".to_string(), "PostOnly".to_string());
            }

            OrderType::Ioc { price } => {
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                if let Some(p) = price {
                    params.insert("price".to_string(), self.precision.price(&formatted_symbol, p));
                }
                params.insert("timeInForce".to_string(), "IOC".to_string());
            }

            OrderType::Fok { price } => {
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                params.insert("price".to_string(), self.precision.price(&formatted_symbol, price));
                params.insert("timeInForce".to_string(), "FOK".to_string());
            }

            OrderType::StopMarket { stop_price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopMarket is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                params.insert("type".to_string(), "STOP_MARKET".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                params.insert("stopPrice".to_string(), self.precision.price(&formatted_symbol, stop_price));
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopLimit is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                params.insert("type".to_string(), "STOP".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                params.insert("price".to_string(), self.precision.price(&formatted_symbol, limit_price));
                params.insert("stopPrice".to_string(), self.precision.price(&formatted_symbol, stop_price));
                params.insert("timeInForce".to_string(), "GTC".to_string());
            }

            OrderType::TrailingStop { callback_rate, activation_price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "TrailingStop is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                params.insert("type".to_string(), "TRAILING_STOP_MARKET".to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                // priceRate is the trailing distance as a percentage
                params.insert("priceRate".to_string(), callback_rate.to_string());
                if let Some(act_price) = activation_price {
                    params.insert("activationPrice".to_string(), self.precision.price(&formatted_symbol, act_price));
                }
            }

            OrderType::ReduceOnly { price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                let (type_str, price_val) = if let Some(p) = price {
                    ("LIMIT", Some(p))
                } else {
                    ("MARKET", None)
                };
                params.insert("type".to_string(), type_str.to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                params.insert("reduceOnly".to_string(), "true".to_string());
                if let Some(p) = price_val {
                    params.insert("price".to_string(), self.precision.price(&formatted_symbol, p));
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                }
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Bracket orders are only supported for BingX Swap (futures)".to_string()
                    ));
                }
                // BingX Swap supports embedded TP and SL via takeProfit/stopLoss JSON objects
                // However, the current transport sends form-encoded params, not JSON body.
                // BingX Swap order API typically accepts query params with JSON strings.
                // For bracket orders we use a limit/market entry with embedded TP/SL.
                let type_str = if price.is_some() { "LIMIT" } else { "MARKET" };
                params.insert("type".to_string(), type_str.to_string());
                params.insert("quantity".to_string(), self.precision.qty(&formatted_symbol, quantity));
                if let Some(p) = price {
                    params.insert("price".to_string(), self.precision.price(&formatted_symbol, p));
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                }
                // Encode TP/SL as JSON params — BingX accepts these as JSON-encoded strings
                let tp_json = json!({
                    "type": "TAKE_PROFIT_MARKET",
                    "stopPrice": self.precision.price(&formatted_symbol, take_profit),
                    "price": "0",
                    "workingType": "MARK_PRICE"
                });
                let sl_json = json!({
                    "type": "STOP_MARKET",
                    "stopPrice": self.precision.price(&formatted_symbol, stop_loss),
                    "price": "0",
                    "workingType": "MARK_PRICE"
                });
                params.insert("takeProfit".to_string(), tp_json.to_string());
                params.insert("stopLoss".to_string(), sl_json.to_string());
            }

            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on BingX", req.order_type)
                ));
            }
        }

        let response = self.post(endpoint, params, account_type).await?;
        self.check_response(&response)?;

        // Extract order from response
        let data = response.get("data").cloned().unwrap_or(json!({}));
        let order_data = data.get("order").cloned()
            .or_else(|| Some(data.clone()))
            .unwrap_or(json!({}));
        let order_id = order_data.get("orderId")
            .and_then(|v| v.as_str().map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_default();
        let client_order_id = order_data.get("clientOrderId")
            .and_then(|v| v.as_str())
            .map(String::from);

        let price_for_order = match &req.order_type {
            OrderType::Limit { price } | OrderType::PostOnly { price } | OrderType::Fok { price } => Some(*price),
            OrderType::Ioc { price } => *price,
            OrderType::StopLimit { limit_price, .. } => Some(*limit_price),
            _ => None,
        };

        Ok(PlaceOrderResponse::Simple(
            Self::build_placed_order(order_id, client_order_id, &symbol, side, req.order_type, price_for_order, quantity)
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;
                let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

                let endpoint = if is_futures { BingxEndpoint::SwapOrder } else { BingxEndpoint::SpotOrder };
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("orderId".to_string(), order_id.to_string());

                let response = self.delete(endpoint, params, account_type).await?;
                self.check_response(&response)?;
                BingxParser::parse_order(&response, &symbol.to_string())
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope — use CancelAll trait for all/bySymbol on BingX", req.scope)
            )),
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOrder,
            _ => BingxEndpoint::SwapOrder,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_order(&response, &symbol.to_string())
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let symbol: Option<crate::core::Symbol> = symbol.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOpenOrders,
            _ => BingxEndpoint::SwapOpenOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        let mut params = HashMap::new();

        if is_futures {
            // Futures AllOrders requires symbol
            let sym = filter.symbol.as_ref()
                .ok_or_else(|| ExchangeError::InvalidRequest(
                    "BingX Swap order history requires a symbol".to_string()
                ))?;
            params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
            if let Some(start) = filter.start_time {
                params.insert("startTime".to_string(), start.to_string());
            }
            if let Some(end) = filter.end_time {
                params.insert("endTime".to_string(), end.to_string());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.to_string());
            }
            let response = self.get(BingxEndpoint::SwapAllOrders, params, account_type).await?;
            BingxParser::parse_orders(&response)
        } else {
            // Spot history orders
            if let Some(ref sym) = filter.symbol {
                params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
            }
            if let Some(start) = filter.start_time {
                params.insert("startTime".to_string(), start.to_string());
            }
            if let Some(end) = filter.end_time {
                params.insert("endTime".to_string(), end.to_string());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.to_string());
            }
            let response = self.get(BingxEndpoint::SpotHistoryOrders, params, account_type).await?;
            BingxParser::parse_orders(&response)
        }
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        let mut params = HashMap::new();

        if is_futures {
            // Swap fill history: symbol required, uses startTs/endTs
            let sym = filter.symbol
                .ok_or_else(|| ExchangeError::InvalidRequest(
                    "BingX swap user trades requires a symbol".to_string()
                ))?;
            params.insert("symbol".to_string(), sym);
            if let Some(oid) = filter.order_id {
                params.insert("orderId".to_string(), oid);
            }
            if let Some(start) = filter.start_time {
                params.insert("startTs".to_string(), start.to_string());
            }
            if let Some(end) = filter.end_time {
                params.insert("endTs".to_string(), end.to_string());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.min(100).to_string());
            }
            let response = self.get(BingxEndpoint::SwapFillHistory, params, account_type).await?;
            BingxParser::parse_user_trades(&response, true)
        } else {
            // Spot my trades: symbol required
            let sym = filter.symbol
                .ok_or_else(|| ExchangeError::InvalidRequest(
                    "BingX spot user trades requires a symbol".to_string()
                ))?;
            params.insert("symbol".to_string(), sym);
            if let Some(oid) = filter.order_id {
                params.insert("orderId".to_string(), oid);
            }
            if let Some(start) = filter.start_time {
                params.insert("startTime".to_string(), start.to_string());
            }
            if let Some(end) = filter.end_time {
                params.insert("endTime".to_string(), end.to_string());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.min(100).to_string());
            }
            let response = self.get(BingxEndpoint::SpotMyTrades, params, account_type).await?;
            BingxParser::parse_user_trades(&response, false)
        }
    }

    fn trading_capabilities(&self) -> TradingCapabilities {
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            // StopMarket / StopLimit / TrailingStop: Swap (futures) only;
            // Spot returns UnsupportedOperation.
            has_stop_market: true,
            has_stop_limit: true,
            has_trailing_stop: true,
            // Bracket: Swap only via embedded takeProfit/stopLoss JSON params.
            has_bracket: true,
            // OCO: no implementation — no BingX OCO endpoint wired up.
            has_oco: false,
            // AmendOrder trait implemented for Swap via /openApi/swap/v1/trade/amend.
            has_amend: true,
            // BatchOrders trait implemented; max 5 orders per batch (Swap only).
            has_batch: true,
            max_batch_size: Some(5),
            // CancelAll trait implemented for both Spot and Swap.
            has_cancel_all: true,
            has_user_trades: true,
            has_order_history: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BingxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotBalance,
            _ => BingxEndpoint::SwapBalance,
        };

        let params = HashMap::new();
        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => BingxParser::parse_balances(&response),
            _ => BingxParser::parse_swap_balance(&response),
        }
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1, // Default BingX fees, should query from API
            taker_commission: 0.1,
            balances,
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // BingX provides commission rates via dedicated endpoints
        // Try spot commission rate first — works for both spot and futures context
        let params = HashMap::new();
        let response = self.get(BingxEndpoint::SpotCommissionRate, params, AccountType::Spot).await;

        if let Ok(response) = response {
            self.check_response(&response)?;
            if let Some(data) = response.get("data") {
                let maker = data.get("makerCommissionRate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| data.get("maker").and_then(|v| v.as_f64()))
                    .unwrap_or(0.001);
                let taker = data.get("takerCommissionRate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| data.get("taker").and_then(|v| v.as_f64()))
                    .unwrap_or(0.001);
                return Ok(FeeInfo {
                    maker_rate: maker,
                    taker_rate: taker,
                    symbol: _symbol.map(String::from),
                    tier: None,
                });
            }
        }

        // Fallback to default BingX fees
        Ok(FeeInfo {
            maker_rate: 0.001,
            taker_rate: 0.001,
            symbol: _symbol.map(String::from),
            tier: None,
        })
    }

    fn account_capabilities(&self) -> AccountCapabilities {
        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            // AccountTransfers trait implemented: POST /openApi/api/v3/post/account/innerTransfer
            has_transfers: true,
            // SubAccounts trait implemented: create, list, transfer, get_balance
            has_sub_accounts: true,
            // CustodialFunds trait implemented: deposit address, withdraw, deposit/withdrawal history
            has_deposit_withdraw: true,
            // No margin borrowing/repayment endpoints implemented
            has_margin: false,
            // No earn/staking product endpoints implemented
            has_earn_staking: false,
            // SwapIncome endpoint (/openApi/swap/v2/user/income) is defined but not exposed
            // through the Account trait; only raw connector method.
            has_funding_history: false,
            // No full ledger/transaction log endpoint implemented
            has_ledger: false,
            // No coin-to-coin convert endpoint implemented
            has_convert: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BingxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(BingxEndpoint::SwapPositions, params, account_type).await?;
        BingxParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let symbol_str = symbol;
        let sym = {
            let parts: Vec<&str> = symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.to_string(), quote: String::new(), raw: Some(symbol_str.to_string()) }
            }
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));

        let response = self.get(BingxEndpoint::SwapFundingRate, params, account_type).await?;

        // Parse funding rate from response
        let data = response.get("data").cloned().unwrap_or(json!({}));
        let rate = data.get("fundingRate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| data.get("fundingRate").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);
        let next_time = data.get("nextFundingTime")
            .and_then(|v| v.as_i64());

        Ok(FundingRate {
            symbol: symbol_str.to_string(),
            rate,
            next_funding_time: next_time,
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "Leverage not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), "LONG".to_string()); // BingX requires side
                params.insert("leverage".to_string(), leverage.to_string());

                let response = self.post(BingxEndpoint::SwapLeverage, params, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }

            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "MarginMode not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let margin_type_str = match margin_type {
                    MarginType::Cross => "CROSSED",
                    MarginType::Isolated => "ISOLATED",
                };
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("marginType".to_string(), margin_type_str.to_string());

                let response = self.post(BingxEndpoint::SwapMarginType, params, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }

            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                // BingX: place a market order with closePosition=true
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), "SELL".to_string()); // for long position
                params.insert("type".to_string(), "MARKET".to_string());
                params.insert("closePosition".to_string(), "true".to_string());

                let response = self.post(BingxEndpoint::SwapOrder, params, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on BingX", req)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for BingxConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        match scope {
            CancelScope::All { ref symbol } => {
                if is_futures {
                    // Swap cancel-all requires symbol
                    let sym = symbol.as_ref()
                        .ok_or_else(|| ExchangeError::InvalidRequest(
                            "BingX Swap cancel-all requires a symbol".to_string()
                        ))?;
                    let mut params = HashMap::new();
                    params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
                    let response = self.delete(BingxEndpoint::SwapCancelAllOrders, params, account_type).await?;
                    self.check_response(&response)?;
                    Ok(CancelAllResponse {
                        cancelled_count: 0, // BingX doesn't return count in cancel-all response
                        failed_count: 0,
                        details: vec![],
                    })
                } else {
                    let sym = symbol.as_ref()
                        .ok_or_else(|| ExchangeError::InvalidRequest(
                            "BingX Spot cancel-all requires a symbol".to_string()
                        ))?;
                    let mut params = HashMap::new();
                    params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
                    let response = self.delete(BingxEndpoint::SpotCancelAllOrders, params, account_type).await?;
                    self.check_response(&response)?;
                    Ok(CancelAllResponse {
                        cancelled_count: 0,
                        failed_count: 0,
                        details: vec![],
                    })
                }
            }

            CancelScope::BySymbol { ref symbol } => {
                let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted_symbol);

                let endpoint = if is_futures {
                    BingxEndpoint::SwapCancelAllOrders
                } else {
                    BingxEndpoint::SpotCancelAllOrders
                };

                let response = self.delete(endpoint, params, account_type).await?;
                self.check_response(&response)?;

                Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: vec![],
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                "cancel_all_orders only supports All and BySymbol scopes".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (Swap only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for BingxConnector {
    /// Place multiple orders in a single batch request (Swap/Futures only).
    ///
    /// Endpoint: POST /openApi/swap/v2/trade/batchOrders
    /// Body: `{"batchOrders": "[{...}, ...]"}` — JSON-encoded string
    /// Max 5 orders per batch for BingX Swap.
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        // BingX batch orders are swap-only
        let account_type = orders.first().map(|o| o.account_type).unwrap_or(AccountType::FuturesCross);
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        if !is_futures {
            return Err(ExchangeError::UnsupportedOperation(
                "BingX batch orders only supported for Swap (futures)".to_string()
            ));
        }

        // Build batch order objects
        let batch_orders: Vec<Value> = orders.iter().map(|req| {
            let side_str = match req.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
            let formatted_symbol = format_symbol(&req.symbol.base, &req.symbol.quote, req.account_type);

            let mut obj = json!({
                "symbol": formatted_symbol,
                "side": side_str,
                "quantity": req.quantity.to_string(),
            });

            match &req.order_type {
                OrderType::Market => {
                    obj["type"] = json!("MARKET");
                }
                OrderType::Limit { price } => {
                    obj["type"] = json!("LIMIT");
                    obj["price"] = json!(price.to_string());
                    obj["timeInForce"] = json!("GTC");
                }
                OrderType::PostOnly { price } => {
                    obj["type"] = json!("LIMIT");
                    obj["price"] = json!(price.to_string());
                    obj["timeInForce"] = json!("PostOnly");
                }
                _ => {
                    obj["type"] = json!("MARKET");
                }
            }

            obj
        }).collect();

        // BingX encodes batchOrders as a JSON string
        let batch_orders_str = serde_json::to_string(&batch_orders)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize batch orders: {}", e)))?;

        let mut params = HashMap::new();
        params.insert("batchOrders".to_string(), batch_orders_str);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        let base_url = self.urls.rest_url(account_type);
        let path = BingxEndpoint::SwapBatchOrders.path();

        // Build query string with signed params
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        self.rate_limit_wait().await;
        let response = self.http.post(&url, &json!({}), &headers).await?;
        self.check_response(&response)?;

        // Parse response — array of order results
        let data = response.get("data").cloned().unwrap_or(json!([]));
        let results = if let Some(arr) = data.as_array() {
            arr.iter().enumerate().map(|(i, item)| {
                let order_id = item.get("orderId")
                    .and_then(|v| v.as_str().map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string())));
                let code = item.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                let success = code == 0 && order_id.is_some();

                let req = orders.get(i);
                OrderResult {
                    order: order_id.map(|id| Order {
                        id,
                        client_order_id: None,
                        symbol: req.map(|o| o.symbol.to_string()).unwrap_or_default(),
                        side: req.map(|o| o.side).unwrap_or(OrderSide::Buy),
                        order_type: req.map(|o| o.order_type.clone()).unwrap_or(OrderType::Market),
                        status: crate::core::OrderStatus::New,
                        price: None,
                        stop_price: None,
                        quantity: req.map(|o| o.quantity).unwrap_or(0.0),
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: crate::core::timestamp_millis() as i64,
                        updated_at: None,
                        time_in_force: TimeInForce::Gtc,
                    }),
                    client_order_id: None,
                    success,
                    error: if success { None } else {
                        item.get("msg").and_then(|v| v.as_str()).map(String::from)
                    },
                    error_code: if success { None } else { Some(code as i32) },
                }
            }).collect()
        } else {
            orders.iter().map(|_| OrderResult {
                order: None,
                client_order_id: None,
                success: false,
                error: Some("Unexpected response format".to_string()),
                error_code: None,
            }).collect()
        };

        Ok(results)
    }

    /// Cancel multiple orders by IDs (Swap only).
    ///
    /// Endpoint: DELETE /openApi/swap/v2/trade/batchOrders
    /// Param: `orderIdList` — JSON array of order IDs as a string
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
        if !is_futures {
            return Err(ExchangeError::UnsupportedOperation(
                "BingX batch cancel only supported for Swap (futures)".to_string()
            ));
        }

        let sym = symbol.ok_or_else(|| ExchangeError::InvalidRequest(
            "Symbol required for BingX batch cancel".to_string()
        ))?;

        let ids_str = serde_json::to_string(&order_ids)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize order IDs: {}", e)))?;

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), sym.to_string());
        params.insert("orderIdList".to_string(), ids_str);

        let response = self.delete(BingxEndpoint::SwapBatchCancelOrders, params, account_type).await?;
        self.check_response(&response)?;

        let data = response.get("data").cloned().unwrap_or(json!([]));
        let results = if let Some(arr) = data.as_array() {
            arr.iter().map(|item| {
                let success = item.get("code").and_then(|v| v.as_i64()).unwrap_or(0) == 0;
                OrderResult {
                    order: None,
                    client_order_id: None,
                    success,
                    error: if success { None } else {
                        item.get("msg").and_then(|v| v.as_str()).map(String::from)
                    },
                    error_code: None,
                }
            }).collect()
        } else {
            order_ids.iter().map(|_| OrderResult {
                order: None,
                client_order_id: None,
                success: true,
                error: None,
                error_code: None,
            }).collect()
        };

        Ok(results)
    }

    /// Maximum batch place size (BingX Swap limit: 5 orders per batch).
    fn max_batch_place_size(&self) -> usize {
        5
    }

    /// Maximum batch cancel size (BingX Swap: no documented hard limit, use 20).
    fn max_batch_cancel_size(&self) -> usize {
        20
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (Swap only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for BingxConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let symbol = &req.symbol;
        let account_type = req.account_type;
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        if !is_futures {
            return Err(ExchangeError::UnsupportedOperation(
                "AmendOrder is only supported for BingX Swap (futures). Spot requires cancel+replace.".to_string()
            ));
        }

        // POST /openApi/swap/v1/trade/amend
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str.clone());
        params.insert("orderId".to_string(), req.order_id.clone());

        if let Some(new_price) = req.fields.price {
            params.insert("price".to_string(), self.precision.price(&symbol_str, new_price));
        }
        if let Some(new_qty) = req.fields.quantity {
            params.insert("quantity".to_string(), self.precision.qty(&symbol_str, new_qty));
        }
        if let Some(trigger) = req.fields.trigger_price {
            params.insert("stopPrice".to_string(), self.precision.price(&symbol_str, trigger));
        }

        let response = self.post(BingxEndpoint::SwapAmend, params, account_type).await?;
        self.check_response(&response)?;

        // Fetch the updated order
        self.get_order(&symbol.to_string(), &req.order_id, account_type).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountTransfers for BingxConnector {
    /// Transfer between Fund and Standard account.
    ///
    /// Endpoint: POST /openApi/api/v3/post/account/innerTransfer
    /// Params: asset, amount, transferSide (FUND_TO_STANDARD | STANDARD_TO_FUND)
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        use crate::core::AccountType;

        // Determine direction from from_account / to_account
        let transfer_side = match (&req.from_account, &req.to_account) {
            (AccountType::Spot, AccountType::FuturesCross)
            | (AccountType::Spot, AccountType::FuturesIsolated) => "STANDARD_TO_FUND",
            (AccountType::FuturesCross, AccountType::Spot)
            | (AccountType::FuturesIsolated, AccountType::Spot) => "FUND_TO_STANDARD",
            _ => "STANDARD_TO_FUND",
        };

        let mut params = HashMap::new();
        params.insert("asset".to_string(), req.asset.clone());
        params.insert("amount".to_string(), req.amount.to_string());
        params.insert("transferSide".to_string(), transfer_side.to_string());

        let response = self.post(BingxEndpoint::InnerTransfer, params, AccountType::Spot).await?;
        self.check_response(&response)?;

        let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
        let transfer_id = data.get("tranId")
            .and_then(|v| v.as_str().map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(TransferResponse {
            transfer_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: None,
        })
    }

    /// Get internal transfer history.
    ///
    /// Endpoint: GET /openApi/api/v3/get/asset/transfer
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        use crate::core::AccountType;

        let mut params = HashMap::new();
        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(BingxEndpoint::TransferHistory, params, AccountType::Spot).await?;
        let data = BingxParser::extract_data(&response)?;

        let records = data.as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|item| {
                let transfer_id = item.get("tranId")
                    .and_then(|v| v.as_str().map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string())))
                    .unwrap_or_default();
                let asset = item.get("asset")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let amount = BingxParser::get_f64(item, "amount").unwrap_or(0.0);
                let status = item.get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let timestamp = item.get("timestamp").and_then(|v| v.as_i64());

                TransferResponse {
                    transfer_id,
                    status,
                    asset,
                    amount,
                    timestamp,
                }
            })
            .collect();

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for BingxConnector {
    /// Get deposit address for an asset on a given network.
    ///
    /// Endpoint: GET /openApi/wallets/v1/capital/deposit/address
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        use crate::core::AccountType;

        let mut params = HashMap::new();
        params.insert("coin".to_string(), asset.to_string());
        if let Some(net) = network {
            params.insert("network".to_string(), net.to_string());
        }

        let response = self.get(BingxEndpoint::DepositAddress, params, AccountType::Spot).await?;
        let data = BingxParser::extract_data(&response)?;

        let address = data.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'address' field".to_string()))?
            .to_string();
        let tag = data.get("tag")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let net = data.get("network")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| network.map(String::from));

        Ok(DepositAddress {
            address,
            tag,
            network: net,
            asset: asset.to_string(),
            created_at: None,
        })
    }

    /// Submit a withdrawal request.
    ///
    /// Endpoint: POST /openApi/wallets/v1/capital/withdraw/apply
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        use crate::core::AccountType;

        let mut params = HashMap::new();
        params.insert("coin".to_string(), req.asset.clone());
        params.insert("address".to_string(), req.address.clone());
        params.insert("amount".to_string(), req.amount.to_string());
        // walletType: 0 = on-chain, 1 = internal
        params.insert("walletType".to_string(), "0".to_string());
        if let Some(net) = &req.network {
            params.insert("network".to_string(), net.clone());
        }
        if let Some(tag) = &req.tag {
            params.insert("addressTag".to_string(), tag.clone());
        }

        let response = self.post(BingxEndpoint::Withdraw, params, AccountType::Spot).await?;
        self.check_response(&response)?;

        let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
        let withdraw_id = data.get("id")
            .and_then(|v| v.as_str().map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Get deposit and/or withdrawal history.
    ///
    /// Endpoint (deposits): GET /openApi/api/v3/capital/deposit/hisrec
    /// Endpoint (withdrawals): GET /openApi/api/v3/capital/withdraw/history
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        use crate::core::AccountType;

        let mut records: Vec<FundsRecord> = Vec::new();

        let build_params = |f: &FundsHistoryFilter| {
            let mut p = HashMap::new();
            if let Some(a) = &f.asset {
                p.insert("coin".to_string(), a.clone());
            }
            if let Some(s) = f.start_time {
                p.insert("startTime".to_string(), s.to_string());
            }
            if let Some(e) = f.end_time {
                p.insert("endTime".to_string(), e.to_string());
            }
            if let Some(l) = f.limit {
                p.insert("limit".to_string(), l.to_string());
            }
            p
        };

        // Fetch deposits
        if matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both) {
            let params = build_params(&filter);
            let response = self.get(BingxEndpoint::DepositHistory, params, AccountType::Spot).await?;
            if let Ok(data) = BingxParser::extract_data(&response) {
                if let Some(arr) = data.as_array() {
                    for item in arr {
                        let id = item.get("id")
                            .and_then(|v| v.as_str().map(String::from)
                                .or_else(|| v.as_i64().map(|n| n.to_string())))
                            .unwrap_or_default();
                        let asset = item.get("coin").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let amount = BingxParser::get_f64(item, "amount").unwrap_or(0.0);
                        let tx_hash = item.get("txId").and_then(|v| v.as_str()).map(String::from);
                        let network = item.get("network").and_then(|v| v.as_str()).map(String::from);
                        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                        let timestamp = item.get("insertTime").and_then(|v| v.as_i64()).unwrap_or(0);

                        records.push(FundsRecord::Deposit {
                            id,
                            asset,
                            amount,
                            tx_hash,
                            network,
                            status,
                            timestamp,
                        });
                    }
                }
            }
        }

        // Fetch withdrawals
        if matches!(filter.record_type, FundsRecordType::Withdrawal | FundsRecordType::Both) {
            let params = build_params(&filter);
            let response = self.get(BingxEndpoint::WithdrawHistory, params, AccountType::Spot).await?;
            if let Ok(data) = BingxParser::extract_data(&response) {
                if let Some(arr) = data.as_array() {
                    for item in arr {
                        let id = item.get("id")
                            .and_then(|v| v.as_str().map(String::from)
                                .or_else(|| v.as_i64().map(|n| n.to_string())))
                            .unwrap_or_default();
                        let asset = item.get("coin").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let amount = BingxParser::get_f64(item, "amount").unwrap_or(0.0);
                        let fee = BingxParser::get_f64(item, "transactionFee");
                        let address = item.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let tag = item.get("addressTag").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from);
                        let tx_hash = item.get("txId").and_then(|v| v.as_str()).map(String::from);
                        let network = item.get("network").and_then(|v| v.as_str()).map(String::from);
                        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                        let timestamp = item.get("applyTime").and_then(|v| v.as_i64()).unwrap_or(0);

                        records.push(FundsRecord::Withdrawal {
                            id,
                            asset,
                            amount,
                            fee,
                            address,
                            tag,
                            tx_hash,
                            network,
                            status,
                            timestamp,
                        });
                    }
                }
            }
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl SubAccounts for BingxConnector {
    /// Perform a sub-account operation.
    ///
    /// - Create: POST /openApi/subAccount/v1/create
    /// - List: GET /openApi/subAccount/v1/list
    /// - Transfer: POST /openApi/subAccount/v1/transfer
    /// - GetBalance: GET /openApi/subAccount/v1/assets
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        use crate::core::AccountType;

        match op {
            SubAccountOperation::Create { label } => {
                let mut params = HashMap::new();
                params.insert("subAccountString".to_string(), label.clone());

                let response = self.post(BingxEndpoint::SubAccountCreate, params, AccountType::Spot).await?;
                self.check_response(&response)?;

                let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
                let id = data.get("subUid")
                    .and_then(|v| v.as_str().map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string())));

                Ok(SubAccountResult {
                    id,
                    name: Some(label),
                    accounts: vec![],
                    transaction_id: None,
                })
            }

            SubAccountOperation::List => {
                let response = self.get(BingxEndpoint::SubAccountList, HashMap::new(), AccountType::Spot).await?;
                let data = BingxParser::extract_data(&response)?;

                let accounts = data.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|item| {
                        let id = item.get("subUid")
                            .and_then(|v| v.as_str().map(String::from)
                                .or_else(|| v.as_i64().map(|n| n.to_string())))
                            .unwrap_or_default();
                        let name = item.get("note")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let status = item.get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Normal")
                            .to_string();
                        SubAccount { id, name, status }
                    })
                    .collect();

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                // type: 1 = master → sub, 2 = sub → master
                let transfer_type = if to_sub { "1" } else { "2" };

                let mut params = HashMap::new();
                params.insert("subUid".to_string(), sub_account_id);
                params.insert("coin".to_string(), asset.clone());
                params.insert("amount".to_string(), amount.to_string());
                params.insert("type".to_string(), transfer_type.to_string());

                let response = self.post(BingxEndpoint::SubAccountTransfer, params, AccountType::Spot).await?;
                self.check_response(&response)?;

                let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
                let transaction_id = data.get("tranId")
                    .and_then(|v| v.as_str().map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string())));

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id,
                })
            }

            SubAccountOperation::GetBalance { sub_account_id } => {
                let mut params = HashMap::new();
                params.insert("subUid".to_string(), sub_account_id.clone());

                let response = self.get(BingxEndpoint::SubAccountAssets, params, AccountType::Spot).await?;
                self.check_response(&response)?;

                Ok(SubAccountResult {
                    id: Some(sub_account_id),
                    name: None,
                    accounts: vec![],
                    transaction_id: None,
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS — Market data & trade history additions
// ═══════════════════════════════════════════════════════════════════════════════

impl BingxConnector {
    /// Spot fill/trade history — `GET /openApi/spot/v1/trade/myTrades` (signed)
    ///
    /// Returns the authenticated user's spot trade fills.
    /// Optional params: `symbol`, `orderId`, `startTime`, `endTime`, `fromId`, `limit`.
    pub async fn spot_my_trades(
        &self,
        symbol: Option<&str>,
        order_id: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        if let Some(oid) = order_id {
            params.insert("orderId".to_string(), oid.to_string());
        }
        if let Some(st) = start_time {
            params.insert("startTime".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(500).to_string());
        }
        let response = self.get(BingxEndpoint::SpotMyTrades, params, AccountType::Spot).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Swap all fill orders — `GET /openApi/swap/v2/trade/allFillOrders` (signed)
    ///
    /// Returns the authenticated user's perpetual swap fill history.
    /// Optional params: `symbol`, `orderId`, `startTime`, `endTime`, `lastFillId`, `pageIndex`.
    pub async fn swap_all_fill_orders(
        &self,
        symbol: Option<&str>,
        order_id: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        if let Some(oid) = order_id {
            params.insert("orderId".to_string(), oid.to_string());
        }
        if let Some(st) = start_time {
            params.insert("startTime".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("pageSize".to_string(), l.min(500).to_string());
        }
        let response = self.get(BingxEndpoint::SwapAllFillOrders, params, AccountType::FuturesCross).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Swap open interest — `GET /openApi/swap/v2/quote/openInterest` (public)
    ///
    /// Required param: `symbol` (e.g. `BTC-USDT`).
    pub async fn swap_open_interest(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        let response = self.get(BingxEndpoint::SwapOpenInterest, params, AccountType::FuturesCross).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Swap funding rate history — `GET /openApi/swap/v2/quote/fundingRateHistory` (public)
    ///
    /// Required param: `symbol`. Optional: `startTime`, `endTime`, `limit`.
    pub async fn swap_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(st) = start_time {
            params.insert("startTime".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }
        let response = self.get(BingxEndpoint::SwapFundingRateHistory, params, AccountType::FuturesCross).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Swap premium index (mark price + index price + funding rate) —
    /// `GET /openApi/swap/v2/quote/premiumIndex` (public)
    ///
    /// Optional param: `symbol`. Without symbol returns all contracts.
    pub async fn swap_premium_index(&self, symbol: Option<&str>) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        let response = self.get(BingxEndpoint::SwapPremiumIndex, params, AccountType::FuturesCross).await?;
        self.check_response(&response)?;
        Ok(response)
    }
}
