//! Tinkoff Invest connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

#[cfg(feature = "grpc")]
use super::proto::tinkoff::{
    self as tinkoff_proto,
    GetCandlesRequest, GetCandlesResponse,
    GetOrderBookRequest, GetOrderBookResponse,
    GetLastPricesRequest, GetLastPricesResponse,
    GetTradingStatusRequest, GetTradingStatusResponse,
    PostOrderRequest, PostOrderResponse,
    CancelOrderRequest, CancelOrderResponse,
    GetOrdersRequest, GetOrdersResponse,
    GetOrderStateRequest, OrderState,
    PortfolioRequest, PortfolioResponse,
    PositionsRequest, PositionsResponse,
    Timestamp,
    paths,
};
#[cfg(feature = "grpc")]
use tonic::codec::ProstCodec;
#[cfg(feature = "grpc")]
use tonic::transport::Channel;

/// Tinkoff Invest connector
///
/// Russian broker with full trading support for MOEX (Moscow Exchange).
///
/// ## Features
/// - Real-time market data
/// - Historical candles (5s to 1 month intervals, up to 10 years)
/// - Full trading support (stocks, bonds, ETFs, futures, options)
/// - Portfolio and position tracking
/// - Multiple account types (standard, IIS, sandbox)
///
/// ## Authentication
/// All endpoints require Bearer token authentication.
/// Generate token at: https://www.tinkoff.ru/invest/settings/
pub struct TinkoffConnector {
    client: Client,
    auth: TinkoffAuth,
    endpoints: TinkoffEndpoints,
    testnet: bool,
    /// Account ID to use for operations (set after GetAccounts)
    account_id: Option<String>,
    /// Optional gRPC channel to the Tinkoff Invest gRPC endpoint.
    ///
    /// When present, gRPC methods become available (get_candles_grpc,
    /// place_order_grpc, etc.).  Absent by default — the connector operates
    /// via the REST proxy.
    #[cfg(feature = "grpc")]
    grpc_channel: Option<Channel>,
}

impl TinkoffConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `token` - API token (starts with "t.")
    /// * `testnet` - Use sandbox environment
    pub fn new(token: impl Into<String>, testnet: bool) -> Self {
        let endpoints = if testnet {
            TinkoffEndpoints::sandbox()
        } else {
            TinkoffEndpoints::default()
        };

        Self {
            client: Client::new(),
            auth: TinkoffAuth::new(token),
            endpoints,
            testnet,
            account_id: None,
            #[cfg(feature = "grpc")]
            grpc_channel: None,
        }
    }

    /// Create connector from environment variable TINKOFF_TOKEN
    pub fn from_env() -> Self {
        Self::new(TinkoffAuth::from_env().token, false)
    }

    /// Create sandbox connector from environment variable TINKOFF_SANDBOX_TOKEN
    pub fn from_env_sandbox() -> Self {
        let token = std::env::var("TINKOFF_SANDBOX_TOKEN")
            .unwrap_or_default();
        Self::new(token, true)
    }

    /// Set account ID to use for operations
    pub fn set_account_id(&mut self, account_id: impl Into<String>) {
        self.account_id = Some(account_id.into());
    }

    /// Get list of accounts and set the first one as active
    ///
    /// This is useful for initializing the connector.
    /// Tinkoff requires account_id for most trading operations.
    pub async fn initialize_account(&mut self) -> ExchangeResult<String> {
        let accounts = self.get_accounts_list().await?;
        if accounts.is_empty() {
            return Err(ExchangeError::NotFound("No accounts found".to_string()));
        }

        let account_id = accounts[0].clone();
        self.account_id = Some(account_id.clone());
        Ok(account_id)
    }

    /// Get list of account IDs
    pub async fn get_accounts_list(&self) -> ExchangeResult<Vec<String>> {
        let response = self.post(TinkoffEndpoint::GetAccounts, serde_json::json!({})).await?;

        let accounts = response
            .get("accounts")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'accounts' array".to_string()))?;

        Ok(accounts.iter()
            .filter_map(|acc| acc.get("id").and_then(|id| id.as_str()))
            .map(|s| s.to_string())
            .collect())
    }

    /// Internal: Make POST request (Tinkoff uses POST for all methods)
    async fn post(
        &self,
        endpoint: TinkoffEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add JSON body
        request = request.json(&body);

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse Tinkoff error format
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(code) = error_json.get("code").and_then(|c| c.as_i64()) {
                    let message = error_json.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");

                    // Map Tinkoff error codes to ExchangeError
                    return Err(match code {
                        40003 => ExchangeError::Auth("Invalid or expired token".to_string()),
                        40002 => ExchangeError::PermissionDenied("Insufficient privileges - use full-access token for trading".to_string()),
                        30052 => ExchangeError::InvalidRequest("Instrument forbidden for API trading".to_string()),
                        50002 => ExchangeError::NotFound("Instrument not found".to_string()),
                        80002 => ExchangeError::RateLimit,
                        90003 => ExchangeError::InvalidRequest("Order value exceeds 6,000,000 RUB limit".to_string()),
                        _ => ExchangeError::Api {
                            code: code as i32,
                            message: message.to_string()
                        },
                    });
                }
            }

            return Err(ExchangeError::Http(format!("HTTP {} - {}", status, error_text)));
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Get FIGI for a ticker symbol
    ///
    /// FIGI (Financial Instrument Global Identifier) is required for many operations.
    /// This method searches for instruments by ticker.
    pub async fn get_figi_by_ticker(&self, ticker: &str) -> ExchangeResult<String> {
        let body = serde_json::json!({
            "query": ticker,
        });

        let response = self.post(TinkoffEndpoint::FindInstrument, body).await?;

        let instruments = response
            .get("instruments")
            .and_then(|i| i.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'instruments' array".to_string()))?;

        if instruments.is_empty() {
            return Err(ExchangeError::NotFound(format!("Instrument '{}' not found", ticker)));
        }

        // Return first matching instrument's FIGI
        instruments[0]
            .get("figi")
            .and_then(|f| f.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing FIGI in response".to_string()))
    }

    /// Map candle interval to Tinkoff format
    fn map_interval(interval: &str) -> &'static str {
        match interval {
            "5s" => "CANDLE_INTERVAL_5_SEC",
            "10s" => "CANDLE_INTERVAL_10_SEC",
            "30s" => "CANDLE_INTERVAL_30_SEC",
            "1m" => "CANDLE_INTERVAL_1_MIN",
            "2m" => "CANDLE_INTERVAL_2_MIN",
            "3m" => "CANDLE_INTERVAL_3_MIN",
            "5m" => "CANDLE_INTERVAL_5_MIN",
            "10m" => "CANDLE_INTERVAL_10_MIN",
            "15m" => "CANDLE_INTERVAL_15_MIN",
            "30m" => "CANDLE_INTERVAL_30_MIN",
            "1h" => "CANDLE_INTERVAL_HOUR",
            "2h" => "CANDLE_INTERVAL_2_HOUR",
            "4h" => "CANDLE_INTERVAL_4_HOUR",
            "1d" => "CANDLE_INTERVAL_DAY",
            "1w" => "CANDLE_INTERVAL_WEEK",
            "1M" => "CANDLE_INTERVAL_MONTH",
            _ => "CANDLE_INTERVAL_HOUR", // default
        }
    }

    /// Calculate time range for candles based on limit and interval
    fn calculate_time_range(limit: u16, interval: &str) -> (String, String) {
        use chrono::{Utc, Duration};

        let now = Utc::now();
        let seconds_per_candle = match interval {
            "5s" => 5,
            "10s" => 10,
            "30s" => 30,
            "1m" => 60,
            "2m" => 120,
            "3m" => 180,
            "5m" => 300,
            "10m" => 600,
            "15m" => 900,
            "30m" => 1800,
            "1h" => 3600,
            "2h" => 7200,
            "4h" => 14400,
            "1d" => 86400,
            "1w" => 604800,
            "1M" => 2592000, // approximate
            _ => 3600,
        };

        let total_seconds = seconds_per_candle * limit as i64;
        let from = now - Duration::seconds(total_seconds);

        (
            from.to_rfc3339(),
            now.to_rfc3339(),
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for TinkoffConnector {
    fn exchange_name(&self) -> &'static str {
        "tinkoff"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Tinkoff
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Tinkoff supports spot trading (stocks, bonds, ETFs) and futures
        vec![AccountType::Spot, AccountType::FuturesCross]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for TinkoffConnector {
    /// Get current price using GetLastPrices
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<f64> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        let body = serde_json::json!({
            "figi": [figi],
        });

        let response = self.post(TinkoffEndpoint::GetLastPrices, body).await?;
        TinkoffParser::parse_price(&response)
    }

    /// Get ticker (24h stats) using GetOrderBook
    ///
    /// Note: Tinkoff doesn't provide 24h stats like crypto exchanges.
    /// We use order book data to construct a basic ticker.
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        let body = serde_json::json!({
            "figi": figi,
            "depth": 1,
        });

        let response = self.post(TinkoffEndpoint::GetOrderBook, body).await?;
        TinkoffParser::parse_ticker(&response, &ticker)
    }

    /// Get orderbook using GetOrderBook
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        // Tinkoff supports depths: 1, 10, 20, 30, 40, 50
        let depth_value = match depth.unwrap_or(10) {
            1 => 1,
            d if d <= 10 => 10,
            d if d <= 20 => 20,
            d if d <= 30 => 30,
            d if d <= 40 => 40,
            _ => 50,
        };

        let body = serde_json::json!({
            "figi": figi,
            "depth": depth_value,
        });

        let response = self.post(TinkoffEndpoint::GetOrderBook, body).await?;
        TinkoffParser::parse_orderbook(&response)
    }

    /// Get klines/candles using GetCandles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        let limit_value = limit.unwrap_or(100).min(2500); // Max 2500 candles
        let (from, to) = Self::calculate_time_range(limit_value, interval);
        let interval_enum = Self::map_interval(interval);

        let body = serde_json::json!({
            "figi": figi,
            "from": from,
            "to": to,
            "interval": interval_enum,
        });

        let response = self.post(TinkoffEndpoint::GetCandles, body).await?;
        TinkoffParser::parse_klines(&response)
    }

    /// Ping the server
    async fn ping(&self) -> ExchangeResult<()> {
        // Use GetAccounts as a simple ping endpoint
        let _response = self.post(TinkoffEndpoint::GetAccounts, serde_json::json!({})).await?;
        Ok(())
    }

    /// Get exchange info — returns list of available MOEX shares from Tinkoff
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let body = serde_json::json!({
            "instrumentStatus": "INSTRUMENT_STATUS_BASE",
        });

        let response = self.post(TinkoffEndpoint::Shares, body).await?;
        let symbols = TinkoffParser::parse_symbols(&response)?;

        let infos = symbols.into_iter().map(|ticker| SymbolInfo {
            symbol: ticker.clone(),
            base_asset: ticker,
            quote_asset: "RUB".to_string(),
            status: "TRADING".to_string(),
            price_precision: 2,
            quantity_precision: 0,
            min_quantity: Some(1.0),
            max_quantity: None,
            tick_size: None,
            step_size: Some(1.0),
            min_notional: None,
            account_type,
        }).collect();

        Ok(infos)
    }
}

impl TinkoffConnector {
    /// Get available symbols using Shares endpoint (extended method)
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let body = serde_json::json!({
            "instrumentStatus": "INSTRUMENT_STATUS_BASE",
        });

        let response = self.post(TinkoffEndpoint::Shares, body).await?;
        TinkoffParser::parse_symbols(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (Full support - Tinkoff is a broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for TinkoffConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;

        let direction_str = match side {
            OrderSide::Buy => "ORDER_DIRECTION_BUY",
            OrderSide::Sell => "ORDER_DIRECTION_SELL",
        };

        match req.order_type {
            OrderType::Market => {
                let account_id = self.account_id.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest(
                        "Account ID not set. Call initialize_account() first".to_string()
                    ))?;

                let ticker = format_ticker(&symbol);
                let figi = self.get_figi_by_ticker(&ticker).await?;
                let order_id = req.client_order_id
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                let body = serde_json::json!({
                    "figi": figi,
                    "quantity": quantity as i64,
                    "direction": direction_str,
                    "accountId": account_id,
                    "orderType": "ORDER_TYPE_MARKET",
                    "orderId": order_id,
                });

                let response = self.post(TinkoffEndpoint::PostOrder, body).await?;
                let mut result = TinkoffParser::parse_order_result(&response)?;
                result.symbol = ticker;
                Ok(PlaceOrderResponse::Simple(result))
            }

            OrderType::Limit { price } => {
                let account_id = self.account_id.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest(
                        "Account ID not set. Call initialize_account() first".to_string()
                    ))?;

                let ticker = format_ticker(&symbol);
                let figi = self.get_figi_by_ticker(&ticker).await?;
                let order_id = req.client_order_id
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                let (units, nano) = price_to_quotation(price);

                let body = serde_json::json!({
                    "figi": figi,
                    "quantity": quantity as i64,
                    "direction": direction_str,
                    "accountId": account_id,
                    "orderType": "ORDER_TYPE_LIMIT",
                    "orderId": order_id,
                    "price": { "units": units, "nano": nano },
                });

                let response = self.post(TinkoffEndpoint::PostOrder, body).await?;
                let mut result = TinkoffParser::parse_order_result(&response)?;
                result.symbol = ticker;
                Ok(PlaceOrderResponse::Simple(result))
            }

            OrderType::StopMarket { stop_price } => {
                let account_id = self.account_id.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest(
                        "Account ID not set. Call initialize_account() first".to_string()
                    ))?;

                let ticker = format_ticker(&symbol);
                let figi = self.get_figi_by_ticker(&ticker).await?;
                let (stop_units, stop_nano) = price_to_quotation(stop_price);

                let stop_direction = match side {
                    OrderSide::Buy => "STOP_ORDER_DIRECTION_BUY",
                    OrderSide::Sell => "STOP_ORDER_DIRECTION_SELL",
                };

                let body = serde_json::json!({
                    "figi": figi,
                    "quantity": quantity as i64,
                    "stopPrice": { "units": stop_units, "nano": stop_nano },
                    "direction": stop_direction,
                    "accountId": account_id,
                    "stopOrderType": "STOP_ORDER_TYPE_STOP_LOSS",
                });

                let response = self.post(TinkoffEndpoint::PostStopOrder, body).await?;
                let mut result = TinkoffParser::parse_stop_order_result(&response)?;
                result.symbol = ticker;
                result.stop_price = Some(stop_price);
                Ok(PlaceOrderResponse::Simple(result))
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                let account_id = self.account_id.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest(
                        "Account ID not set. Call initialize_account() first".to_string()
                    ))?;

                let ticker = format_ticker(&symbol);
                let figi = self.get_figi_by_ticker(&ticker).await?;
                let (stop_units, stop_nano) = price_to_quotation(stop_price);
                let (limit_units, limit_nano) = price_to_quotation(limit_price);

                let stop_direction = match side {
                    OrderSide::Buy => "STOP_ORDER_DIRECTION_BUY",
                    OrderSide::Sell => "STOP_ORDER_DIRECTION_SELL",
                };

                let body = serde_json::json!({
                    "figi": figi,
                    "quantity": quantity as i64,
                    "stopPrice": { "units": stop_units, "nano": stop_nano },
                    "price": { "units": limit_units, "nano": limit_nano },
                    "direction": stop_direction,
                    "accountId": account_id,
                    "stopOrderType": "STOP_ORDER_TYPE_STOP_LIMIT",
                });

                let response = self.post(TinkoffEndpoint::PostStopOrder, body).await?;
                let mut result = TinkoffParser::parse_stop_order_result(&response)?;
                result.symbol = ticker;
                result.stop_price = Some(stop_price);
                result.price = Some(limit_price);
                Ok(PlaceOrderResponse::Simple(result))
            }

            other => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on Tinkoff", other)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        use chrono::{Utc, TimeZone};

        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest("Account ID not set".to_string()))?;

        // Tinkoff GetOperations requires from/to timestamps
        let now = Utc::now();
        let from = filter.start_time
            .map(|ms| Utc.timestamp_millis_opt(ms).single().unwrap_or(now - chrono::Duration::days(7)))
            .unwrap_or_else(|| now - chrono::Duration::days(7));
        let to = filter.end_time
            .map(|ms| Utc.timestamp_millis_opt(ms).single().unwrap_or(now))
            .unwrap_or(now);

        let mut body = serde_json::json!({
            "accountId": account_id,
            "from": from.to_rfc3339(),
            "to": to.to_rfc3339(),
            // OPERATION_STATE_EXECUTED for filled, OPERATION_STATE_CANCELED for cancelled
            "state": "OPERATION_STATE_EXECUTED",
        });

        // Add FIGI filter if symbol is provided
        if let Some(ref sym) = filter.symbol {
            let ticker = format_ticker(sym);
            // Best effort: if we have a cached FIGI we'd use it, else skip filter
            // For now, include ticker as a hint — Tinkoff ignores unknown fields gracefully
            body["figi"] = serde_json::Value::String(ticker);
        }

        let response = self.post(TinkoffEndpoint::GetOperations, body).await?;
        let limit = filter.limit.unwrap_or(u32::MAX) as usize;
        TinkoffParser::parse_operations(&response, limit)
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let account_id = self.account_id.as_ref()
                .ok_or_else(|| ExchangeError::InvalidRequest("Account ID not set".to_string()))?;

            let body = serde_json::json!({
                "accountId": account_id,
                "orderId": order_id,
            });

            let response = self.post(TinkoffEndpoint::CancelOrder, body).await?;
            TinkoffParser::parse_order_result(&response)
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
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

        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest("Account ID not set".to_string()))?;

        let body = serde_json::json!({
            "accountId": account_id,
            "orderId": order_id,
        });

        let response = self.post(TinkoffEndpoint::GetOrderState, body).await?;
        TinkoffParser::parse_order_result(&response)
    
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

        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest("Account ID not set".to_string()))?;

        let body = serde_json::json!({
            "accountId": account_id,
        });

        let response = self.post(TinkoffEndpoint::GetOrders, body).await?;

        let orders = response
            .get("orders")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orders' array".to_string()))?;

        orders.iter()
            .map(TinkoffParser::parse_order_result)
            .collect()
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (Full support - Tinkoff is a broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for TinkoffConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let _account_type = query.account_type;

        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest("Account ID not set".to_string()))?;

        let body = serde_json::json!({
            "accountId": account_id,
        });

        let response = self.post(TinkoffEndpoint::GetPositions, body).await?;
        let mut balances = TinkoffParser::parse_balance(&response)?;

        // Filter by asset if provided
        if let Some(asset_filter) = asset {
            balances.retain(|b| b.asset == asset_filter);
        }

        Ok(balances)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0, // Tinkoff doesn't use maker/taker model
            taker_commission: 0.05, // Approximate commission rate (0.05%)
            balances: self.get_balance(BalanceQuery { asset: None, account_type }).await?,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        let response = self.post(TinkoffEndpoint::GetUserTariff, serde_json::json!({})).await?;
        TinkoffParser::parse_fee_info(&response, symbol)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (Partial support - stocks don't use funding rate)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for TinkoffConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest("Account ID not set".to_string()))?;

        let body = serde_json::json!({
            "accountId": account_id,
        });

        let response = self.post(TinkoffEndpoint::GetPortfolio, body).await?;
        let mut positions = TinkoffParser::parse_positions(&response)?;

        // Filter by symbol if provided
        if let Some(sym) = symbol {
            let ticker = format_ticker(&sym);
            positions.retain(|p| p.symbol == ticker);
        }

        Ok(positions)
    
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

        // Funding rate is not applicable for stock trading
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available - not applicable for stock market".to_string()
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::ClosePosition { symbol, account_type } => {
                // Close position by placing a counter-order for the full open quantity
                let positions = self.get_positions(PositionQuery {
                    symbol: Some(symbol.clone()),
                    account_type,
                }).await?;

                if positions.is_empty() {
                    return Err(ExchangeError::NotFound(
                        format!("No open position for {}", format_ticker(&symbol))
                    ));
                }

                let pos = &positions[0];
                let close_side = match pos.side {
                    PositionSide::Long => OrderSide::Sell,
                    PositionSide::Short => OrderSide::Buy,
                    PositionSide::Both => OrderSide::Sell,
                };

                let order_req = OrderRequest {
                    symbol: symbol.clone(),
                    side: close_side,
                    order_type: OrderType::Market,
                    quantity: pos.quantity,
                    time_in_force: TimeInForce::Gtc,
                    account_type,
                    client_order_id: None,
                    reduce_only: false,
                };

                self.place_order(order_req).await?;
                Ok(())
            }

            PositionModification::SetLeverage { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Leverage setting not available — not applicable for stock market".to_string()
                ))
            }

            other => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on Tinkoff", other)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPTIONAL TRAIT: AmendOrder (Tinkoff supports ReplaceOrder)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for TinkoffConnector {
    /// Amend a live order using Tinkoff's ReplaceOrder endpoint.
    ///
    /// Tinkoff ReplaceOrder supports changing quantity and/or price of a
    /// live limit order without cancel+replace. At least one of `fields.price`
    /// or `fields.quantity` must be `Some`.
    ///
    /// Note: Only limit orders can be amended on Tinkoff.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first".to_string()
            ))?;

        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "AmendRequest: at least one of price or quantity must be Some".to_string()
            ));
        }

        let mut body = serde_json::json!({
            "accountId": account_id,
            "orderId": req.order_id,
            // idempotencyKey is required by Tinkoff ReplaceOrder
            "idempotencyKey": uuid::Uuid::new_v4().to_string(),
        });

        if let Some(qty) = req.fields.quantity {
            body["quantity"] = serde_json::Value::Number(
                serde_json::Number::from(qty as i64)
            );
        }

        if let Some(price) = req.fields.price {
            let (units, nano) = price_to_quotation(price);
            body["price"] = serde_json::json!({ "units": units, "nano": nano });
        }

        let response = self.post(TinkoffEndpoint::ReplaceOrder, body).await?;
        let mut order = TinkoffParser::parse_order_result(&response)?;
        order.symbol = format_ticker(&req.symbol);
        Ok(order)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS — Quotation conversion
// ═══════════════════════════════════════════════════════════════════════════

/// Convert f64 price to Tinkoff Quotation (units: i64, nano: i32).
///
/// Tinkoff uses `Quotation { units: i64, nano: i32 }` for prices.
/// Example: 123.45 → (123, 450_000_000)
fn price_to_quotation(price: f64) -> (i64, i32) {
    let units = price.floor() as i64;
    let nano = ((price - units as f64) * 1_000_000_000.0).round() as i32;
    (units, nano)
}

// ═══════════════════════════════════════════════════════════════════════════
// gRPC SUPPORT
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "grpc")]
impl TinkoffConnector {
    // ─────────────────────────────────────────────────────────────────────
    // Builder
    // ─────────────────────────────────────────────────────────────────────

    /// Attach a gRPC channel, enabling direct gRPC calls to the Tinkoff API.
    ///
    /// Use [`crate::core::grpc::GrpcClient::connect`] to obtain a channel,
    /// connecting to [`tinkoff_proto::GRPC_ENDPOINT`] (`https://invest-public-api.tinkoff.ru:443`).
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "grpc")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use digdigdig3::core::grpc::GrpcClient;
    /// use digdigdig3::stocks::russia::tinkoff::TinkoffConnector;
    ///
    /// let grpc = GrpcClient::connect("https://invest-public-api.tinkoff.ru:443").await?;
    /// let connector = TinkoffConnector::from_env()
    ///     .with_grpc_channel(grpc.channel());
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_grpc_channel(mut self, channel: Channel) -> Self {
        self.grpc_channel = Some(channel);
        self
    }

    // ─────────────────────────────────────────────────────────────────────
    // Internal gRPC helper
    // ─────────────────────────────────────────────────────────────────────

    /// Require an attached gRPC channel or return `ExchangeError::Network`.
    fn require_channel(&self) -> ExchangeResult<Channel> {
        self.grpc_channel.clone().ok_or_else(|| {
            ExchangeError::Network(
                "No gRPC channel attached. Call TinkoffConnector::with_grpc_channel() \
                 with a channel connected to invest-public-api.tinkoff.ru:443."
                    .to_string(),
            )
        })
    }

    /// Build a tonic metadata value with the Bearer token for auth.
    fn bearer_metadata(&self) -> ExchangeResult<tonic::metadata::MetadataValue<tonic::metadata::Ascii>> {
        let val = format!("Bearer {}", self.auth.token);
        tonic::metadata::MetadataValue::try_from(val.as_str())
            .map_err(|e| ExchangeError::Auth(format!("Invalid token for gRPC metadata: {}", e)))
    }

    /// Perform a unary gRPC call with Bearer auth, returning the decoded response.
    async fn grpc_unary<Req, Resp>(
        &self,
        request: Req,
        path: &'static str,
    ) -> ExchangeResult<Resp>
    where
        Req: prost::Message + 'static,
        Resp: prost::Message + Default + 'static,
    {
        use tonic::client::Grpc;
        use tonic::IntoRequest;

        let channel = self.require_channel()?;
        let mut grpc: Grpc<Channel> = Grpc::new(channel);

        grpc.ready().await.map_err(|e| {
            ExchangeError::Network(format!("gRPC channel not ready: {}", e))
        })?;

        let path = tonic::codegen::http::uri::PathAndQuery::try_from(path)
            .map_err(|e| ExchangeError::Network(format!("Invalid gRPC path '{}': {}", path, e)))?;

        let codec: ProstCodec<Req, Resp> = ProstCodec::default();

        // Attach Bearer auth in request metadata.
        let mut req = request.into_request();
        let auth_value = self.bearer_metadata()?;
        req.metadata_mut().insert("authorization", auth_value);

        grpc.unary(req, path, codec)
            .await
            .map(|r| r.into_inner())
            .map_err(|status| {
                ExchangeError::Network(format!(
                    "gRPC call failed [{}]: {}",
                    status.code(),
                    status.message()
                ))
            })
    }

    // ─────────────────────────────────────────────────────────────────────
    // MarketDataService
    // ─────────────────────────────────────────────────────────────────────

    /// Fetch historical candles via gRPC `MarketDataService/GetCandles`.
    ///
    /// # Parameters
    /// - `figi`     — FIGI of the instrument (e.g. `"BBG004730N88"` for SBER)
    /// - `from_sec` — Range start as UTC epoch seconds
    /// - `to_sec`   — Range end as UTC epoch seconds
    /// - `interval` — [`tinkoff_proto::CandleInterval`] enum value (as i32)
    ///
    /// # Returns
    /// The raw [`GetCandlesResponse`] containing a `candles` vec of
    /// [`tinkoff_proto::HistoricCandle`].
    pub async fn get_candles_grpc(
        &self,
        figi: &str,
        from_sec: i64,
        to_sec: i64,
        interval: tinkoff_proto::CandleInterval,
    ) -> ExchangeResult<GetCandlesResponse> {
        let request = GetCandlesRequest {
            figi: figi.to_string(),
            from: Some(Timestamp { seconds: from_sec, nanos: 0 }),
            to: Some(Timestamp { seconds: to_sec, nanos: 0 }),
            interval: interval as i32,
            instrument_id: String::new(),
        };

        self.grpc_unary::<GetCandlesRequest, GetCandlesResponse>(
            request,
            paths::GET_CANDLES,
        )
        .await
    }

    /// Fetch order book snapshot via gRPC `MarketDataService/GetOrderBook`.
    ///
    /// # Parameters
    /// - `figi`  — FIGI of the instrument
    /// - `depth` — Order book depth: 1, 10, 20, 30, 40, or 50
    ///
    /// # Returns
    /// The raw [`GetOrderBookResponse`] containing bids, asks, and last price.
    pub async fn get_order_book_grpc(
        &self,
        figi: &str,
        depth: i32,
    ) -> ExchangeResult<GetOrderBookResponse> {
        let request = GetOrderBookRequest {
            figi: figi.to_string(),
            depth,
            instrument_id: String::new(),
        };

        self.grpc_unary::<GetOrderBookRequest, GetOrderBookResponse>(
            request,
            paths::GET_ORDER_BOOK,
        )
        .await
    }

    /// Fetch last trade prices via gRPC `MarketDataService/GetLastPrices`.
    ///
    /// # Parameters
    /// - `figis` — Slice of FIGI strings to query
    ///
    /// # Returns
    /// The raw [`GetLastPricesResponse`] containing a `last_prices` vec.
    pub async fn get_last_prices_grpc(
        &self,
        figis: &[&str],
    ) -> ExchangeResult<GetLastPricesResponse> {
        let request = GetLastPricesRequest {
            figi: figis.iter().map(|s| s.to_string()).collect(),
            instrument_id: Vec::new(),
        };

        self.grpc_unary::<GetLastPricesRequest, GetLastPricesResponse>(
            request,
            paths::GET_LAST_PRICES,
        )
        .await
    }

    /// Fetch trading status via gRPC `MarketDataService/GetTradingStatus`.
    ///
    /// # Parameters
    /// - `figi` — FIGI of the instrument
    ///
    /// # Returns
    /// The raw [`GetTradingStatusResponse`] with status flags.
    pub async fn get_trading_status_grpc(
        &self,
        figi: &str,
    ) -> ExchangeResult<GetTradingStatusResponse> {
        let request = GetTradingStatusRequest {
            figi: figi.to_string(),
            instrument_id: String::new(),
        };

        self.grpc_unary::<GetTradingStatusRequest, GetTradingStatusResponse>(
            request,
            paths::GET_TRADING_STATUS,
        )
        .await
    }

    // ─────────────────────────────────────────────────────────────────────
    // OrdersService
    // ─────────────────────────────────────────────────────────────────────

    /// Place an order via gRPC `OrdersService/PostOrder`.
    ///
    /// Requires `account_id` to be set (call [`Self::set_account_id`] or
    /// [`Self::initialize_account`] first).
    ///
    /// # Parameters
    /// - `figi`       — FIGI of the instrument
    /// - `quantity`   — Number of lots
    /// - `direction`  — [`tinkoff_proto::OrderDirection`] (Buy or Sell)
    /// - `order_type` — [`tinkoff_proto::OrderType`] (Limit or Market)
    /// - `limit_price`— Optional limit price; `None` for market orders
    /// - `order_id`   — Client idempotency key (UUID recommended)
    ///
    /// # Returns
    /// The raw [`PostOrderResponse`] with the server-assigned order ID and status.
    pub async fn place_order_grpc(
        &self,
        figi: &str,
        quantity: i64,
        direction: tinkoff_proto::OrderDirection,
        order_type: tinkoff_proto::OrderType,
        limit_price: Option<f64>,
        order_id: &str,
    ) -> ExchangeResult<PostOrderResponse> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first.".to_string()
            ))?;

        let price = limit_price.map(|p| {
            let units = p.floor() as i64;
            let nano = ((p - units as f64) * 1_000_000_000.0).round() as i32;
            tinkoff_proto::Quotation { units, nano }
        });

        let request = PostOrderRequest {
            figi: figi.to_string(),
            quantity,
            price,
            direction: direction as i32,
            account_id: account_id.clone(),
            order_type: order_type as i32,
            order_id: order_id.to_string(),
            instrument_id: String::new(),
        };

        self.grpc_unary::<PostOrderRequest, PostOrderResponse>(
            request,
            paths::POST_ORDER,
        )
        .await
    }

    /// Cancel an active order via gRPC `OrdersService/CancelOrder`.
    ///
    /// Requires `account_id` to be set.
    ///
    /// # Parameters
    /// - `order_id` — Server-assigned order ID to cancel
    ///
    /// # Returns
    /// The raw [`CancelOrderResponse`] with the cancellation timestamp.
    pub async fn cancel_order_grpc(
        &self,
        order_id: &str,
    ) -> ExchangeResult<CancelOrderResponse> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first.".to_string()
            ))?;

        let request = CancelOrderRequest {
            account_id: account_id.clone(),
            order_id: order_id.to_string(),
        };

        self.grpc_unary::<CancelOrderRequest, CancelOrderResponse>(
            request,
            paths::CANCEL_ORDER,
        )
        .await
    }

    /// Fetch active orders via gRPC `OrdersService/GetOrders`.
    ///
    /// Requires `account_id` to be set.
    ///
    /// # Returns
    /// The raw [`GetOrdersResponse`] with a vec of active [`OrderState`]s.
    pub async fn get_orders_grpc(&self) -> ExchangeResult<GetOrdersResponse> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first.".to_string()
            ))?;

        let request = GetOrdersRequest {
            account_id: account_id.clone(),
        };

        self.grpc_unary::<GetOrdersRequest, GetOrdersResponse>(
            request,
            paths::GET_ORDERS,
        )
        .await
    }

    /// Fetch a single order state via gRPC `OrdersService/GetOrderState`.
    ///
    /// Requires `account_id` to be set.
    ///
    /// # Parameters
    /// - `order_id` — Server-assigned order ID to query
    ///
    /// # Returns
    /// The raw [`OrderState`] for the requested order.
    pub async fn get_order_state_grpc(
        &self,
        order_id: &str,
    ) -> ExchangeResult<OrderState> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first.".to_string()
            ))?;

        let request = GetOrderStateRequest {
            account_id: account_id.clone(),
            order_id: order_id.to_string(),
        };

        self.grpc_unary::<GetOrderStateRequest, OrderState>(
            request,
            paths::GET_ORDER_STATE,
        )
        .await
    }

    // ─────────────────────────────────────────────────────────────────────
    // OperationsService
    // ─────────────────────────────────────────────────────────────────────

    /// Fetch the full portfolio via gRPC `OperationsService/GetPortfolio`.
    ///
    /// Requires `account_id` to be set.
    ///
    /// # Returns
    /// The raw [`PortfolioResponse`] with totals and per-position data.
    pub async fn get_portfolio_grpc(&self) -> ExchangeResult<PortfolioResponse> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first.".to_string()
            ))?;

        let request = PortfolioRequest {
            account_id: account_id.clone(),
        };

        self.grpc_unary::<PortfolioRequest, PortfolioResponse>(
            request,
            paths::GET_PORTFOLIO,
        )
        .await
    }

    /// Fetch current positions via gRPC `OperationsService/GetPositions`.
    ///
    /// Requires `account_id` to be set.
    ///
    /// # Returns
    /// The raw [`PositionsResponse`] with cash, securities, and futures positions.
    pub async fn get_positions_grpc(&self) -> ExchangeResult<PositionsResponse> {
        let account_id = self.account_id.as_ref()
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Account ID not set. Call initialize_account() first.".to_string()
            ))?;

        let request = PositionsRequest {
            account_id: account_id.clone(),
        };

        self.grpc_unary::<PositionsRequest, PositionsResponse>(
            request,
            paths::GET_POSITIONS,
        )
        .await
    }
}
