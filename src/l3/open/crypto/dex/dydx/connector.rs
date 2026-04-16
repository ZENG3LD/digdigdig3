//! # dYdX v4 Connector
//!
//! Реализация всех core трейтов для dYdX v4 Indexer API.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные (read-only via Indexer)
//! - `Account` - информация об аккаунте (read-only via Indexer)
//! - `Positions` - perpetual futures позиции (read-only via Indexer)
//!
//! ## Limitations
//! - Текущая реализация: только Indexer API (read-only)
//! - Trading операции требуют Node API (gRPC) - будущая реализация

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::OnceCell;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook, Balance, AccountInfo,
    Position, FundingRate,
    Order, OrderRequest, CancelRequest,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    UserTrade, UserTradeFilter,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    FundingHistory,
};
use crate::core::utils::SimpleRateLimiter;
use crate::core::types::{
    ConnectorStats, SymbolInfo, FundingPayment, FundingFilter,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};

use super::endpoints::{DydxUrls, DydxEndpoint, format_symbol, map_kline_interval};
use super::auth::DydxAuth;
use super::parser::DydxParser;

#[cfg(feature = "grpc")]
use super::proto::dydxprotocol::{
    BroadcastTxRequest, BroadcastTxResponse,
    BroadcastMode,
};
#[cfg(feature = "grpc")]
use tonic::codec::ProstCodec;
#[cfg(feature = "grpc")]
use tonic::transport::Channel;

#[cfg(feature = "onchain-cosmos")]
use crate::core::chain::cosmos::CosmosProvider;

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-market configuration fetched from `GET /v4/perpetualMarkets`.
///
/// Used to correctly encode `quantums` (size) and `subticks` (price) for
/// on-chain order placement. Values are market-specific and must not be
/// hard-coded.
#[derive(Debug, Clone)]
pub struct MarketConfig {
    /// dYdX market ticker, e.g. `"BTC-USD"`.
    pub ticker: String,
    /// CLOB pair ID (0 = BTC-USD, 1 = ETH-USD, …).
    pub clob_pair_id: u32,
    /// Minimum size increment in base quantums.
    ///
    /// `quantums = round(size / step_base_quantums)` — must not be zero.
    pub step_base_quantums: f64,
    /// Price increment: `subticks = round(price * subticks_per_tick)`.
    pub subticks_per_tick: f64,
    /// Exponent used to convert quantums to base units:
    /// `size_in_base = quantums * 10^quantum_conversion_exponent`.
    pub quantum_conversion_exponent: i32,
    /// Atomic resolution: decimal places of the base asset on the chain.
    pub atomic_resolution: i32,
}

/// Thread-safe lazy cache of all perpetual market configs.
///
/// Populated on first call to `fetch_market_configs()` and reused thereafter.
type MarketConfigCache = OnceCell<HashMap<String, MarketConfig>>;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// dYdX v4 коннектор
pub struct DydxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (не используется для Indexer API)
    auth: DydxAuth,
    /// URL'ы (mainnet/testnet)
    urls: DydxUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (conservative guard: 100 req/10s)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Optional gRPC channel to a dYdX validator node.
    ///
    /// When present, `place_order` and `cancel_order` broadcast signed
    /// `TxRaw` bytes via `cosmos.tx.v1beta1.Service/BroadcastTx`.
    /// Absent by default — the connector operates in read-only REST mode.
    #[cfg(feature = "grpc")]
    grpc_channel: Option<Channel>,
    /// Optional Cosmos chain provider.
    ///
    /// When both `onchain-cosmos` and `grpc` features are active and a
    /// `CosmosProvider` is attached via [`Self::with_cosmos_provider`],
    /// `place_order` and `cancel_order` automatically build and broadcast
    /// signed Cosmos SDK transactions using the tx builder.
    ///
    /// The provider manages sequence numbers to prevent nonce collisions
    /// across concurrent order calls.
    #[cfg(feature = "onchain-cosmos")]
    cosmos_provider: Option<Arc<CosmosProvider>>,
    /// Lazy-loaded cache of perpetual market configurations.
    ///
    /// Populated on first call to [`Self::fetch_market_configs`] by hitting
    /// `GET /v4/perpetualMarkets`. Subsequent calls return the cached value
    /// without a network round-trip.
    market_config_cache: Arc<MarketConfigCache>,
}

impl DydxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            DydxUrls::TESTNET
        } else {
            DydxUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = DydxAuth::new(credentials.as_ref())?;

        // Conservative guard: 100 requests per 10 seconds
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            #[cfg(feature = "grpc")]
            grpc_channel: None,
            #[cfg(feature = "onchain-cosmos")]
            cosmos_provider: None,
            market_config_cache: Arc::new(OnceCell::new()),
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
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
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

    /// GET запрос к Indexer API
    async fn get(
        &self,
        endpoint: DydxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.indexer_rest;
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in &params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        // Build query string from remaining params
        let mut query_params: Vec<String> = Vec::new();
        for (key, value) in &params {
            if !path.contains(value) {
                query_params.push(format!("{}={}", key, value));
            }
        }

        let query = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);
        let headers = self.auth.sign_request("GET", &path, "");

        self.http.get_with_headers(&url, &HashMap::new(), &headers).await
    }

    /// Извлечь data field или вернуть весь response
    fn _unwrap_response(&self, response: Value) -> Value {
        response
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // gRPC CHANNEL BUILDER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Attach a gRPC channel to this connector, enabling order placement and
    /// cancellation via the dYdX validator node.
    ///
    /// Call [`crate::core::grpc::GrpcClient::connect`] to obtain a channel,
    /// then pass `grpc_client.channel()` here.
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "grpc")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use digdigdig3::core::grpc::GrpcClient;
    /// use digdigdig3::crypto::dex::dydx::DydxConnector;
    ///
    /// let grpc = GrpcClient::connect("https://dydx-ops-grpc.kingnodes.com:443").await?;
    /// let connector = DydxConnector::public(false).await?
    ///     .with_grpc_channel(grpc.channel());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "grpc")]
    pub fn with_grpc_channel(mut self, channel: Channel) -> Self {
        self.grpc_channel = Some(channel);
        self
    }

    /// Attach a [`CosmosProvider`] to enable automatic tx building.
    ///
    /// When both `onchain-cosmos` and `grpc` features are enabled and a
    /// `CosmosProvider` is attached, calling `place_order` / `cancel_order`
    /// with a properly configured connector will automatically build and sign
    /// the Cosmos SDK transaction using `tx_builder`.
    ///
    /// The provider should also be connected to a gRPC channel via
    /// [`Self::with_grpc_channel`] for broadcasting. Typically you would
    /// call both:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::sync::Arc;
    /// use digdigdig3::core::chain::cosmos::CosmosProvider;
    /// use digdigdig3::core::grpc::GrpcClient;
    /// use digdigdig3::crypto::dex::dydx::DydxConnector;
    ///
    /// let cosmos = Arc::new(CosmosProvider::dydx_mainnet());
    /// let grpc = GrpcClient::connect("https://dydx-ops-grpc.kingnodes.com:443").await?;
    /// let connector = DydxConnector::public(false).await?
    ///     .with_cosmos_provider(cosmos)
    ///     .with_grpc_channel(grpc.channel());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "onchain-cosmos")]
    pub fn with_cosmos_provider(mut self, provider: Arc<CosmosProvider>) -> Self {
        self.cosmos_provider = Some(provider);
        self
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // gRPC HELPERS — PLACE / CANCEL
    // ═══════════════════════════════════════════════════════════════════════════

    /// Broadcast a pre-signed `TxRaw` that wraps a `MsgPlaceOrder` to the
    /// dYdX validator node via `cosmos.tx.v1beta1.Service/BroadcastTx`.
    ///
    /// ## Parameters
    ///
    /// - `tx_raw_bytes` — protobuf-serialised `TxRaw` (body_bytes +
    ///   auth_info_bytes + signatures).  The caller is responsible for
    ///   constructing and signing the Cosmos SDK transaction (e.g. via
    ///   `cosmrs`).
    ///
    /// ## Returns
    ///
    /// The raw `BroadcastTxResponse` on success.  The caller should inspect
    /// `response.tx_response.code` — `0` means accepted.
    ///
    /// ## Errors
    ///
    /// Returns [`ExchangeError::Network`] if there is no gRPC channel attached
    /// (call [`Self::with_grpc_channel`] first) or if the RPC call fails.
    #[cfg(feature = "grpc")]
    pub async fn place_order_grpc(
        &self,
        tx_raw_bytes: Vec<u8>,
    ) -> ExchangeResult<BroadcastTxResponse> {
        let channel = self.grpc_channel.as_ref().ok_or_else(|| {
            ExchangeError::Network(
                "No gRPC channel attached. Call DydxConnector::with_grpc_channel() \
                 with a channel connected to a dYdX validator node."
                    .to_string(),
            )
        })?;

        let request = BroadcastTxRequest {
            tx_bytes: tx_raw_bytes,
            mode: BroadcastMode::Sync as i32,
        };

        self.broadcast_tx(channel.clone(), request).await
    }

    /// Broadcast a pre-signed `TxRaw` that wraps a `MsgCancelOrder` to the
    /// dYdX validator node via `cosmos.tx.v1beta1.Service/BroadcastTx`.
    ///
    /// ## Parameters
    ///
    /// - `tx_raw_bytes` — protobuf-serialised `TxRaw` containing a signed
    ///   `MsgCancelOrder`.
    ///
    /// ## Returns
    ///
    /// The raw `BroadcastTxResponse` on success.
    ///
    /// ## Errors
    ///
    /// Returns [`ExchangeError::Network`] if there is no gRPC channel or the
    /// RPC call fails.
    #[cfg(feature = "grpc")]
    pub async fn cancel_order_grpc(
        &self,
        tx_raw_bytes: Vec<u8>,
    ) -> ExchangeResult<BroadcastTxResponse> {
        // Cancel orders are also broadcast via the same BroadcastTx endpoint;
        // the difference is only in the message type embedded in TxRaw.
        let channel = self.grpc_channel.as_ref().ok_or_else(|| {
            ExchangeError::Network(
                "No gRPC channel attached. Call DydxConnector::with_grpc_channel() \
                 with a channel connected to a dYdX validator node."
                    .to_string(),
            )
        })?;

        let request = BroadcastTxRequest {
            tx_bytes: tx_raw_bytes,
            mode: BroadcastMode::Sync as i32,
        };

        self.broadcast_tx(channel.clone(), request).await
    }

    /// Internal helper: send a `BroadcastTxRequest` to the Cosmos
    /// `cosmos.tx.v1beta1.Service/BroadcastTx` endpoint using the raw tonic
    /// `Grpc` client (no generated service stub required).
    #[cfg(feature = "grpc")]
    async fn broadcast_tx(
        &self,
        channel: Channel,
        request: BroadcastTxRequest,
    ) -> ExchangeResult<BroadcastTxResponse> {
        use tonic::client::Grpc;
        use tonic::IntoRequest;

        let mut grpc: Grpc<Channel> = Grpc::new(channel);

        // Wait until the channel is ready before sending.
        grpc.ready().await.map_err(|e| {
            ExchangeError::Network(format!("gRPC channel not ready: {}", e))
        })?;

        // Full gRPC method path for BroadcastTx.
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(
            "/cosmos.tx.v1beta1.Service/BroadcastTx",
        );

        let codec: ProstCodec<BroadcastTxRequest, BroadcastTxResponse> =
            ProstCodec::default();

        let response = grpc
            .unary(request.into_request(), path, codec)
            .await
            .map_err(|e| {
                ExchangeError::Network(format!("BroadcastTx gRPC error: {}", e))
            })?;

        Ok(response.into_inner())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for DydxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Dydx
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            (limiter.current_count(), limiter.max_requests())
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

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::FuturesCross, AccountType::FuturesIsolated]
    }
}

#[async_trait]
impl MarketData for DydxConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;
        DydxParser::parse_price(&response, &market)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;
        DydxParser::parse_ticker(&response, &market)
    }

    async fn get_orderbook(&self, symbol: Symbol, _depth: Option<u16>, _account_type: AccountType) -> ExchangeResult<OrderBook> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let mut params = HashMap::new();
        params.insert("market".to_string(), market.clone());

        let response = self.get(DydxEndpoint::Orderbook, params).await?;
        DydxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let resolution = map_kline_interval(interval);

        let mut params = HashMap::new();
        params.insert("market".to_string(), market.clone());
        params.insert("resolution".to_string(), resolution.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }
        if let Some(et) = end_time {
            if let Some(dt) = chrono::DateTime::from_timestamp_millis(et) {
                params.insert("toISO".to_string(), dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
            }
        }

        let response = self.get(DydxEndpoint::Candles, params).await?;
        DydxParser::parse_klines(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(DydxEndpoint::ServerTime, HashMap::new()).await?;
        if response.get("epoch").is_some() {
            Ok(())
        } else {
            Err(ExchangeError::Api {
                code: 0,
                message: "Ping failed".to_string(),
            })
        }
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;

        let markets = response.get("markets")
            .and_then(|m| m.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing markets".to_string()))?;

        let infos = markets.iter().map(|(ticker, data)| {
            // dYdX uses "BTC-USD" format
            let parts: Vec<&str> = ticker.splitn(2, '-').collect();
            let base = parts.first().copied().unwrap_or(ticker).to_string();
            let quote = parts.get(1).copied().unwrap_or("USD").to_string();

            let status = data.get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("ACTIVE")
                .to_string();

            // Parse step size / tick size for precision hints
            let step_size = data.get("stepSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let tick_size = data.get("tickSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_notional = data.get("minOrderSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            SymbolInfo {
                symbol: ticker.clone(),
                base_asset: base,
                quote_asset: quote,
                status,
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: min_notional,
                max_quantity: None,
                tick_size,
                step_size,
                min_notional: None,
                account_type,
            }
        }).collect();

        Ok(infos)
    }

    fn market_data_capabilities(&self) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,         // GET /v4/time
            has_price: true,        // GET /v4/perpetualMarkets (oracle price field)
            has_ticker: true,       // GET /v4/perpetualMarkets (24h stats)
            has_orderbook: true,    // GET /v4/orderbooks/perpetualMarket/{market}
            has_klines: true,       // GET /v4/candles/perpetualMarkets/{market}
            has_exchange_info: true, // GET /v4/perpetualMarkets (full symbol list)
            has_recent_trades: false, // Trades endpoint exists but not exposed via trait
            supported_intervals: &["1m", "5m", "15m", "30m", "1h", "4h", "1d"],
            max_kline_limit: Some(1000),
        }
    }
}

#[async_trait]
impl Account for DydxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        // dYdX balances are per-subaccount; address is stored in credentials.
        // If credentials contain an address, use subaccount 0 by default.
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_balance requires a dYdX address. Provide it via Credentials::new(address, \"\").".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccount_number".to_string(), "0".to_string());

        let response = self.get(DydxEndpoint::SpecificSubaccount, params).await?;
        let mut balances = DydxParser::parse_balances(&response)?;

        // Filter by asset if requested
        if let Some(asset_filter) = &query.asset {
            balances.retain(|b| b.asset.eq_ignore_ascii_case(asset_filter));
        }

        Ok(balances)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_account_info requires a dYdX address.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccount_number".to_string(), "0".to_string());

        let response = self.get(DydxEndpoint::SpecificSubaccount, params).await?;
        let balances = DydxParser::parse_balances(&response)?;

        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0,   // dYdX fees vary; fills endpoint needed
            taker_commission: 0.0005, // dYdX default taker: 0.05%
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // dYdX does not expose a dedicated fee-schedule endpoint.
        // We approximate fees from fills: compute effective rate as fee/size.
        // Without an address we return the published default schedule.
        let (maker_rate, taker_rate) = if let Some(address) = self.auth.address() {
            let mut params = HashMap::new();
            params.insert("address".to_string(), address.to_string());
            params.insert("subaccountNumber".to_string(), "0".to_string());
            params.insert("limit".to_string(), "10".to_string());
            if let Some(sym) = symbol {
                params.insert("ticker".to_string(), sym.to_string());
            }

            match self.get(DydxEndpoint::Fills, params).await {
                Ok(response) => {
                    let fills = response.get("fills")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    if fills.is_empty() {
                        // No fills — use published defaults
                        (0.0, 0.0005)
                    } else {
                        // Compute effective fee rate from recent fills
                        let mut total_fee = 0.0f64;
                        let mut total_value = 0.0f64;
                        let mut maker_count = 0usize;
                        let mut taker_count = 0usize;

                        for fill in &fills {
                            let size: f64 = fill.get("size")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let price: f64 = fill.get("price")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let fee: f64 = fill.get("fee")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let liquidity = fill.get("liquidity")
                                .and_then(|v| v.as_str())
                                .unwrap_or("TAKER");

                            total_fee += fee.abs();
                            total_value += size * price;
                            if liquidity == "MAKER" { maker_count += 1; } else { taker_count += 1; }
                        }

                        let effective_rate = if total_value > 0.0 { total_fee / total_value } else { 0.0005 };

                        // Estimate maker/taker split from liquidity counts
                        let total = (maker_count + taker_count) as f64;
                        if total == 0.0 {
                            (0.0, effective_rate)
                        } else {
                            let maker_share = maker_count as f64 / total;
                            let taker_share = taker_count as f64 / total;
                            // Maker rate is typically negative (rebate) or zero on dYdX
                            let implied_taker = if taker_share > 0.0 { effective_rate / taker_share } else { 0.0005 };
                            let implied_maker = if maker_share > 0.0 { -(effective_rate * 0.1) } else { 0.0 };
                            (implied_maker, implied_taker)
                        }
                    }
                }
                Err(_) => (0.0, 0.0005), // Fallback to published defaults
            }
        } else {
            // No credentials — published default fee schedule
            // dYdX v4: maker rebate ~ -0.011%, taker fee ~ 0.050%
            (-0.00011, 0.0005)
        };

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }

    fn account_capabilities(&self) -> AccountCapabilities {
        AccountCapabilities {
            has_balances: true,       // GET /v4/addresses/{addr}/subaccountNumber/{n}
            has_account_info: true,   // same endpoint, wrapped into AccountInfo
            has_fees: true,           // approximated from GET /v4/fills (or published defaults)
            has_transfers: false,     // Transfers endpoint exists but not in trait impl
            has_sub_accounts: true,   // SpecificSubaccount + ParentSubaccount endpoints used
            has_deposit_withdraw: false, // no deposit/withdraw API on Indexer
            has_margin: false,        // margin mode changes require gRPC Node API
            has_earn_staking: false,  // no earn/staking on dYdX v4
            has_funding_history: true, // FundingHistory trait is implemented
            has_ledger: false,        // no ledger/transaction log endpoint
            has_convert: false,       // no coin conversion endpoint
        }
    }
}

#[async_trait]
impl Positions for DydxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_positions requires a dYdX address in credentials.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());
        params.insert("status".to_string(), "OPEN".to_string());

        if let Some(sym) = &query.symbol {
            // dYdX symbol format: BTC-USD
            let market = format!("{}-USD", sym.base.to_uppercase());
            params.insert("market".to_string(), market);
        }

        let response = self.get(DydxEndpoint::PerpetualPositions, params).await?;
        DydxParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Normalize symbol: "BTC" → "BTC-USD", "BTC-USD" → "BTC-USD"
        let market = if symbol.contains('-') {
            symbol.to_uppercase()
        } else {
            format!("{}-USD", symbol.to_uppercase())
        };

        let mut params = HashMap::new();
        params.insert("market".to_string(), market.clone());
        params.insert("limit".to_string(), "1".to_string());

        let response = self.get(DydxEndpoint::HistoricalFunding, params).await?;
        let mut funding = DydxParser::parse_funding_rate(&response)?;

        // Override symbol with the normalized market ticker
        funding.symbol = market;
        Ok(funding)
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "dYdX v4 position modification (leverage, margin mode) requires Cosmos gRPC (Node API). \
             The Indexer REST API is read-only.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING (Read-only via Indexer; write operations require Node gRPC)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for DydxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        // dYdX v4 order placement requires Cosmos SDK gRPC (MsgPlaceOrder).
        // The Indexer REST API is read-only; write operations go through validator
        // nodes via gRPC/Protobuf and require a signed Cosmos transaction.
        //
        // Path 1 (onchain-cosmos + grpc + cosmos_provider + grpc_channel + credentials):
        //   → use tx_builder to build + sign TxRaw, then broadcast via gRPC.
        //
        // Path 2 (grpc only, no cosmos_provider):
        //   → return an informative error pointing to place_order_grpc().
        //
        // Path 3 (neither feature):
        //   → return UnsupportedOperation.

        #[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
        {
            use crate::core::chain::cosmos::CosmosChain as _;
            use crate::core::chain::ChainProvider as _;
            use super::tx_builder::{
                build_place_order_tx, build_place_conditional_order_tx,
                build_place_long_term_order_tx,
                signing_key_from_bytes, ShortTermOrderParams, ConditionalOrderParams,
                LongTermOrderParams,
            };
            use super::proto::dydxprotocol::OrderConditionType;

            if let (Some(cosmos), Some(_channel)) =
                (self.cosmos_provider.as_ref(), self.grpc_channel.as_ref())
            {
                // api_key = hex-encoded secp256k1 private key (32 bytes)
                // api_secret = bech32 dYdX chain address (dydx1...)
                let (key_hex, owner_address) = self.auth.trading_credentials()
                    .ok_or_else(|| ExchangeError::Auth(
                        "dYdX place_order requires trading credentials: \
                         api_key = hex secp256k1 private key, \
                         api_secret = bech32 address (dydx1...)"
                            .to_string()
                    ))?;

                let key_bytes = hex::decode(key_hex.trim_start_matches("0x"))
                    .map_err(|e| ExchangeError::Auth(format!(
                        "dYdX place_order: api_key is not valid hex: {}",
                        e
                    )))?;
                let signing_key = signing_key_from_bytes(&key_bytes)?;

                let (account_number, _) = cosmos.query_account(&owner_address).await?;
                let sequence = cosmos.next_sequence(&owner_address).await?;

                let chain_id = match cosmos.chain_family() {
                    crate::core::chain::ChainFamily::Cosmos { ref chain_id } => {
                        chain_id.clone()
                    }
                    _ => "dydx-mainnet-1".to_string(),
                };

                // Fetch market config for this symbol (cached after first call).
                // Used for both CLOB pair ID lookup and dynamic step size encoding.
                let ticker = format!("{}-USD", req.symbol.base.to_uppercase());
                let market_cfg = self.fetch_market_configs().await
                    .ok()
                    .and_then(|map| map.get(&ticker));

                // Use dynamic CLOB pair ID from market config; fall back to static map.
                let clob_pair_id = market_cfg
                    .map(|cfg| cfg.clob_pair_id)
                    .unwrap_or_else(|| symbol_to_clob_pair_id(&req.symbol.base));

                // Determine whether this is a conditional (stop/TP) order.
                // Conditional orders use ORDER_FLAG_CONDITIONAL (32), must have a
                // trigger_subticks and condition_type, and always use good_til_block_time.
                //
                // Mapping:
                //   StopMarket / StopLimit                 → StopLoss  (1)
                //   ConditionalPlan { Above, ... }         → TakeProfit (2)
                //     — price rises above trigger: typical TP for a long position
                //   ConditionalPlan { Below, ... }         → StopLoss  (1)
                //     — price falls below trigger: typical SL for a long position
                let conditional_info: Option<(super::proto::dydxprotocol::OrderConditionType, f64)> =
                    match &req.order_type {
                        crate::core::OrderType::StopMarket { stop_price } => {
                            Some((OrderConditionType::StopLoss, *stop_price))
                        }
                        crate::core::OrderType::StopLimit { stop_price, .. } => {
                            Some((OrderConditionType::StopLoss, *stop_price))
                        }
                        // ConditionalPlan with TriggerDirection::Above routes to TakeProfit.
                        // ConditionalPlan with TriggerDirection::Below routes to StopLoss.
                        crate::core::OrderType::ConditionalPlan {
                            trigger_price,
                            trigger_direction,
                            ..
                        } => {
                            let cond_type = match trigger_direction {
                                crate::core::TriggerDirection::Above => {
                                    OrderConditionType::TakeProfit
                                }
                                crate::core::TriggerDirection::Below => {
                                    OrderConditionType::StopLoss
                                }
                            };
                            Some((cond_type, *trigger_price))
                        }
                        _ => None,
                    };

                // Extract quantums/subticks using market-specific step sizes.
                let (quantums, subticks) =
                    order_request_to_quantums_subticks(&req, market_cfg)?;

                let client_id = req.client_order_id
                    .as_deref()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or_else(|| {
                        // Fallback: subsecond nanos as a cheap unique client ID
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .subsec_nanos()
                    });

                let tif_i32 = map_tif_to_dydx_i32(&req.time_in_force);
                let reduce_only = if req.reduce_only { 1 } else { 0 };
                let is_buy = req.side == crate::core::OrderSide::Buy;

                let tx_bytes = if let Some((condition_type, trigger_price)) = conditional_info {
                    // Conditional orders must use good_til_block_time (LONG_TERM expiry).
                    // Default to 28 days from now (maximum dYdX conditional lifetime).
                    let good_til_block_time = {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        (now + 28 * 24 * 3600) as u32
                    };

                    // Compute trigger_subticks from trigger_price using market's subticks_per_tick.
                    let subticks_per_tick = market_cfg
                        .map(|cfg| cfg.subticks_per_tick)
                        .unwrap_or(100_000.0);
                    let trigger_subticks = (trigger_price * subticks_per_tick).round() as u64;

                    let cond_params = ConditionalOrderParams {
                        owner_address: owner_address.clone(),
                        subaccount_number: 0,
                        client_id,
                        clob_pair_id,
                        is_buy,
                        quantums,
                        subticks,
                        good_til_block_time,
                        time_in_force: tif_i32,
                        reduce_only,
                        condition_type,
                        trigger_subticks,
                    };

                    build_place_conditional_order_tx(
                        &cond_params,
                        &signing_key,
                        account_number,
                        sequence,
                        &chain_id,
                        None,
                    )?
                } else {
                    // Detect whether this is a LONG_TERM order.
                    //
                    // dYdX supports two non-conditional order lifetimes:
                    //   SHORT_TERM — expires at a block height (ORDER_FLAG_SHORT_TERM = 0)
                    //   LONG_TERM  — expires at a UTC timestamp (ORDER_FLAG_LONG_TERM = 64)
                    //
                    // A caller signals LONG_TERM intent by using TimeInForce::Gtd, which
                    // carries an `expire_time` in milliseconds since the Unix epoch.
                    // We also treat OrderType::Gtd { expire_time } the same way.
                    //
                    // Any other TIF falls through to the SHORT_TERM path.
                    let long_term_expiry_secs: Option<u32> = match (&req.time_in_force, &req.order_type) {
                        (crate::core::TimeInForce::Gtd, crate::core::OrderType::Gtd { expire_time, .. }) => {
                            // expire_time is Unix ms; convert to Unix seconds for dYdX wire format.
                            Some((*expire_time / 1000) as u32)
                        }
                        (crate::core::TimeInForce::Gtd, _) => {
                            // TIF is Gtd but order type doesn't carry a timestamp.
                            // Default to 90 days from now — maximum LONG_TERM lifetime on dYdX.
                            let now_secs = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            Some((now_secs + 90 * 24 * 3600) as u32)
                        }
                        _ => None,
                    };

                    if let Some(good_til_block_time) = long_term_expiry_secs {
                        // LONG_TERM order: expires at a wall-clock timestamp.
                        // Uses ORDER_FLAG_LONG_TERM (64) and good_til_block_time instead of
                        // good_til_block.  No fee is required (same zero-fee policy as SHORT_TERM).
                        let params = LongTermOrderParams {
                            owner_address: owner_address.clone(),
                            subaccount_number: 0,
                            client_id,
                            clob_pair_id,
                            is_buy,
                            quantums,
                            subticks,
                            good_til_block_time,
                            time_in_force: tif_i32,
                            reduce_only,
                        };

                        build_place_long_term_order_tx(
                            &params,
                            &signing_key,
                            account_number,
                            sequence,
                            &chain_id,
                            None,
                        )?
                    } else {
                        // Standard SHORT_TERM order (Limit / Market / PostOnly / IOC / FOK).
                        // Expires at a block height; 0 means "use exchange default" (current + 20).
                        let good_til_block = match &req.time_in_force {
                            crate::core::TimeInForce::GoodTilBlock { block_height } => {
                                *block_height as u32
                            }
                            _ => 0u32,
                        };

                        let params = ShortTermOrderParams {
                            owner_address: owner_address.clone(),
                            subaccount_number: 0,
                            client_id,
                            clob_pair_id,
                            is_buy,
                            quantums,
                            subticks,
                            good_til_block,
                            time_in_force: tif_i32,
                            reduce_only,
                        };

                        build_place_order_tx(
                            &params,
                            &signing_key,
                            account_number,
                            sequence,
                            &chain_id,
                            None, // zero fee — standard for dYdX short-term orders
                        )?
                    }
                };

                let resp = self.place_order_grpc(tx_bytes).await?;

                let tx_response = resp.tx_response.as_ref();
                let code = tx_response.map(|r| r.code).unwrap_or(1);
                let raw_log = tx_response
                    .map(|r| r.raw_log.clone())
                    .unwrap_or_default();

                if code != 0 {
                    return Err(ExchangeError::Api {
                        code: code as i32,
                        message: format!("dYdX place_order rejected (code {}): {}", code, raw_log),
                    });
                }

                let tx_hash = tx_response
                    .map(|r| r.txhash.clone())
                    .unwrap_or_default();

                let display_price = match &req.order_type {
                    crate::core::OrderType::Limit { price } => Some(*price),
                    crate::core::OrderType::StopLimit { limit_price, .. } => Some(*limit_price),
                    crate::core::OrderType::Gtd { price, .. } => Some(*price),
                    _ => None,
                };

                let display_stop_price = match &req.order_type {
                    crate::core::OrderType::StopMarket { stop_price } => Some(*stop_price),
                    crate::core::OrderType::StopLimit { stop_price, .. } => Some(*stop_price),
                    crate::core::OrderType::ConditionalPlan { trigger_price, .. } => {
                        Some(*trigger_price)
                    }
                    _ => None,
                };

                let order = Order {
                    id: tx_hash,
                    client_order_id: req.client_order_id,
                    symbol: format!("{}-{}", req.symbol.base, req.symbol.quote),
                    side: req.side,
                    order_type: req.order_type,
                    status: crate::core::OrderStatus::Open,
                    price: display_price,
                    stop_price: display_stop_price,
                    quantity: req.quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: chrono::Utc::now().timestamp_millis(),
                    updated_at: None,
                    time_in_force: req.time_in_force,
                };

                return Ok(PlaceOrderResponse::Simple(order));
            }
        }

        let _ = req;

        #[cfg(feature = "grpc")]
        if self.grpc_channel.is_some() {
            return Err(ExchangeError::UnsupportedOperation(
                "dYdX v4 order placement via gRPC: use `place_order_grpc(tx_raw_bytes)` \
                 directly. Build and sign the Cosmos SDK TxRaw externally (e.g. with \
                 `cosmrs`), then pass the serialised bytes to that method. \
                 Or enable `onchain-cosmos` feature and attach a CosmosProvider \
                 via `DydxConnector::with_cosmos_provider` for automatic tx building."
                    .to_string(),
            ));
        }

        Err(ExchangeError::UnsupportedOperation(
            "dYdX v4 order placement requires Cosmos gRPC (Node API). \
             Enable the `grpc` + `onchain-cosmos` features, connect a validator \
             channel via `DydxConnector::with_grpc_channel`, attach a CosmosProvider \
             via `DydxConnector::with_cosmos_provider`, and ensure credentials contain \
             a signing key (api_key) and address (api_secret)."
                .to_string(),
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        // dYdX v4 order cancellation also requires Node gRPC (MsgCancelOrder).
        //
        // Path 1 (onchain-cosmos + grpc + cosmos_provider + grpc_channel + credentials):
        //   → use tx_builder to build + sign cancel TxRaw, then broadcast.
        //
        // Path 2 / Path 3: same fallback as place_order.

        #[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
        {
            use crate::core::chain::cosmos::CosmosChain as _;
            use crate::core::chain::ChainProvider as _;
            use super::tx_builder::{
                build_cancel_order_tx, signing_key_from_bytes, CancelOrderParams,
            };
            use super::proto::dydxprotocol::ORDER_FLAG_SHORT_TERM;
            use crate::core::CancelScope;

            if let (Some(cosmos), Some(_channel)) =
                (self.cosmos_provider.as_ref(), self.grpc_channel.as_ref())
            {
                let (key_hex, owner_address) = self.auth.trading_credentials()
                    .ok_or_else(|| ExchangeError::Auth(
                        "dYdX cancel_order requires trading credentials.".to_string()
                    ))?;

                let key_bytes = hex::decode(key_hex.trim_start_matches("0x"))
                    .map_err(|e| ExchangeError::Auth(format!(
                        "dYdX cancel_order: api_key is not valid hex: {}",
                        e
                    )))?;
                let signing_key = signing_key_from_bytes(&key_bytes)?;

                let (account_number, _) = cosmos.query_account(&owner_address).await?;
                let sequence = cosmos.next_sequence(&owner_address).await?;

                let chain_id = match cosmos.chain_family() {
                    crate::core::chain::ChainFamily::Cosmos { ref chain_id } => {
                        chain_id.clone()
                    }
                    _ => "dydx-mainnet-1".to_string(),
                };

                // Extract order_id and symbol from the cancel scope
                let (order_id_str, symbol_base) = match &req.scope {
                    CancelScope::Single { order_id } => {
                        let base = req.symbol.as_ref()
                            .map(|s| s.base.clone())
                            .unwrap_or_default();
                        (order_id.clone(), base)
                    }
                    _ => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "dYdX cancel_order only supports CancelScope::Single.".to_string()
                        ));
                    }
                };

                let clob_pair_id = symbol_to_clob_pair_id(&symbol_base);

                // dYdX order IDs encode client_id — parse best-effort
                let client_id = order_id_str.parse::<u32>().unwrap_or(0);

                let params = CancelOrderParams {
                    owner_address: owner_address.clone(),
                    subaccount_number: 0,
                    client_id,
                    clob_pair_id,
                    order_flags: ORDER_FLAG_SHORT_TERM,
                    good_til_block: None,    // caller must set via separate API if needed
                    good_til_block_time: None,
                };

                let tx_bytes = build_cancel_order_tx(
                    &params,
                    &signing_key,
                    account_number,
                    sequence,
                    &chain_id,
                    None,
                )?;

                let resp = self.cancel_order_grpc(tx_bytes).await?;

                let tx_response = resp.tx_response.as_ref();
                let code = tx_response.map(|r| r.code).unwrap_or(1);
                let raw_log = tx_response
                    .map(|r| r.raw_log.clone())
                    .unwrap_or_default();

                if code != 0 {
                    return Err(ExchangeError::Api {
                        code: code as i32,
                        message: format!("dYdX cancel_order rejected (code {}): {}", code, raw_log),
                    });
                }

                let tx_hash = tx_response
                    .map(|r| r.txhash.clone())
                    .unwrap_or_default();

                let symbol_str = req.symbol
                    .as_ref()
                    .map(|s| format!("{}-{}", s.base, s.quote))
                    .unwrap_or_else(|| symbol_base.clone());

                return Ok(Order {
                    id: order_id_str,
                    client_order_id: Some(tx_hash),
                    symbol: symbol_str,
                    side: crate::core::OrderSide::Buy, // unknown at cancel time
                    order_type: crate::core::OrderType::Limit { price: 0.0 },
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(chrono::Utc::now().timestamp_millis()),
                    time_in_force: crate::core::TimeInForce::Gtc,
                });
            }
        }

        let _ = req;

        #[cfg(feature = "grpc")]
        if self.grpc_channel.is_some() {
            return Err(ExchangeError::UnsupportedOperation(
                "dYdX v4 order cancellation via gRPC: use `cancel_order_grpc(tx_raw_bytes)` \
                 directly. Build and sign the Cosmos SDK TxRaw externally (e.g. with \
                 `cosmrs`), then pass the serialised bytes to that method. \
                 Or enable `onchain-cosmos` feature and attach a CosmosProvider \
                 via `DydxConnector::with_cosmos_provider` for automatic tx building."
                    .to_string(),
            ));
        }

        Err(ExchangeError::UnsupportedOperation(
            "dYdX v4 order cancellation requires Cosmos gRPC (Node API). \
             Enable the `grpc` + `onchain-cosmos` features, connect a validator \
             channel via `DydxConnector::with_grpc_channel`, and attach a CosmosProvider \
             via `DydxConnector::with_cosmos_provider`."
                .to_string(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(DydxEndpoint::SpecificOrder, params).await?;
        DydxParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_open_orders requires a dYdX address in credentials.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());
        params.insert("status".to_string(), "OPEN".to_string());

        if let Some(sym) = symbol {
            // Normalize to dYdX format
            let market = if sym.contains('-') {
                sym.to_uppercase()
            } else {
                format!("{}-USD", sym.to_uppercase())
            };
            params.insert("ticker".to_string(), market);
        }

        let response = self.get(DydxEndpoint::Orders, params).await?;
        // Orders endpoint returns an array directly
        DydxParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_order_history requires a dYdX address in credentials.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());

        if let Some(sym) = &filter.symbol {
            let market = format!("{}-USD", sym.base.to_uppercase());
            params.insert("ticker".to_string(), market);
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }
        // Filter to non-open orders (filled, canceled)
        params.insert("returnLatestOrders".to_string(), "true".to_string());

        let response = self.get(DydxEndpoint::Orders, params).await?;
        let mut orders = DydxParser::parse_orders(&response)?;

        // Apply time filters if provided
        if let Some(start) = filter.start_time {
            orders.retain(|o| o.created_at >= start);
        }
        if let Some(end) = filter.end_time {
            orders.retain(|o| o.created_at <= end);
        }

        Ok(orders)
    }

    /// Fetch trade fills from `GET /v4/fills`.
    ///
    /// Requires a dYdX chain address stored in `credentials.api_key`.
    /// No cryptographic signature is needed — the Indexer API is public
    /// and keyed only by address.
    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_user_trades requires a dYdX address in credentials.api_key.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());

        if let Some(sym) = &filter.symbol {
            // dYdX market format: "BTC-USD". Accept "BTC" or "BTC-USD".
            let market = if sym.contains('-') {
                sym.clone()
            } else {
                format!("{}-USD", sym.to_uppercase())
            };
            params.insert("market".to_string(), market);
            params.insert("marketType".to_string(), "PERPETUAL".to_string());
        }

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        let response = self.get(DydxEndpoint::Fills, params).await?;
        let mut trades = DydxParser::parse_fills(&response)?;

        // Apply order_id filter (not supported as a query param)
        if let Some(oid) = &filter.order_id {
            trades.retain(|t| &t.order_id == oid);
        }

        // Apply time filters (filter values are u64 ms; timestamp is i64 ms)
        if let Some(start) = filter.start_time {
            trades.retain(|t| t.timestamp >= start as i64);
        }
        if let Some(end) = filter.end_time {
            trades.retain(|t| t.timestamp <= end as i64);
        }

        Ok(trades)
    }

    fn trading_capabilities(&self) -> TradingCapabilities {
        TradingCapabilities {
            // Order placement requires grpc + onchain-cosmos features + credentials.
            // Without those features the connector is read-only; declare false here
            // because the default (no-feature) build returns UnsupportedOperation.
            has_market_order: false,
            has_limit_order: false,   // available only with grpc+onchain-cosmos features
            has_stop_market: false,   // conditional orders require same feature gates
            has_stop_limit: false,
            has_trailing_stop: false, // not supported by dYdX v4 protocol
            has_bracket: false,       // not a native dYdX order type
            has_oco: false,           // not a native dYdX order type
            has_amend: false,         // no order amendment endpoint
            has_batch: false,         // no batch order API
            max_batch_size: None,
            has_cancel_all: false,    // cancel_all_orders is a helper, not in the trait
            has_user_trades: true,    // GET /v4/fills is implemented
            has_order_history: true,  // GET /v4/orders (returnLatestOrders) is implemented
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS
// ═══════════════════════════════════════════════════════════════════════════════

impl DydxConnector {
    /// Получить balances для конкретного subaccount
    pub async fn get_subaccount_balances(
        &self,
        address: &str,
        subaccount_number: u32,
    ) -> ExchangeResult<Vec<Balance>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccount_number".to_string(), subaccount_number.to_string());

        let response = self.get(DydxEndpoint::SpecificSubaccount, params).await?;
        DydxParser::parse_balances(&response)
    }

    /// Получить positions для конкретного subaccount
    pub async fn get_subaccount_positions(
        &self,
        address: &str,
        subaccount_number: u32,
    ) -> ExchangeResult<Vec<Position>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), subaccount_number.to_string());

        let response = self.get(DydxEndpoint::PerpetualPositions, params).await?;
        DydxParser::parse_positions(&response)
    }

    /// Fetch all perpetual market configs from `GET /v4/perpetualMarkets`.
    ///
    /// Results are cached after the first successful fetch so subsequent calls
    /// return immediately without a network round-trip.
    ///
    /// Each entry in the returned map is keyed by the dYdX ticker (e.g. `"BTC-USD"`)
    /// and contains the market-specific step sizes needed for on-chain order encoding.
    pub async fn fetch_market_configs(&self) -> ExchangeResult<&HashMap<String, MarketConfig>> {
        self.market_config_cache
            .get_or_try_init(|| async {
                let response = self
                    .get(DydxEndpoint::PerpetualMarkets, HashMap::new())
                    .await?;

                let markets = response
                    .get("markets")
                    .and_then(|m| m.as_object())
                    .ok_or_else(|| {
                        ExchangeError::Parse("Missing 'markets' in perpetualMarkets response".to_string())
                    })?;

                let mut configs = HashMap::with_capacity(markets.len());

                for (ticker, market) in markets {
                    // Parse all numeric fields; skip markets with missing required data.
                    let clob_pair_id = market
                        .get("clobPairId")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<u32>().ok())
                            .or_else(|| v.as_u64().map(|n| n as u32)))
                        .unwrap_or(0);

                    let step_base_quantums = market
                        .get("stepBaseQuantum")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                            .or_else(|| v.as_f64()))
                        .unwrap_or(1_000_000.0); // safe fallback

                    let subticks_per_tick = market
                        .get("subticksPerTick")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                            .or_else(|| v.as_f64()))
                        .unwrap_or(100_000.0); // safe fallback

                    let quantum_conversion_exponent = market
                        .get("quantumConversionExponent")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(-9) as i32;

                    let atomic_resolution = market
                        .get("atomicResolution")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(-10) as i32;

                    configs.insert(
                        ticker.clone(),
                        MarketConfig {
                            ticker: ticker.clone(),
                            clob_pair_id,
                            step_base_quantums,
                            subticks_per_tick,
                            quantum_conversion_exponent,
                            atomic_resolution,
                        },
                    );
                }

                Ok(configs)
            })
            .await
    }

    /// Get the [`MarketConfig`] for a specific ticker symbol (e.g. `"BTC-USD"`).
    ///
    /// Fetches and caches the full market config on the first call, then looks
    /// up the requested ticker. Returns an error if the ticker is not found.
    pub async fn get_market_config(&self, ticker: &str) -> ExchangeResult<&MarketConfig> {
        let configs = self.fetch_market_configs().await?;
        configs.get(ticker).ok_or_else(|| {
            ExchangeError::Parse(format!(
                "dYdX market '{}' not found in perpetualMarkets. \
                 Check that the ticker is in 'BASE-USD' format and exists on dYdX.",
                ticker
            ))
        })
    }

    /// Получить market info (для clobPairId mapping)
    pub async fn get_market_info(&self, ticker: &str) -> ExchangeResult<Value> {
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;

        let markets = response.get("markets")
            .and_then(|m| m.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing markets".to_string()))?;

        markets.get(ticker)
            .cloned()
            .ok_or_else(|| ExchangeError::Parse(format!("Market {} not found", ticker)))
    }

    /// Получить orders для конкретного subaccount (read-only via Indexer)
    pub async fn get_orders_for_subaccount(
        &self,
        address: &str,
        subaccount_number: u32,
        ticker: Option<&str>,
        status: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), subaccount_number.to_string());
        if let Some(t) = ticker {
            params.insert("ticker".to_string(), t.to_string());
        }
        if let Some(s) = status {
            params.insert("status".to_string(), s.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let response = self.get(DydxEndpoint::Orders, params).await?;
        DydxParser::parse_orders(&response)
    }

    /// Получить все markets
    pub async fn get_all_markets(&self) -> ExchangeResult<HashMap<String, Value>> {
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;

        let markets = response.get("markets")
            .and_then(|m| m.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing markets".to_string()))?;

        Ok(markets.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    /// Get transfers between two subaccounts
    ///
    /// Returns transfers from `source_subaccount_number` to `recipient_subaccount_number`
    /// for the given `address`.
    pub async fn get_transfers_between(
        &self,
        address: &str,
        source_subaccount_number: u32,
        recipient_subaccount_number: u32,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("sourceSubaccountNumber".to_string(), source_subaccount_number.to_string());
        params.insert("recipientSubaccountNumber".to_string(), recipient_subaccount_number.to_string());
        self.get(DydxEndpoint::TransfersBetween, params).await
    }

    /// Get asset positions for a parent subaccount number
    ///
    /// Returns asset positions across all child subaccounts under the given parent.
    pub async fn get_parent_asset_positions(
        &self,
        address: &str,
        parent_subaccount_number: u32,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("parentSubaccountNumber".to_string(), parent_subaccount_number.to_string());
        self.get(DydxEndpoint::ParentAssetPositions, params).await
    }

    /// Get transfers for a parent subaccount number
    pub async fn get_parent_transfers(
        &self,
        address: &str,
        parent_subaccount_number: u32,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("parentSubaccountNumber".to_string(), parent_subaccount_number.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(DydxEndpoint::ParentTransfers, params).await
    }

    /// Get MegaVault historical PnL
    ///
    /// Returns historical profit and loss data for the dYdX MegaVault.
    pub async fn get_megavault_pnl(
        &self,
        resolution: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(r) = resolution {
            params.insert("resolution".to_string(), r.to_string());
        }
        self.get(DydxEndpoint::MegaVaultPnl, params).await
    }

    /// Get MegaVault positions
    ///
    /// Returns current positions held in the dYdX MegaVault.
    pub async fn get_megavault_positions(&self) -> ExchangeResult<Value> {
        self.get(DydxEndpoint::MegaVaultPositions, HashMap::new()).await
    }

    /// Get historical PnL for all individual vaults
    ///
    /// Returns historical PnL data for all vaults (not just the MegaVault).
    pub async fn get_all_vaults_pnl(
        &self,
        resolution: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(r) = resolution {
            params.insert("resolution".to_string(), r.to_string());
        }
        self.get(DydxEndpoint::AllVaultsPnl, params).await
    }

    /// Get affiliate program metadata for an address
    pub async fn get_affiliate_metadata(&self, address: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        self.get(DydxEndpoint::AffiliateMetadata, params).await
    }

    /// Get affiliate address info for a referral code
    pub async fn get_affiliate_address(&self, referral_code: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("referralCode".to_string(), referral_code.to_string());
        self.get(DydxEndpoint::AffiliateAddress, params).await
    }

    /// Cancel all open orders, optionally filtered to a single symbol.
    ///
    /// This is a helper that is NOT part of the `Trading` trait because dYdX v4
    /// has no native "cancel-all" endpoint — every cancel is a separate signed
    /// on-chain transaction (`MsgCancelOrder`).
    ///
    /// ## Behaviour
    ///
    /// 1. Fetches all open orders via the Indexer REST API (`get_open_orders`).
    /// 2. Iterates serially and cancels each order using `cancel_order`.
    ///    Serial execution is intentional: the Cosmos sequence number must
    ///    increment monotonically across broadcast transactions, and the
    ///    `CosmosProvider` already manages that counter.
    /// 3. If any individual cancel fails, the error is collected and execution
    ///    continues for the remaining orders.  A combined error is returned only
    ///    if **all** cancels fail; partial failures surface via `tracing::warn`.
    ///
    /// ## Requires
    ///
    /// - `onchain-cosmos` + `grpc` features enabled.
    /// - A `CosmosProvider` attached via [`Self::with_cosmos_provider`].
    /// - A gRPC channel attached via [`Self::with_grpc_channel`].
    /// - Trading credentials (api_key = hex private key, api_secret = address).
    ///
    /// ## Returns
    ///
    /// The list of `Order` structs with `status = Canceled` for every order
    /// that was successfully cancelled.
    #[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
    pub async fn cancel_all_orders(
        &self,
        symbol: Option<&str>,
        account_type: crate::core::AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        use crate::core::CancelScope;

        // Fetch all open orders via the read-only Indexer REST API.
        let open_orders = self.get_open_orders(symbol, account_type).await?;

        if open_orders.is_empty() {
            return Ok(Vec::new());
        }

        let symbol_hint: Option<crate::core::Symbol> = symbol.map(|s| {
            let parts: Vec<&str> = s.splitn(2, '-').collect();
            crate::core::Symbol {
                base: parts.first().copied().unwrap_or(s).to_string(),
                quote: parts.get(1).copied().unwrap_or("USD").to_string(),
            }
        });

        let mut cancelled = Vec::with_capacity(open_orders.len());
        let mut error_count = 0usize;

        for order in open_orders {
            let cancel_req = crate::core::CancelRequest {
                scope: CancelScope::Single {
                    order_id: order.id.clone(),
                },
                symbol: symbol_hint.clone(),
                account_type,
            };

            match self.cancel_order(cancel_req).await {
                Ok(cancelled_order) => {
                    cancelled.push(cancelled_order);
                }
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(
                        "cancel_all_orders: failed to cancel order {} for {}: {}",
                        order.id,
                        order.symbol,
                        e
                    );
                }
            }
        }

        if cancelled.is_empty() && error_count > 0 {
            return Err(ExchangeError::Api {
                code: -1,
                message: format!(
                    "cancel_all_orders: all {} cancel attempts failed. \
                     Check trading credentials and gRPC connectivity.",
                    error_count
                ),
            });
        }

        Ok(cancelled)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for DydxConnector {
    /// Get historical funding payments from `GET /v4/fundingPayments`.
    ///
    /// dYdX Indexer API is public — no auth headers required. The account
    /// address is read from credentials (stored in `auth`).
    ///
    /// Params: `address`, `subaccountNumber` (0), optionally `market`, `limit`,
    /// `effectiveBeforeOrAtHeight`.
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        let address = self.auth.address().ok_or_else(|| {
            ExchangeError::Auth(
                "dYdX get_funding_payments requires a dYdX address in credentials.".to_string(),
            )
        })?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());

        if let Some(sym) = &filter.symbol {
            // dYdX market format is e.g. "BTC-USD"
            params.insert("market".to_string(), sym.clone());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(DydxEndpoint::FundingPayments, params).await?;
        DydxParser::parse_funding_payments(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TX BUILDER HELPERS (onchain-cosmos + grpc)
// ═══════════════════════════════════════════════════════════════════════════════

/// Map a base asset symbol to a dYdX CLOB pair ID.
///
/// This is a best-effort static mapping for common markets. For accurate
/// mapping, call `get_exchange_info()` and cache the `clobPairId` values
/// from the Indexer `/perpetualMarkets` endpoint.
#[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
fn symbol_to_clob_pair_id(base: &str) -> u32 {
    match base.to_uppercase().as_str() {
        "BTC" => 0,
        "ETH" => 1,
        "LINK" => 2,
        "MATIC" => 3,
        "CRV" => 4,
        "SOL" => 5,
        "ADA" => 6,
        "AVAX" => 7,
        "FIL" => 8,
        "LTC" => 9,
        "DOGE" => 10,
        "ATOM" => 11,
        "DOT" => 12,
        "UNI" => 13,
        "BCH" => 14,
        "TRX" => 15,
        "NEAR" => 16,
        "MKR" => 17,
        "XLM" => 18,
        "ETC" => 19,
        "COMP" => 20,
        "APE" => 21,
        "APT" => 22,
        "ARB" => 23,
        "BLUR" => 24,
        "LDO" => 25,
        "OP" => 26,
        "PEPE" => 27,
        "SEI" => 28,
        "SUI" => 29,
        "XRP" => 30,
        "DYDX" => 31,
        // Unknown market — callers should use exchange info to look up the real ID
        _ => {
            tracing::warn!(
                "symbol_to_clob_pair_id: unknown market '{}', defaulting to 0 (BTC-USD)",
                base
            );
            0
        }
    }
}

/// Extract `(quantums, subticks)` from an `OrderRequest` using dynamic market config.
///
/// When `market_cfg` is `Some`, the market-specific `step_base_quantums` and
/// `subticks_per_tick` values from the dYdX Indexer are used for precise encoding.
/// When `None`, conservative fallback constants are used (suitable only for
/// BTC-USD with approximate precision).
///
/// ## Encoding rules (from dYdX protocol)
///
/// ```text
/// quantums = round(quantity / step_base_quantums)
/// subticks  = round(price   * subticks_per_tick)
/// ```
///
/// Both values must be non-zero integers that fit in `u64`.
#[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
fn order_request_to_quantums_subticks(
    req: &OrderRequest,
    market_cfg: Option<&MarketConfig>,
) -> ExchangeResult<(u64, u64)> {
    // Extract the execution price from the order type.
    // For conditional orders the "subticks" field is the limit price that
    // executes after the trigger fires; for stop-market we simulate a sweep.
    let price = match &req.order_type {
        crate::core::OrderType::Limit { price } => *price,
        crate::core::OrderType::Market => {
            // Market orders on dYdX use a very large price (buy) or 1 (sell)
            // for crossing the book. Use a sentinel that the chain accepts.
            if req.side == crate::core::OrderSide::Buy {
                f64::MAX / 1e10 // large but fits in u64 subticks range
            } else {
                1.0
            }
        }
        // Stop-market: trigger = stop_price; execution uses a sweep sentinel
        // so the order fills immediately at best available price after trigger.
        crate::core::OrderType::StopMarket { .. } => {
            if req.side == crate::core::OrderSide::Buy {
                // Buy stop-market sweeps up: large price sentinel
                f64::MAX / 1e10
            } else {
                // Sell stop-market sweeps down: 1 tick sentinel
                1.0
            }
        }
        // Stop-limit: execution at limit_price; trigger at stop_price
        crate::core::OrderType::StopLimit { limit_price, .. } => *limit_price,

        // ConditionalPlan: the inner order_after_trigger determines the execution price.
        // Extract the limit price from the inner order when available; otherwise sweep.
        crate::core::OrderType::ConditionalPlan { order_after_trigger, .. } => {
            match order_after_trigger.as_ref() {
                crate::core::OrderType::Limit { price } => *price,
                crate::core::OrderType::StopLimit { limit_price, .. } => *limit_price,
                // Inner market order → sweep sentinel
                _ => {
                    if req.side == crate::core::OrderSide::Buy {
                        f64::MAX / 1e10
                    } else {
                        1.0
                    }
                }
            }
        }

        // Gtd (Good-Till-Date): long-term limit order with timestamp expiry.
        crate::core::OrderType::Gtd { price, .. } => *price,

        _ => {
            return Err(ExchangeError::UnsupportedOperation(format!(
                "dYdX tx builder: order type {:?} is not yet supported via \
                 the automatic tx building path. Use place_order_grpc() directly.",
                req.order_type
            )));
        }
    };

    // Use dynamic step sizes from market config when available.
    // Fallback to BTC-USD defaults only as a last resort.
    let (step_base_quantums, subticks_per_tick) = if let Some(cfg) = market_cfg {
        (cfg.step_base_quantums, cfg.subticks_per_tick)
    } else {
        // Fallback: BTC-USD defaults (stepBaseQuantum=1e6, subticksPerTick=1e5)
        tracing::warn!(
            "order_request_to_quantums_subticks: no market config for '{}', \
             using BTC-USD fallback step sizes. Fetch market config first for accuracy.",
            req.symbol.base
        );
        (1_000_000.0_f64, 100_000.0_f64)
    };

    // quantums = round(quantity / step_base_quantums)
    let quantums = (req.quantity / step_base_quantums).round() as u64;
    // subticks  = round(price   * subticks_per_tick)
    let subticks = (price * subticks_per_tick).round() as u64;

    if quantums == 0 {
        return Err(ExchangeError::InvalidRequest(format!(
            "dYdX tx builder: computed quantums = 0 for quantity {} with \
             step_base_quantums = {}; quantity is too small.",
            req.quantity, step_base_quantums
        )));
    }

    Ok((quantums, subticks))
}

/// Map `TimeInForce` to the dYdX `OrderTimeInForce` i32 value.
#[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
fn map_tif_to_dydx_i32(tif: &crate::core::TimeInForce) -> i32 {
    use super::proto::dydxprotocol::OrderTimeInForce;
    match tif {
        crate::core::TimeInForce::Gtc
        | crate::core::TimeInForce::Gtd
        | crate::core::TimeInForce::GoodTilBlock { .. } => {
            OrderTimeInForce::Unspecified as i32 // GTC on dYdX
        }
        crate::core::TimeInForce::Ioc => OrderTimeInForce::Ioc as i32,
        crate::core::TimeInForce::Fok => OrderTimeInForce::FillOrKill as i32,
        crate::core::TimeInForce::PostOnly => OrderTimeInForce::PostOnly as i32,
    }
}
