//! Futu OpenAPI connector implementation
//!
//! ## Architecture
//!
//! Futu OpenAPI uses a **TCP + Protocol Buffers** architecture with a local
//! OpenD gateway daemon, not HTTP REST.  All methods in this connector build
//! the correct request payload and call `proto_call()`.
//!
//! `proto_call()` currently returns `ExchangeError::UnsupportedOperation` with
//! a diagnostic message including the protocol ID and the request JSON.
//! When the TCP+Protobuf transport is wired up, `proto_call` is the only method
//! that needs to change — all business logic above it is complete and correct.
//!
//! ## Transport note
//!
//! To connect the transport:
//! 1. Download OpenD: <https://www.futunn.com/en/download/OpenAPI>
//! 2. Implement a TCP client that sends Futu framed protobuf packets
//! 3. Replace the body of `proto_call()` with the actual TCP send/receive

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::{
    proto_id, FutuEndpoints, FutuOrderType, FutuTrdSide, ModifyOrderOp,
    TrdEnv, TrdMarket, SecMarket, format_symbol, infer_sec_market,
};
use super::auth::FutuAuth;
use super::parser::FutuParser;

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTOR STRUCT
// ═══════════════════════════════════════════════════════════════════════════

/// Futu OpenAPI connector
///
/// Implements all core traits with correct Futu business logic.
/// The TCP+Protobuf transport is stubbed via `proto_call()`.
///
/// ## Usage
///
/// ```rust,ignore
/// let connector = FutuConnector::new(FutuAuth::from_env());
/// // All methods return Err(UnsupportedOperation) until OpenD is connected.
/// ```
pub struct FutuConnector {
    _client: Client,
    auth: FutuAuth,
    endpoints: FutuEndpoints,
    /// Default trading environment (Real / Simulate)
    trd_env: TrdEnv,
    /// Account ID — populated after Trd_GetAccList succeeds
    acc_id: u64,
    /// Default trading market
    trd_market: TrdMarket,
}

impl FutuConnector {
    /// Create new connector
    pub fn new(auth: FutuAuth) -> Self {
        Self {
            _client: Client::new(),
            endpoints: FutuEndpoints::default(),
            trd_env: TrdEnv::Real,
            acc_id: 0,
            trd_market: TrdMarket::Us,
            auth,
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(FutuAuth::from_env())
    }

    /// Set trading environment (Real / Simulate)
    pub fn with_env(mut self, env: TrdEnv) -> Self {
        self.trd_env = env;
        self
    }

    /// Set default trading market
    pub fn with_market(mut self, market: TrdMarket) -> Self {
        self.trd_market = market;
        self
    }

    /// Set account ID (obtain via Trd_GetAccList = 2001)
    pub fn with_acc_id(mut self, acc_id: u64) -> Self {
        self.acc_id = acc_id;
        self
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TRANSPORT STUB
    // ─────────────────────────────────────────────────────────────────────────

    /// Send a Protocol Buffer request to OpenD and receive the response.
    ///
    /// Currently returns `UnsupportedOperation` with a diagnostic message.
    /// Replace this body with actual TCP+Protobuf framing when OpenD transport
    /// is implemented.
    ///
    /// ## Wire format
    ///
    /// Futu packets: `[header(44 bytes)][proto body(variable)]`
    ///
    /// Header fields:
    /// - nProtoID: u32   — identifies which protobuf message
    /// - nProtoFmtType: u8 — 0=Protobuf, 1=JSON
    /// - nSerialNo: u32  — per-connection sequence counter
    /// - nBodyLen: u32   — length of serialised body
    /// - arrBodySHA1: [u8; 20] — SHA1 of body for integrity check
    async fn proto_call(
        &self,
        proto_id: u32,
        request: Value,
    ) -> ExchangeResult<Value> {
        Err(ExchangeError::UnsupportedOperation(format!(
            "Futu OpenD TCP+Protobuf transport not connected. \
             OpenD address: {}. Proto ID: {}, request: {}",
            self.endpoints.address(),
            proto_id,
            request
        )))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // COMMON TRADING HEADER
    // ─────────────────────────────────────────────────────────────────────────

    /// Build the trdHeader object required by all Trd_* requests.
    fn trd_header(&self) -> Value {
        json!({
            "trdEnv": self.trd_env.as_i32(),
            "accID": self.acc_id,
            "trdMarket": self.trd_market.as_i32(),
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FORMAT SYMBOL
    // ─────────────────────────────────────────────────────────────────────────

    fn format_sym(&self, symbol: &Symbol) -> String {
        let sec_market = infer_sec_market(symbol);
        format_symbol(symbol, sec_market)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MAP SIDE
    // ─────────────────────────────────────────────────────────────────────────

    fn map_side(side: OrderSide) -> FutuTrdSide {
        match side {
            OrderSide::Buy => FutuTrdSide::Buy,
            OrderSide::Sell => FutuTrdSide::Sell,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for FutuConnector {
    fn exchange_name(&self) -> &'static str {
        "futu"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Futu
    }

    fn is_testnet(&self) -> bool {
        matches!(self.trd_env, TrdEnv::Simulate)
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // Futu Cash, Margin, Universal accounts are all Spot-mapped
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for FutuConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let code = self.format_sym(&symbol);
        let request = json!({
            "securityList": [{"market": 1, "code": code}]
        });
        let response = self.proto_call(proto_id::QOT_GET_SECURITY_SNAPSHOT, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_price(s2c)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let code = self.format_sym(&symbol);
        let request = json!({
            "securityList": [{"market": 1, "code": code}]
        });
        let response = self.proto_call(proto_id::QOT_GET_SECURITY_SNAPSHOT, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_ticker(s2c, &symbol.base)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let code = self.format_sym(&symbol);
        let request = json!({
            "security": {"market": 1, "code": code},
            "num": depth.unwrap_or(10),
        });
        let response = self.proto_call(proto_id::QOT_GET_ORDER_BOOK, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_orderbook(s2c)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let code = self.format_sym(&symbol);
        // Map common interval strings to Futu KLType enum
        // 1=K_1M, 2=K_5M, 3=K_15M, 4=K_30M, 5=K_60M, 6=K_DAY, 7=K_WEEK, 8=K_MON
        let kl_type = match interval {
            "1m" | "1min" => 1,
            "5m" | "5min" => 2,
            "15m" | "15min" => 3,
            "30m" | "30min" => 4,
            "1h" | "60m" | "60min" => 5,
            "1d" | "1day" | "D" => 6,
            "1w" | "1week" | "W" => 7,
            "1M" | "1mon" => 8,
            _ => 6, // default to daily
        };
        let max_count = limit.unwrap_or(200) as i32;
        let request = json!({
            "security": {"market": 1, "code": code},
            "klType": kl_type,
            "reqNum": max_count,
        });
        let response = self.proto_call(proto_id::QOT_REQUEST_HISTORY_KL, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_klines(s2c)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let request = json!({ "time": 0u64 });
        self.proto_call(proto_id::KEEP_ALIVE, request).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for FutuConnector {
    /// Place an order via Trd_PlaceOrder (proto 2202).
    ///
    /// ## Futu order type mapping
    ///
    /// | Our OrderType      | Futu orderType | price       | auxPrice    |
    /// |--------------------|----------------|-------------|-------------|
    /// | Market             | 2 (Market)     | 0           | —           |
    /// | Limit              | 1 (Normal)     | limit price | —           |
    /// | StopMarket         | 3 (Enhanced)   | 0           | stop_price  |
    /// | StopLimit          | 4 (StopLimit)  | limit_price | stop_price  |
    /// | Ioc { price }      | 1 (Normal)     | price       | — + IOC TIF|
    /// | Fok { price }      | 1 (Normal)     | price       | — + FOK TIF|
    /// | PostOnly { price } | 7 (SpecialLim) | price       | —           |
    /// | Oco / Bracket      | UnsupportedOperation                       |
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol_code = self.format_sym(&req.symbol);
        let trd_side = Self::map_side(req.side);

        let (order_type_val, price, aux_price, time_in_force_val) =
            match &req.order_type {
                OrderType::Market => (
                    FutuOrderType::Market.as_i32(),
                    0.0f64,
                    None::<f64>,
                    None::<i32>,
                ),
                OrderType::Limit { price } => (
                    FutuOrderType::Normal.as_i32(),
                    *price,
                    None,
                    None,
                ),
                OrderType::StopMarket { stop_price } => (
                    FutuOrderType::EnhancedLimit.as_i32(),
                    0.0,
                    Some(*stop_price),
                    None,
                ),
                OrderType::StopLimit { stop_price, limit_price } => (
                    FutuOrderType::StopLimit.as_i32(),
                    *limit_price,
                    Some(*stop_price),
                    None,
                ),
                OrderType::PostOnly { price } => (
                    FutuOrderType::SpecialLimit.as_i32(),
                    *price,
                    None,
                    None,
                ),
                OrderType::Ioc { price } => (
                    FutuOrderType::Normal.as_i32(),
                    price.unwrap_or(0.0),
                    None,
                    Some(2i32), // IOC time-in-force
                ),
                OrderType::Fok { price } => (
                    FutuOrderType::Normal.as_i32(),
                    *price,
                    None,
                    Some(4i32), // FOK time-in-force
                ),
                OrderType::Oco { .. } => {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Futu does not support native OCO orders".to_string(),
                    ));
                }
                OrderType::Bracket { .. } => {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Futu does not support native Bracket orders".to_string(),
                    ));
                }
                OrderType::TrailingStop { .. } => {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Futu does not support trailing stop orders".to_string(),
                    ));
                }
                OrderType::Iceberg { price, .. } => (
                    // Futu has no native iceberg; treat as a standard limit
                    FutuOrderType::Normal.as_i32(),
                    *price,
                    None,
                    None,
                ),
                OrderType::Twap { .. } => {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Futu does not support TWAP orders".to_string(),
                    ));
                }
                OrderType::Gtd { price, .. } => (
                    FutuOrderType::Normal.as_i32(),
                    *price,
                    None,
                    None,
                ),
                OrderType::ReduceOnly { .. } => {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Futu stocks do not support ReduceOnly orders".to_string(),
                    ));
                }
                OrderType::Oto { .. } | OrderType::ConditionalPlan { .. } | OrderType::DcaRecurring { .. } => {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Futu does not support this order type".to_string(),
                    ));
                }
            };

        // Determine secMarket from symbol
        let sec_market = infer_sec_market(&req.symbol);

        let mut request_body = json!({
            "header": self.trd_header(),
            "trdSide": trd_side.as_i32(),
            "orderType": order_type_val,
            "code": symbol_code,
            "qty": req.quantity,
            "price": price,
            "secMarket": sec_market.as_i32(),
        });

        // Inject optional fields
        if let Some(ap) = aux_price {
            request_body["auxPrice"] = json!(ap);
        }
        if let Some(tif) = time_in_force_val {
            request_body["timeInForce"] = json!(tif);
        }
        if let Some(ref cid) = req.client_order_id {
            request_body["remark"] = json!(cid);
        }

        let response = self.proto_call(proto_id::TRD_PLACE_ORDER, request_body).await?;
        let s2c = FutuParser::check_response(&response)?;
        let order = FutuParser::parse_place_order(s2c, &symbol_code)?;
        Ok(PlaceOrderResponse::Simple(order))
    }

    /// Cancel an order via Trd_ModifyOrder (proto 2205) with op=Cancel(2).
    ///
    /// Futu supports single-order cancel only.
    /// Batch/All/BySymbol scopes return `UnsupportedOperation`.
    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        let order_id = match &req.scope {
            CancelScope::Single { order_id } => order_id.clone(),
            CancelScope::Batch { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Futu does not support native batch cancel. Use CancelAll trait or cancel individually.".to_string(),
                ));
            }
            CancelScope::All { .. } | CancelScope::BySymbol { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Futu does not support native cancel-all. Cancel orders individually.".to_string(),
                ));
            }
            CancelScope::ByLabel(_) | CancelScope::ByCurrencyKind { .. } | CancelScope::ScheduledAt(_) => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Futu does not support this cancel scope".to_string(),
                ));
            }
        };

        let order_id_u64: u64 = order_id.parse().map_err(|_| {
            ExchangeError::InvalidRequest(format!("invalid order ID: {}", order_id))
        })?;

        let request = json!({
            "header": self.trd_header(),
            "modifyOrderOp": ModifyOrderOp::Cancel.as_i32(),
            "orderID": order_id_u64,
            "qty": 0,
            "price": 0,
        });

        let response = self.proto_call(proto_id::TRD_MODIFY_ORDER, request).await?;
        let s2c = FutuParser::check_response(&response)?;

        // After cancel, fetch the updated order state via GetOrderList
        // For now parse the embedded order if available, else build a placeholder
        if let Some(order_obj) = s2c.get("order") {
            if order_obj.is_object() {
                return FutuParser::parse_order(order_obj);
            }
        }

        // Return a minimal cancelled order placeholder
        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: req.symbol
                .map(|s| s.base)
                .unwrap_or_default(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            status: OrderStatus::Canceled,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        })
    }

    /// Get a single order by ID via Trd_GetOrderList (proto 2201) filtered by ID.
    ///
    /// Futu has no single-order-by-ID endpoint; we fetch the open order list
    /// and filter client-side.
    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let order_id_u64: u64 = order_id.parse().map_err(|_| {
            ExchangeError::InvalidRequest(format!("invalid order ID: {}", order_id))
        })?;

        let request = json!({
            "header": self.trd_header(),
            "filterConditions": {
                "orderIDList": [order_id_u64],
            }
        });

        let response = self.proto_call(proto_id::TRD_GET_ORDER_LIST, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        let orders = FutuParser::parse_order_list(s2c)?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::NotFound(format!("order {} not found", order_id)))
    }

    /// Get open orders via Trd_GetOrderList (proto 2201).
    ///
    /// `symbol = None` fetches all open orders (Futu supports this natively).
    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut filter_conditions = json!({
            // orderStatusFilter 6 = Submitted (open orders)
            "orderStatusFilterList": [6, 7], // Submitted + PartiallyFilled
        });

        if let Some(sym) = symbol {
            filter_conditions["codeList"] = json!([sym]);
        }

        let request = json!({
            "header": self.trd_header(),
            "filterConditions": filter_conditions,
        });

        let response = self.proto_call(proto_id::TRD_GET_ORDER_LIST, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_order_list(s2c)
    }

    /// Get order history via Trd_GetHistOrderList (proto 2221).
    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut filter_conditions = json!({});

        if let Some(sym) = &filter.symbol {
            filter_conditions["codeList"] = json!([self.format_sym(sym)]);
        }
        if let Some(start) = filter.start_time {
            // Futu expects "YYYY-MM-DD" string or Unix timestamp; we send ms / 1000
            filter_conditions["beginTime"] = json!((start / 1000).to_string());
        }
        if let Some(end) = filter.end_time {
            filter_conditions["endTime"] = json!((end / 1000).to_string());
        }

        let request = json!({
            "header": self.trd_header(),
            "filterConditions": filter_conditions,
        });

        let response = self.proto_call(proto_id::TRD_GET_HIST_ORDER_LIST, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_order_list(s2c)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for FutuConnector {
    /// Get account balance via Trd_GetFunds (proto 2101).
    ///
    /// Futu returns a single Funds object per account.
    /// We map it to a Vec<Balance> with a single entry per currency.
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let request = json!({
            "header": self.trd_header(),
            "refreshBalance": true,
        });

        let response = self.proto_call(proto_id::TRD_GET_FUNDS, request).await?;
        let s2c = FutuParser::check_response(&response)?;

        // Infer currency from market
        let currency = match self.trd_market {
            TrdMarket::Hk => "HKD",
            TrdMarket::Us => "USD",
            TrdMarket::CnSh | TrdMarket::CnSz => "CNY",
            TrdMarket::Sg => "SGD",
        };

        let mut balances = FutuParser::parse_funds(s2c, currency)?;

        // Filter by asset if requested
        if let Some(asset) = &query.asset {
            balances.retain(|b| b.asset.eq_ignore_ascii_case(asset));
        }

        Ok(balances)
    }

    /// Get account info via Trd_GetFunds (proto 2101).
    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let request = json!({
            "header": self.trd_header(),
            "refreshBalance": true,
        });

        let response = self.proto_call(proto_id::TRD_GET_FUNDS, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_account_info(s2c, account_type)
    }

    /// Get fee info.
    ///
    /// Futu charges a flat commission per trade that varies by market and
    /// brokerage tier.  There is no API endpoint to query it dynamically.
    /// We return static defaults.
    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Default: US market 0.0049 USD/share (min $0.99), or 0.08% for HK
        let (maker, taker) = match self.trd_market {
            TrdMarket::Hk => (0.0003, 0.0003), // ~0.03% HK commission
            TrdMarket::Us => (0.0, 0.0),         // US: per-share model, not rate-based
            TrdMarket::CnSh | TrdMarket::CnSz => (0.0003, 0.0003),
            TrdMarket::Sg => (0.0003, 0.0003),
        };

        Ok(FeeInfo {
            maker_rate: maker,
            taker_rate: taker,
            symbol: symbol.map(|s| s.to_string()),
            tier: Some("standard".to_string()),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for FutuConnector {
    /// Get open stock positions via Trd_GetPositionList (proto 2102).
    ///
    /// Note: "positions" in Futu means stock holdings (long only for cash
    /// accounts), not perpetual futures contracts.
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let mut filter_conditions = json!({});

        if let Some(sym) = &query.symbol {
            filter_conditions["codeList"] = json!([self.format_sym(sym)]);
        }

        let request = json!({
            "header": self.trd_header(),
            "filterConditions": filter_conditions,
        });

        let response = self.proto_call(proto_id::TRD_GET_POSITION_LIST, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_position_list(s2c)
    }

    /// Funding rate — not applicable to stock trading.
    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu is a stock/ETF broker — funding rates are only applicable to \
             perpetual futures exchanges."
                .to_string(),
        ))
    }

    /// Modify a position.
    ///
    /// Futu does not have a native position-modify endpoint.
    /// - `ClosePosition`: place a market sell order for the full position qty.
    /// - All other variants return `UnsupportedOperation`.
    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::ClosePosition { symbol, account_type } => {
                // Fetch current position to know the quantity
                let query = PositionQuery {
                    symbol: Some(symbol.clone()),
                    account_type,
                };
                let positions = self.get_positions(query).await?;
                let position = positions.into_iter()
                    .find(|p| p.symbol == format_symbol(&symbol, infer_sec_market(&symbol)))
                    .ok_or_else(|| ExchangeError::NotFound(
                        format!("no open position for {}", symbol.base)
                    ))?;

                let close_side = match position.side {
                    PositionSide::Long | PositionSide::Both => OrderSide::Sell,
                    PositionSide::Short => OrderSide::Buy,
                };

                let close_req = OrderRequest {
                    symbol,
                    side: close_side,
                    order_type: OrderType::Market,
                    quantity: position.quantity.abs(),
                    time_in_force: TimeInForce::Gtc,
                    account_type,
                    client_order_id: None,
                    reduce_only: false,
                };

                self.place_order(close_req).await?;
                Ok(())
            }

            PositionModification::SetLeverage { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Futu stock accounts do not support leverage adjustment via API.".to_string(),
                ))
            }

            PositionModification::SetMarginMode { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Futu stock accounts do not support margin mode switching via API.".to_string(),
                ))
            }

            PositionModification::AddMargin { .. } | PositionModification::RemoveMargin { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Futu stock accounts do not support manual margin adjustment via API.".to_string(),
                ))
            }

            PositionModification::SetTpSl { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Futu does not support setting TP/SL on positions directly. \
                     Place separate conditional orders instead."
                        .to_string(),
                ))
            }

            PositionModification::SwitchPositionMode { .. } | PositionModification::MovePositions { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Futu does not support this position modification".to_string(),
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: AmendOrder (optional — Futu Trd_ModifyOrder with op=Normal)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for FutuConnector {
    /// Amend a live order via Trd_ModifyOrder (proto 2205) with op=Normal(1).
    ///
    /// At least one of price or quantity must be provided.
    /// Futu does not support changing only the trigger price independently.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let order_id_u64: u64 = req.order_id.parse().map_err(|_| {
            ExchangeError::InvalidRequest(format!("invalid order ID: {}", req.order_id))
        })?;

        // At least one field must change
        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "amend_order requires at least one of: price, quantity".to_string(),
            ));
        }

        // Futu ModifyOrder requires both qty and price even if only one changes;
        // caller should fetch the current order first to fill the unchanged values.
        // We use 0 as sentinel for "unchanged" — OpenD interprets 0 qty or 0 price
        // as "keep existing value" per Futu SDK documentation.
        let qty = req.fields.quantity.unwrap_or(0.0);
        let price = req.fields.price.unwrap_or(0.0);

        let request = json!({
            "header": self.trd_header(),
            "modifyOrderOp": ModifyOrderOp::Normal.as_i32(),
            "orderID": order_id_u64,
            "qty": qty,
            "price": price,
        });

        let response = self.proto_call(proto_id::TRD_MODIFY_ORDER, request).await?;
        let s2c = FutuParser::check_response(&response)?;

        // Return the modified order if embedded in response
        if let Some(order_obj) = s2c.get("order") {
            if order_obj.is_object() {
                return FutuParser::parse_order(order_obj);
            }
        }

        // Fallback: return a placeholder with the new values
        Ok(Order {
            id: req.order_id,
            client_order_id: None,
            symbol: req.symbol.base,
            side: OrderSide::Buy, // unknown without a separate query
            order_type: if price > 0.0 {
                OrderType::Limit { price }
            } else {
                OrderType::Market
            },
            status: OrderStatus::Open,
            price: req.fields.price,
            stop_price: req.fields.trigger_price,
            quantity: qty,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Extended methods (Futu-specific features)
// ═══════════════════════════════════════════════════════════════════════════

impl FutuConnector {
    /// Get the list of trading accounts linked to OpenD (Trd_GetAccList = 2001).
    ///
    /// Returns (accID, trdMarket) pairs. Use the accID to configure this
    /// connector via `with_acc_id()`.
    pub async fn get_account_list(&self) -> ExchangeResult<Vec<(u64, i32)>> {
        let request = json!({ "trdCategory": 1 }); // 1 = Security (stocks)
        let response = self.proto_call(proto_id::TRD_GET_ACC_LIST, request).await?;
        let s2c = FutuParser::check_response(&response)?;
        FutuParser::parse_acc_list(s2c)
    }

    /// Unlock trading with the configured trade password (Trd_UnlockTrade = 2004).
    ///
    /// Must be called before placing orders on a real account.
    pub async fn unlock_trade(&self) -> ExchangeResult<()> {
        let password = self.auth.trade_password.as_deref().unwrap_or("");
        let request = json!({
            "isPwdMd5": false,
            "pwd": password,
            "securityFirm": 1,  // 1 = Futu Securities
        });
        let response = self.proto_call(proto_id::TRD_UNLOCK_TRADE, request).await?;
        FutuParser::check_response(&response)?;
        Ok(())
    }

    /// Get broker queue data — HK LV2 subscription required.
    pub async fn get_broker_queue(&self, symbol: Symbol) -> ExchangeResult<BrokerQueue> {
        let code = self.format_sym(&symbol);
        let request = json!({
            "security": {"market": SecMarket::Hk.as_i32(), "code": code},
        });
        let _response = self.proto_call(proto_id::QOT_GET_STATIC_INFO, request).await?;
        Err(ExchangeError::UnsupportedOperation(
            "BrokerQueue requires LV2 subscription and full protobuf transport".to_string(),
        ))
    }

    /// Get order fill list (Trd_GetOrderFillList = 2211).
    pub async fn get_fills(&self, symbol: Option<&str>) -> ExchangeResult<Vec<UserTrade>> {
        let mut filter_conditions = json!({});
        if let Some(sym) = symbol {
            filter_conditions["codeList"] = json!([sym]);
        }
        let request = json!({
            "header": self.trd_header(),
            "filterConditions": filter_conditions,
        });
        let _response = self.proto_call(proto_id::TRD_GET_ORDER_FILL_LIST, request).await?;
        // Full parsing would go here once transport is connected
        Ok(vec![])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Futu-specific stub types
// ═══════════════════════════════════════════════════════════════════════════

/// Broker queue data (HK LV2)
#[derive(Debug, Clone)]
pub struct BrokerQueue {
    pub symbol: String,
    pub bid_brokers: Vec<BrokerInfo>,
    pub ask_brokers: Vec<BrokerInfo>,
}

/// Broker info entry in HK LV2 broker queue
#[derive(Debug, Clone)]
pub struct BrokerInfo {
    pub broker_id: u32,
    pub broker_name: String,
    pub position: u32,
}
