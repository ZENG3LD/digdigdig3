//! Futu OpenAPI connector implementation
//!
//! **STUB IMPLEMENTATION**
//!
//! This connector documents that Futu OpenAPI requires a different implementation
//! approach due to its custom TCP + Protocol Buffer architecture.

use async_trait::async_trait;
use reqwest::Client;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;

/// Futu OpenAPI connector (stub implementation)
///
/// This is a placeholder that returns `UnsupportedOperation` for all methods.
///
/// To actually use Futu OpenAPI, you need to:
/// 1. Run OpenD gateway (download from https://www.futunn.com/en/download/OpenAPI)
/// 2. Implement Protocol Buffer client in Rust, OR
/// 3. Use Futu's Python SDK via FFI/PyO3, OR
/// 4. Create a REST adapter service using Futu Python SDK
pub struct FutuConnector {
    _client: Client,
    _auth: FutuAuth,
    _endpoints: FutuEndpoints,
}

impl FutuConnector {
    /// Create new connector (stub)
    pub fn new(auth: FutuAuth) -> Self {
        Self {
            _client: Client::new(),
            _auth: auth,
            _endpoints: FutuEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(FutuAuth::from_env())
    }

    /// Generate a helpful error message
    fn not_implemented_error() -> ExchangeError {
        ExchangeError::UnsupportedOperation(
            "Futu OpenAPI uses custom TCP + Protocol Buffers (not HTTP REST). \
             \n\nTo integrate Futu: \
             \n1. Download OpenD: https://www.futunn.com/en/download/OpenAPI \
             \n2. Choose integration method: \
             \n   - Implement Protocol Buffer client in Rust \
             \n   - Use Python SDK via PyO3/FFI \
             \n   - Create REST adapter with Python SDK \
             \n3. See research docs in: src/stocks/china/futu/research/"
                .to_string()
        )
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
        false // Futu uses TrdEnv.SIMULATE for paper trading
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // Futu supports Cash, Margin, Universal
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for FutuConnector {
    async fn get_price(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        Err(Self::not_implemented_error())
    }

    async fn get_ticker(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        Err(Self::not_implemented_error())
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(Self::not_implemented_error())
    }

    async fn get_klines(
        &self,
        _symbol: Symbol,
        _interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        Err(Self::not_implemented_error())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        Err(Self::not_implemented_error())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for FutuConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "Not supported".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Not supported".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Not supported".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Not supported".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for FutuConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(Self::not_implemented_error())
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(Self::not_implemented_error())
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for FutuConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu does not trade perpetual futures - funding rate not applicable".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu does not trade perpetual futures - funding rate not applicable".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu does not trade perpetual futures - funding rate not applicable".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Extended methods (Futu-specific features)
// ═══════════════════════════════════════════════════════════════════════════

impl FutuConnector {
    /// Get broker queue data (HK LV2 only)
    pub async fn get_broker_queue(&self, _symbol: Symbol) -> ExchangeResult<BrokerQueue> {
        Err(Self::not_implemented_error())
    }

    /// Get capital flow data (HK market)
    pub async fn get_capital_flow(&self, _symbol: Symbol) -> ExchangeResult<CapitalFlow> {
        Err(Self::not_implemented_error())
    }

    /// Get options chain
    pub async fn get_option_chain(&self, _underlying: &str) -> ExchangeResult<OptionChain> {
        Err(Self::not_implemented_error())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stub types for extended methods
// ═══════════════════════════════════════════════════════════════════════════

/// Broker queue data (stub)
#[derive(Debug, Clone)]
pub struct BrokerQueue {
    pub symbol: String,
    pub bid_brokers: Vec<BrokerInfo>,
    pub ask_brokers: Vec<BrokerInfo>,
}

/// Broker info (stub)
#[derive(Debug, Clone)]
pub struct BrokerInfo {
    pub broker_id: u32,
    pub broker_name: String,
    pub position: u32,
}

/// Capital flow data (stub)
#[derive(Debug, Clone)]
pub struct CapitalFlow {
    pub symbol: String,
    pub main_inflow: f64,
    pub main_outflow: f64,
    pub medium_inflow: f64,
    pub medium_outflow: f64,
    pub small_inflow: f64,
    pub small_outflow: f64,
}

/// Option chain data (stub)
#[derive(Debug, Clone)]
pub struct OptionChain {
    pub underlying: String,
    pub expirations: Vec<String>,
    pub strikes: Vec<f64>,
}
