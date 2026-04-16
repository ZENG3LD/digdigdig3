//! # Connector Registry
//!
//! Static metadata registry for all connectors.
//!
//! Provides O(1) lookup by ExchangeId and filtering by category/type.

use std::collections::HashMap;
use crate::core::types::{ExchangeId, ExchangeType};

// ═══════════════════════════════════════════════════════════════════════════════
// SUPPORTING TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Connector category for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectorCategory {
    /// Centralized crypto exchange (Binance, Bybit, etc.)
    CryptoExchangeCex,
    /// Decentralized crypto exchange (dYdX, Lighter, Paradex)
    CryptoExchangeDex,
    /// US stock market data/broker (Polygon, Alpaca, etc.)
    StockMarketUS,
    /// Indian stock market broker (Zerodha, AngelOne, etc.)
    StockMarketIndia,
    /// Japanese stock market data (JQuants)
    StockMarketJapan,
    /// Korean stock market data (Krx)
    StockMarketKorea,
    /// Russian stock market broker/data (Moex, Tinkoff)
    StockMarketRussia,
    /// Forex broker/data provider (Oanda, Dukascopy, etc.)
    Forex,
    /// Specialized data feed (WhaleAlert, Fred, Coinglass, etc.)
    DataFeed,
    /// Multi-asset broker (IB)
    Broker,
    /// Read-only market data provider (YahooFinance, CryptoCompare)
    DataProvider,
}

/// Supported features for a connector
#[derive(Debug, Clone, Copy)]
pub struct Features {
    /// Supports market data (prices, orderbook, klines)
    pub market_data: bool,
    /// Supports trading (place/cancel orders)
    pub trading: bool,
    /// Supports account info (balances)
    pub account: bool,
    /// Supports positions (futures/margin)
    pub positions: bool,
    /// Supports WebSocket streaming
    pub websocket: bool,
    /// WebSocket kline/candlestick channel
    pub ws_klines: bool,
    /// WebSocket trades channel
    pub ws_trades: bool,
    /// WebSocket order book channel
    pub ws_orderbook: bool,
    /// WebSocket ticker channel
    pub ws_ticker: bool,
    // ── Optional operation traits ────────────────────────────────────────────
    /// Implements CancelAll trait (cancel all open orders at once)
    pub cancel_all: bool,
    /// Implements AmendOrder trait (modify existing order price/qty)
    pub amend_order: bool,
    /// Implements BatchOrders trait (place/cancel orders in bulk)
    pub batch_orders: bool,
    /// Implements AccountTransfers trait (internal sub-account transfers)
    pub account_transfers: bool,
    /// Implements CustodialFunds trait (deposit/withdraw to/from exchange)
    pub custodial_funds: bool,
    /// Implements SubAccounts trait (sub-account management)
    pub sub_accounts: bool,
    /// Implements MarginTrading trait (margin borrow/repay)
    pub margin_trading: bool,
    /// Implements TriggerOrders trait (stop/take-profit conditional orders)
    pub trigger_orders: bool,
    /// Implements ConvertSwap trait (instant coin conversion)
    pub convert_swap: bool,
    /// Implements EarnStaking trait (staking/savings products)
    pub earn_staking: bool,
    /// Implements CopyTrading trait (copy-trade other users)
    pub copy_trading: bool,
}

impl Features {
    /// Full feature set (CEX with all capabilities)
    pub const fn full() -> Self {
        Self {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        }
    }

    /// Data provider only (no trading)
    pub const fn data_only() -> Self {
        Self {
            market_data: true,
            trading: false,
            account: false,
            positions: false,
            websocket: false,
            ws_klines: false,
            ws_trades: false,
            ws_orderbook: false,
            ws_ticker: false,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        }
    }

    /// Data provider with WebSocket
    pub const fn data_with_ws() -> Self {
        Self {
            market_data: true,
            trading: false,
            account: false,
            positions: false,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        }
    }

    /// Broker (trading but no positions)
    pub const fn broker() -> Self {
        Self {
            market_data: true,
            trading: true,
            account: true,
            positions: false,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        }
    }

    /// Spot exchange (no positions)
    pub const fn spot_exchange() -> Self {
        Self {
            market_data: true,
            trading: true,
            account: true,
            positions: false,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        }
    }

    /// DEX (trading but limited account info)
    pub const fn dex() -> Self {
        Self {
            market_data: true,
            trading: true,
            account: false,
            positions: false,
            websocket: false,
            ws_klines: false,
            ws_trades: false,
            ws_orderbook: false,
            ws_ticker: false,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        }
    }
}

/// Authentication type required by connector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    /// API key + secret key (most common)
    ApiKey,
    /// OAuth 2.0 flow
    OAuth2,
    /// Time-based One-Time Password (e.g., AngelOne)
    TOTP,
    /// Basic HTTP authentication
    BasicAuth,
    /// Bearer token (JWT)
    BearerToken,
    /// No authentication required (public data only)
    None,
}

/// Classification of rate limiter implementation model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimiterModel {
    /// Simple request counter per window (e.g., OKX 20/2s)
    SimpleCounter,
    /// Weight-based budget per window (e.g., Binance 6000w/60s)
    WeightBased,
    /// Continuous decay counter (e.g., Kraken Spot, Deribit)
    DecayingCounter,
    /// Multiple independent pools (e.g., Upbit, Paradex)
    GroupBased,
    /// No documented limits / unknown
    Unknown,
}

/// Description of a single rate limit group/pool
#[derive(Debug, Clone, Copy)]
pub struct RateLimitGroup {
    /// Group name (e.g., "public", "private", "spot", "orders", "CONTRACT")
    pub name: &'static str,
    /// Maximum value (requests or weight) per window
    pub max_value: u32,
    /// Window duration in seconds
    pub window_seconds: u32,
    /// true = weight-based, false = simple count
    pub is_weight: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Copy)]
pub struct RateLimits {
    /// Max requests per second (legacy display field)
    pub requests_per_second: Option<u32>,
    /// Max requests per minute (legacy display field)
    pub requests_per_minute: Option<u32>,
    /// Weight-based limit per minute - Binance-style (legacy display field)
    pub weight_per_minute: Option<u32>,
    /// Actual window duration in seconds used by the runtime limiter
    pub window_seconds: u32,
    /// What limiter model this exchange uses
    pub limiter_model: LimiterModel,
    /// Rate limit groups (empty = single limiter described by legacy fields)
    pub groups: &'static [RateLimitGroup],
    /// Whether server returns rate limit headers
    pub has_server_headers: bool,
}

impl RateLimits {
    /// No rate limits defined
    pub const fn none() -> Self {
        Self {
            requests_per_second: None,
            requests_per_minute: None,
            weight_per_minute: None,
            window_seconds: 60,
            limiter_model: LimiterModel::Unknown,
            groups: &[],
            has_server_headers: false,
        }
    }

    /// Standard rate limit (SimpleCounter, 60-second window, no server headers)
    pub const fn standard(rps: u32, rpm: u32) -> Self {
        Self {
            requests_per_second: Some(rps),
            requests_per_minute: Some(rpm),
            weight_per_minute: None,
            window_seconds: 60,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: false,
        }
    }

    /// Weight-based rate limit (Binance-style)
    pub const fn weight_based(rps: u32, rpm: u32, wpm: u32) -> Self {
        Self {
            requests_per_second: Some(rps),
            requests_per_minute: Some(rpm),
            weight_per_minute: Some(wpm),
            window_seconds: 60,
            limiter_model: LimiterModel::WeightBased,
            groups: &[],
            has_server_headers: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR METADATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Static metadata for a connector
#[derive(Debug, Clone)]
pub struct ConnectorMetadata {
    /// Unique identifier
    pub id: ExchangeId,
    /// Human-readable name
    pub name: &'static str,
    /// Exchange type (Cex, Dex, Broker, DataProvider)
    pub exchange_type: ExchangeType,
    /// Category for grouping
    pub category: ConnectorCategory,
    /// Supported features
    pub supported_features: Features,
    /// Authentication type
    pub authentication: AuthType,
    /// Rate limits
    pub rate_limits: RateLimits,
    /// Base REST API URL
    pub base_url: &'static str,
    /// WebSocket URL (if supported)
    pub websocket_url: Option<&'static str>,
    /// Official API documentation URL
    pub documentation_url: Option<&'static str>,
    /// Whether an API key is required to access market data (klines, ticker, orderbook)
    pub requires_api_key_for_data: bool,
    /// Whether an API key is required for trading and account operations
    pub requires_api_key_for_trading: bool,
    /// Whether a free tier is available
    pub free_tier: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATIC RATE LIMIT GROUPS
// ═══════════════════════════════════════════════════════════════════════════════

static COINBASE_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "public",  max_value: 10,  window_seconds: 1,  is_weight: false },
    RateLimitGroup { name: "private", max_value: 30,  window_seconds: 1,  is_weight: false },
];

static GATEIO_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "spot",    max_value: 200, window_seconds: 10, is_weight: false },
    RateLimitGroup { name: "futures", max_value: 200, window_seconds: 10, is_weight: false },
];

static HTX_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "spot_pub", max_value: 100, window_seconds: 10, is_weight: false },
];

static BITGET_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "market",  max_value: 20, window_seconds: 1, is_weight: false },
    RateLimitGroup { name: "trading", max_value: 10, window_seconds: 1, is_weight: false },
];

static BINGX_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "market", max_value: 100, window_seconds: 10, is_weight: false },
];


static UPBIT_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "market",  max_value: 10, window_seconds: 1, is_weight: false },
    RateLimitGroup { name: "account", max_value: 30, window_seconds: 1, is_weight: false },
    RateLimitGroup { name: "order",   max_value: 8,  window_seconds: 1, is_weight: false },
];

static GEMINI_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "public",  max_value: 120, window_seconds: 60, is_weight: false },
    RateLimitGroup { name: "private", max_value: 600, window_seconds: 60, is_weight: false },
];

static PARADEX_GROUPS: &[RateLimitGroup] = &[
    RateLimitGroup { name: "public",       max_value: 1500,  window_seconds: 60, is_weight: false },
    RateLimitGroup { name: "orders",       max_value: 17250, window_seconds: 60, is_weight: false },
    RateLimitGroup { name: "private_gets", max_value: 600,   window_seconds: 60, is_weight: false },
];

// ═══════════════════════════════════════════════════════════════════════════════
// STATIC METADATA ARRAY
// ═══════════════════════════════════════════════════════════════════════════════

/// Static array of all connector metadata.
/// Lives in .rodata segment - zero heap allocations.
static CONNECTOR_METADATA_ARRAY: &[ConnectorMetadata] = &[
    // ═══════════════════════════════════════════════════════════════════════════
    // CEX - Centralized Exchanges (17 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Binance,
        name: "Binance",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(10),
            requests_per_minute: Some(1200),
            weight_per_minute: Some(6000),
            window_seconds: 60,
            limiter_model: LimiterModel::WeightBased,
            groups: &[],
            has_server_headers: true,
        },
        base_url: "https://api.binance.com",
        websocket_url: Some("wss://stream.binance.com:9443"),
        documentation_url: Some("https://binance-docs.github.io/apidocs/spot/en/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Bybit,
        name: "Bybit",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(120),
            weight_per_minute: None,
            window_seconds: 5,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: true,
        },
        base_url: "https://api.bybit.com",
        websocket_url: Some("wss://stream.bybit.com/v5/public/spot"),
        documentation_url: Some("https://bybit-exchange.github.io/docs/v5/intro"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::OKX,
        name: "OKX",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(10),
            requests_per_minute: Some(600),
            weight_per_minute: None,
            window_seconds: 2,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://www.okx.com",
        websocket_url: Some("wss://ws.okx.com:8443/ws/v5/public"),
        documentation_url: Some("https://www.okx.com/docs-v5/en/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::KuCoin,
        name: "KuCoin",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(120),
            weight_per_minute: Some(4000),
            window_seconds: 30,
            limiter_model: LimiterModel::WeightBased,
            groups: &[],
            has_server_headers: true,
        },
        base_url: "https://api.kucoin.com",
        websocket_url: Some("wss://ws-api-spot.kucoin.com"),
        documentation_url: Some("https://docs.kucoin.com/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Kraken,
        name: "Kraken",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(1),
            requests_per_minute: Some(60),
            weight_per_minute: None,
            window_seconds: 0,
            limiter_model: LimiterModel::DecayingCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://api.kraken.com",
        websocket_url: Some("wss://ws.kraken.com"),
        documentation_url: Some("https://docs.kraken.com/rest/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Coinbase,
        name: "Coinbase",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(10),
            requests_per_minute: Some(600),
            weight_per_minute: None,
            window_seconds: 1,
            limiter_model: LimiterModel::SimpleCounter,
            groups: COINBASE_GROUPS,
            has_server_headers: true,
        },
        base_url: "https://api.coinbase.com",
        websocket_url: Some("wss://ws-feed.exchange.coinbase.com"),
        documentation_url: Some("https://docs.cdp.coinbase.com/exchange/docs/welcome/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::GateIO,
        name: "Gate.io",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(1200),
            weight_per_minute: None,
            window_seconds: 10,
            limiter_model: LimiterModel::SimpleCounter,
            groups: GATEIO_GROUPS,
            has_server_headers: true,
        },
        base_url: "https://api.gateio.ws",
        websocket_url: Some("wss://api.gateio.ws/ws/v4/"),
        documentation_url: Some("https://www.gate.io/docs/developers/apiv4/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Bitfinex,
        name: "Bitfinex",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(90),
            weight_per_minute: None,
            window_seconds: 60,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://api-pub.bitfinex.com",
        websocket_url: Some("wss://api-pub.bitfinex.com/ws/2"),
        documentation_url: Some("https://docs.bitfinex.com/docs"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Bitstamp,
        name: "Bitstamp",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: false,
            websocket: true,
            ws_klines: false,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(10),
            requests_per_minute: Some(600),
            weight_per_minute: None,
            window_seconds: 1,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://www.bitstamp.net",
        websocket_url: Some("wss://ws.bitstamp.net"),
        documentation_url: Some("https://www.bitstamp.net/api/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Gemini,
        name: "Gemini",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: false,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(120),
            weight_per_minute: None,
            window_seconds: 60,
            limiter_model: LimiterModel::SimpleCounter,
            groups: GEMINI_GROUPS,
            has_server_headers: false,
        },
        base_url: "https://api.gemini.com",
        websocket_url: Some("wss://api.gemini.com/v1/marketdata"),
        documentation_url: Some("https://docs.gemini.com/rest-api/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::MEXC,
        name: "MEXC",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: false,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(1200),
            weight_per_minute: Some(7200),
            window_seconds: 10,
            limiter_model: LimiterModel::WeightBased,
            groups: &[],
            has_server_headers: true,
        },
        base_url: "https://api.mexc.com",
        websocket_url: Some("wss://wbs.mexc.com/ws"),
        documentation_url: Some("https://mexcdevelop.github.io/apidocs/spot_v3_en/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::HTX,
        name: "HTX (Huobi)",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: false,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(600),
            weight_per_minute: None,
            window_seconds: 10,
            limiter_model: LimiterModel::SimpleCounter,
            groups: HTX_GROUPS,
            has_server_headers: true,
        },
        base_url: "https://api.huobi.pro",
        websocket_url: Some("wss://api.huobi.pro/ws"),
        documentation_url: Some("https://www.htx.com/en-us/opend/newApiPages/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Bitget,
        name: "Bitget",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(20),
            requests_per_minute: Some(1200),
            weight_per_minute: None,
            window_seconds: 1,
            limiter_model: LimiterModel::SimpleCounter,
            groups: BITGET_GROUPS,
            has_server_headers: true,
        },
        base_url: "https://api.bitget.com",
        websocket_url: Some("wss://ws.bitget.com/v2/ws/public"),
        documentation_url: Some("https://www.bitget.com/api-doc/common/intro"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::BingX,
        name: "BingX",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(600),
            weight_per_minute: None,
            window_seconds: 10,
            limiter_model: LimiterModel::SimpleCounter,
            groups: BINGX_GROUPS,
            has_server_headers: false,
        },
        base_url: "https://open-api.bingx.com",
        websocket_url: Some("wss://open-api-ws.bingx.com/market"),
        documentation_url: Some("https://bingx-api.github.io/docs/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::CryptoCom,
        name: "Crypto.com",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: true,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(3),
            requests_per_minute: Some(180),
            weight_per_minute: None,
            window_seconds: 1,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://api.crypto.com",
        websocket_url: Some("wss://stream.crypto.com/exchange/v1/market"),
        documentation_url: Some("https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Upbit,
        name: "Upbit",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: false,
            websocket: true,
            ws_klines: false,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(10),
            requests_per_minute: Some(600),
            weight_per_minute: None,
            window_seconds: 1,
            limiter_model: LimiterModel::GroupBased,
            groups: UPBIT_GROUPS,
            has_server_headers: true,
        },
        base_url: "https://api.upbit.com",
        websocket_url: Some("wss://api.upbit.com/websocket/v1"),
        documentation_url: Some("https://docs.upbit.com/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // DERIVATIVES (2 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Deribit,
        name: "Deribit",
        exchange_type: ExchangeType::Cex,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: true,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(20),
            requests_per_minute: Some(1000),
            weight_per_minute: None,
            window_seconds: 0,
            limiter_model: LimiterModel::DecayingCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://www.deribit.com",
        websocket_url: Some("wss://www.deribit.com/ws/api/v2"),
        documentation_url: Some("https://docs.deribit.com/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::HyperLiquid,
        name: "HyperLiquid",
        exchange_type: ExchangeType::Hybrid,
        category: ConnectorCategory::CryptoExchangeCex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: true,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: Some(20),
            requests_per_minute: Some(1200),
            weight_per_minute: Some(1200),
            window_seconds: 60,
            limiter_model: LimiterModel::WeightBased,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://api.hyperliquid.xyz",
        websocket_url: Some("wss://api.hyperliquid.xyz/ws"),
        documentation_url: Some("https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // DEX - Decentralized Exchanges (7 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Lighter,
        name: "Lighter",
        exchange_type: ExchangeType::Dex,
        category: ConnectorCategory::CryptoExchangeDex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: false,
            positions: false,
            websocket: true,
            ws_klines: false,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::None,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: None,
            weight_per_minute: Some(10_000),
            window_seconds: 60,
            limiter_model: LimiterModel::WeightBased,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://api.lighter.xyz",
        websocket_url: Some("wss://mainnet.zklighter.elliot.ai/stream"),
        documentation_url: Some("https://docs.lighter.xyz/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Paradex,
        name: "Paradex",
        exchange_type: ExchangeType::Dex,
        category: ConnectorCategory::CryptoExchangeDex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: false,
            positions: false,
            websocket: true,
            ws_klines: false,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: true,
            amend_order: true,
            batch_orders: true,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(1500),
            weight_per_minute: None,
            window_seconds: 60,
            limiter_model: LimiterModel::GroupBased,
            groups: PARADEX_GROUPS,
            has_server_headers: true,
        },
        base_url: "https://api.paradex.trade",
        websocket_url: Some("wss://ws.paradex.trade/v1"),
        documentation_url: Some("https://docs.paradex.trade/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Dydx,
        name: "dYdX",
        exchange_type: ExchangeType::Dex,
        category: ConnectorCategory::CryptoExchangeDex,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: false,
            positions: false,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits {
            requests_per_second: None,
            requests_per_minute: Some(360),
            weight_per_minute: None,
            window_seconds: 10,
            limiter_model: LimiterModel::SimpleCounter,
            groups: &[],
            has_server_headers: false,
        },
        base_url: "https://api.dydx.exchange",
        websocket_url: Some("wss://api.dydx.exchange/v3/ws"),
        documentation_url: Some("https://docs.dydx.exchange/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: true,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCK MARKET - US (5 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Polygon,
        name: "Polygon.io",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketUS,
        supported_features: Features::data_with_ws(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(5, 5),
        base_url: "https://api.polygon.io",
        websocket_url: Some("wss://socket.polygon.io"),
        documentation_url: Some("https://polygon.io/docs/stocks"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Finnhub,
        name: "Finnhub",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketUS,
        supported_features: Features::data_with_ws(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(1, 60),
        base_url: "https://finnhub.io",
        websocket_url: Some("wss://ws.finnhub.io"),
        documentation_url: Some("https://finnhub.io/docs/api"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Tiingo,
        name: "Tiingo",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketUS,
        supported_features: Features::data_with_ws(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(50, 1000),
        base_url: "https://api.tiingo.com",
        websocket_url: Some("wss://api.tiingo.com/iex"),
        documentation_url: Some("https://www.tiingo.com/documentation/general/overview"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Twelvedata,
        name: "Twelve Data",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketUS,
        supported_features: Features::data_with_ws(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(8, 800),
        base_url: "https://api.twelvedata.com",
        websocket_url: Some("wss://ws.twelvedata.com"),
        documentation_url: Some("https://twelvedata.com/docs"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Alpaca,
        name: "Alpaca",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketUS,
        supported_features: Features::broker(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(3, 200),
        base_url: "https://api.alpaca.markets",
        websocket_url: Some("wss://stream.data.alpaca.markets/v2"),
        documentation_url: Some("https://docs.alpaca.markets/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCK MARKET - INDIA (5 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::AngelOne,
        name: "Angel One",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketIndia,
        supported_features: Features::broker(),
        authentication: AuthType::TOTP,
        rate_limits: RateLimits::standard(10, 600),
        base_url: "https://apiconnect.angelbroking.com",
        websocket_url: Some("wss://smartapisocket.angelone.in/smart-stream"),
        documentation_url: Some("https://smartapi.angelbroking.com/docs"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Zerodha,
        name: "Zerodha (Kite)",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketIndia,
        supported_features: Features::broker(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(3, 180),
        base_url: "https://api.kite.trade",
        websocket_url: Some("wss://ws.kite.trade"),
        documentation_url: Some("https://kite.trade/docs/connect/v3/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: false,
    },
    ConnectorMetadata {
        id: ExchangeId::Upstox,
        name: "Upstox",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketIndia,
        supported_features: Features::broker(),
        authentication: AuthType::OAuth2,
        rate_limits: RateLimits::standard(25, 1000),
        base_url: "https://api.upstox.com",
        websocket_url: Some("wss://api.upstox.com/v2/feed/market-data-feed"),
        documentation_url: Some("https://upstox.com/developer/api-documentation/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: false,
    },
    ConnectorMetadata {
        id: ExchangeId::Dhan,
        name: "Dhan",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketIndia,
        supported_features: Features::broker(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(10, 600),
        base_url: "https://api.dhan.co",
        websocket_url: Some("wss://api-feed.dhan.co"),
        documentation_url: Some("https://dhanhq.co/docs/v2/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Fyers,
        name: "Fyers",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketIndia,
        supported_features: Features::broker(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(10, 600),
        base_url: "https://api-t1.fyers.in",
        websocket_url: Some("wss://api-t1.fyers.in/socket/v2"),
        documentation_url: Some("https://fyers.in/api-documentation/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCK MARKET - JAPAN (1 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::JQuants,
        name: "J-Quants",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketJapan,
        supported_features: Features::data_only(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(10, 600),
        base_url: "https://api.jquants.com",
        websocket_url: None,
        documentation_url: Some("https://jpx.gitbook.io/j-quants-en"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCK MARKET - KOREA (1 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Krx,
        name: "Korea Exchange (KRX)",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketKorea,
        supported_features: Features::data_only(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(5, 100),
        base_url: "https://data.krx.co.kr",
        websocket_url: None,
        documentation_url: Some("https://data.krx.co.kr/contents/MDC/MAIN/main/index.cmd"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCK MARKET - RUSSIA (2 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Moex,
        name: "Moscow Exchange (MOEX)",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::StockMarketRussia,
        supported_features: Features {
            market_data: true,
            trading: false,
            account: false,
            positions: false,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::None,
        rate_limits: RateLimits::standard(10, 600),
        base_url: "https://iss.moex.com",
        websocket_url: Some("wss://iss.moex.com/infocx/v3/websocket"),
        documentation_url: Some("https://www.moex.com/a2193"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Tinkoff,
        name: "Tinkoff Invest",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::StockMarketRussia,
        supported_features: Features::broker(),
        authentication: AuthType::BearerToken,
        rate_limits: RateLimits::standard(100, 300),
        base_url: "https://invest-public-api.tinkoff.ru",
        websocket_url: Some("wss://invest-public-api.tinkoff.ru/ws"),
        documentation_url: Some("https://tinkoff.github.io/investAPI/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // FOREX (3 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Oanda,
        name: "OANDA",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::Forex,
        supported_features: Features::broker(),
        authentication: AuthType::BearerToken,
        rate_limits: RateLimits::standard(10, 120),
        base_url: "https://api-fxtrade.oanda.com",
        websocket_url: Some("wss://stream-fxtrade.oanda.com"),
        documentation_url: Some("https://developer.oanda.com/rest-live-v20/introduction/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::Dukascopy,
        name: "Dukascopy",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::Forex,
        supported_features: Features::data_only(),
        authentication: AuthType::None,
        rate_limits: RateLimits::standard(5, 300),
        base_url: "https://datafeed.dukascopy.com",
        websocket_url: None,
        documentation_url: Some("https://www.dukascopy.com/swiss/english/marketwatch/historical/"),
        requires_api_key_for_data: false,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::AlphaVantage,
        name: "Alpha Vantage",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::Forex,
        supported_features: Features::data_only(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(1, 5),
        base_url: "https://www.alphavantage.co",
        websocket_url: None,
        documentation_url: Some("https://www.alphavantage.co/documentation/"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // PREDICTION (1 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Polymarket,
        name: "Polymarket",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::DataFeed,
        supported_features: Features {
            market_data: true,
            trading: false,
            account: false,
            positions: false,
            websocket: true,
            ws_klines: false,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: false,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::ApiKey, // L2 HMAC-SHA256 (optional for public data)
        rate_limits: RateLimits::standard(10, 500), // ~500 req/min undocumented
        base_url: "https://clob.polymarket.com",
        websocket_url: Some("wss://ws-subscriptions-clob.polymarket.com/ws/market"),
        documentation_url: Some("https://docs.polymarket.com/"),
        requires_api_key_for_data: false, // Public data is accessible without auth
        requires_api_key_for_trading: false,
        free_tier: true,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // AGGREGATORS (4 active)
    // ═══════════════════════════════════════════════════════════════════════════
    ConnectorMetadata {
        id: ExchangeId::Ib,
        name: "Interactive Brokers",
        exchange_type: ExchangeType::Broker,
        category: ConnectorCategory::Broker,
        supported_features: Features {
            market_data: true,
            trading: true,
            account: true,
            positions: true,
            websocket: true,
            ws_klines: true,
            ws_trades: true,
            ws_orderbook: true,
            ws_ticker: true,
            cancel_all: false,
            amend_order: false,
            batch_orders: false,
            account_transfers: false,
            custodial_funds: false,
            sub_accounts: false,
            margin_trading: false,
            trigger_orders: false,
            convert_swap: false,
            earn_staking: false,
            copy_trading: false,
        },
        authentication: AuthType::OAuth2,
        rate_limits: RateLimits::none(),
        base_url: "https://localhost:5000/v1/api", // Gateway default, can use https://api.ibkr.com/v1/api for OAuth
        websocket_url: Some("wss://localhost:5000/v1/api/ws"),
        documentation_url: Some("https://www.interactivebrokers.com/api/doc.html"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: true,
        free_tier: false,
    },
    ConnectorMetadata {
        id: ExchangeId::YahooFinance,
        name: "Yahoo Finance",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::DataProvider,
        supported_features: Features::data_only(),
        authentication: AuthType::None,
        rate_limits: RateLimits::standard(5, 100),
        base_url: "https://query1.finance.yahoo.com",
        websocket_url: None,
        documentation_url: None,
        requires_api_key_for_data: false,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
    ConnectorMetadata {
        id: ExchangeId::CryptoCompare,
        name: "CryptoCompare",
        exchange_type: ExchangeType::DataProvider,
        category: ConnectorCategory::DataProvider,
        supported_features: Features::data_with_ws(),
        authentication: AuthType::ApiKey,
        rate_limits: RateLimits::standard(10, 250000),
        base_url: "https://min-api.cryptocompare.com",
        websocket_url: Some("wss://streamer.cryptocompare.com/v2"),
        documentation_url: Some("https://min-api.cryptocompare.com/documentation"),
        requires_api_key_for_data: true,
        requires_api_key_for_trading: false,
        free_tier: true,
    },
];

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Registry for connector metadata with O(1) lookup
pub struct ConnectorRegistry {
    /// HashMap for fast lookup by ExchangeId
    lookup: HashMap<ExchangeId, &'static ConnectorMetadata>,
}

impl ConnectorRegistry {
    /// Create a new registry from static metadata
    pub fn new() -> Self {
        let lookup = CONNECTOR_METADATA_ARRAY
            .iter()
            .map(|meta| (meta.id, meta))
            .collect();
        Self { lookup }
    }

    /// Get metadata by ExchangeId
    pub fn get(&self, id: &ExchangeId) -> Option<&'static ConnectorMetadata> {
        self.lookup.get(id).copied()
    }

    /// Iterate over all metadata entries
    pub fn iter(&self) -> impl Iterator<Item = &'static ConnectorMetadata> {
        CONNECTOR_METADATA_ARRAY.iter()
    }

    /// List all connector metadata
    pub fn list_all(&self) -> Vec<&'static ConnectorMetadata> {
        CONNECTOR_METADATA_ARRAY.iter().collect()
    }

    /// List connectors by category
    pub fn list_by_category(&self, category: ConnectorCategory) -> Vec<&'static ConnectorMetadata> {
        CONNECTOR_METADATA_ARRAY
            .iter()
            .filter(|m| m.category == category)
            .collect()
    }

    /// List connectors by exchange type
    pub fn list_by_type(&self, exchange_type: ExchangeType) -> Vec<&'static ConnectorMetadata> {
        CONNECTOR_METADATA_ARRAY
            .iter()
            .filter(|m| m.exchange_type == exchange_type)
            .collect()
    }

    /// List connectors that support trading
    pub fn list_with_trading(&self) -> Vec<&'static ConnectorMetadata> {
        CONNECTOR_METADATA_ARRAY
            .iter()
            .filter(|m| m.supported_features.trading)
            .collect()
    }

    /// List connectors that support WebSocket
    pub fn list_with_websocket(&self) -> Vec<&'static ConnectorMetadata> {
        CONNECTOR_METADATA_ARRAY
            .iter()
            .filter(|m| m.supported_features.websocket)
            .collect()
    }

    /// Get total count of connectors
    pub fn count(&self) -> usize {
        CONNECTOR_METADATA_ARRAY.len()
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that Default trait creates a valid registry
    #[test]
    fn test_registry_default() {
        let registry = ConnectorRegistry::default();

        // Default should create same registry as new()
        assert_eq!(registry.count(), CONNECTOR_METADATA_ARRAY.len());
        assert!(registry.count() > 0, "Default registry should not be empty");
    }

    /// Test that new() creates a valid registry
    #[test]
    fn test_registry_new() {
        let registry = ConnectorRegistry::new();

        // Registry should be populated
        assert!(registry.count() > 0);
        assert_eq!(registry.count(), CONNECTOR_METADATA_ARRAY.len());
    }

    /// Test that list_all() returns exactly 47 entries
    #[test]
    fn test_registry_count() {
        let registry = ConnectorRegistry::new();

        // Count should match CONNECTOR_METADATA_ARRAY length
        assert_eq!(registry.count(), CONNECTOR_METADATA_ARRAY.len());

        // Verify we have exactly 47 connectors
        assert_eq!(registry.count(), 47, "Should have exactly 47 active connectors");

        // list_all() should return same count
        let all = registry.list_all();
        assert_eq!(all.len(), 47, "list_all() should return 47 entries");
    }

    /// Test get() for Binance connector
    #[test]
    fn test_registry_get_binance() {
        let registry = ConnectorRegistry::new();

        let binance = registry.get(&ExchangeId::Binance);
        assert!(binance.is_some(), "Binance should be in registry");

        let meta = binance.unwrap();
        assert_eq!(meta.id, ExchangeId::Binance);
        assert_eq!(meta.name, "Binance");
        assert_eq!(meta.exchange_type, ExchangeType::Cex);
        assert_eq!(meta.category, ConnectorCategory::CryptoExchangeCex);
        assert!(meta.supported_features.market_data);
        assert!(meta.supported_features.trading);
        assert_eq!(meta.base_url, "https://api.binance.com");
    }

    /// Test get() for OKX connector
    #[test]
    fn test_registry_get_okx() {
        let registry = ConnectorRegistry::new();

        let okx = registry.get(&ExchangeId::OKX);
        assert!(okx.is_some(), "OKX should be in registry");

        let meta = okx.unwrap();
        assert_eq!(meta.id, ExchangeId::OKX);
        assert_eq!(meta.name, "OKX");
        assert_eq!(meta.exchange_type, ExchangeType::Cex);
        assert_eq!(meta.category, ConnectorCategory::CryptoExchangeCex);
        assert!(meta.supported_features.websocket);
    }

    /// Test get() for Alpaca (US Stock) connector
    #[test]
    fn test_registry_get_alpaca() {
        let registry = ConnectorRegistry::new();

        let alpaca = registry.get(&ExchangeId::Alpaca);
        assert!(alpaca.is_some(), "Alpaca should be in registry");

        let meta = alpaca.unwrap();
        assert_eq!(meta.id, ExchangeId::Alpaca);
        assert_eq!(meta.name, "Alpaca");
        assert_eq!(meta.exchange_type, ExchangeType::Broker);
        assert_eq!(meta.category, ConnectorCategory::StockMarketUS);
        assert!(meta.supported_features.trading, "Alpaca is a broker with trading");
        assert!(meta.requires_api_key_for_data);
        assert!(meta.requires_api_key_for_trading);
    }

    /// Test get() returns None for invalid ExchangeId
    #[test]
    fn test_registry_get_missing() {
        let registry = ConnectorRegistry::new();

        // Custom(999) should not exist
        let missing = registry.get(&ExchangeId::Custom(999));
        assert!(missing.is_none(), "Invalid ExchangeId should return None");
    }

    /// Test iter() returns all entries
    #[test]
    fn test_registry_iter() {
        let registry = ConnectorRegistry::new();

        let count = registry.iter().count();
        assert_eq!(count, 43, "iter() should return all 43 connectors");

        // Verify we can collect into vector
        let all: Vec<_> = registry.iter().collect();
        assert_eq!(all.len(), 43);
    }

    /// Test list_by_category for CryptoExchangeCex (should be 19)
    #[test]
    fn test_registry_list_by_category_cex() {
        let registry = ConnectorRegistry::new();

        let cex = registry.list_by_category(ConnectorCategory::CryptoExchangeCex);

        // Expected: 17 CEX + 2 derivatives (Deribit, HyperLiquid) = 19
        assert_eq!(cex.len(), 19, "Should have exactly 19 CEX connectors");

        // Verify all are CEX or Hybrid type
        for meta in &cex {
            assert!(
                meta.exchange_type == ExchangeType::Cex || meta.exchange_type == ExchangeType::Hybrid,
                "{} should be CEX or Hybrid type",
                meta.name
            );
        }
    }

    /// Test list_by_category for CryptoExchangeDex (should be 7)
    #[test]
    fn test_registry_list_by_category_dex() {
        let registry = ConnectorRegistry::new();

        let dex = registry.list_by_category(ConnectorCategory::CryptoExchangeDex);

        assert_eq!(dex.len(), 3, "Should have exactly 3 DEX connectors");

        // Verify all are DEX type
        for meta in &dex {
            assert_eq!(meta.exchange_type, ExchangeType::Dex, "{} should be DEX type", meta.name);
        }
    }

    /// Test list_by_category for StockMarketUS (should be 5)
    #[test]
    fn test_registry_list_by_category_stock_us() {
        let registry = ConnectorRegistry::new();

        let stocks = registry.list_by_category(ConnectorCategory::StockMarketUS);

        assert_eq!(stocks.len(), 5, "Should have exactly 5 US stock connectors");

        // Verify expected connectors
        let names: Vec<&str> = stocks.iter().map(|m| m.name).collect();
        assert!(names.contains(&"Polygon.io"));
        assert!(names.contains(&"Finnhub"));
        assert!(names.contains(&"Alpaca"));
    }

    /// Test list_by_type for ExchangeType::Cex
    #[test]
    fn test_registry_list_by_type_cex() {
        let registry = ConnectorRegistry::new();

        let cex_type = registry.list_by_type(ExchangeType::Cex);

        // Should include all CEX exchanges (without Hybrid)
        assert!(cex_type.len() >= 17, "Should have at least 17 CEX-type connectors");

        // Verify all have Cex type
        for meta in &cex_type {
            assert_eq!(meta.exchange_type, ExchangeType::Cex);
        }
    }

    /// Test list_with_trading returns connectors with trading feature
    #[test]
    fn test_registry_list_with_trading() {
        let registry = ConnectorRegistry::new();

        let trading = registry.list_with_trading();

        // CEX (18) + DEX (3) + Brokers (India 5, US 1, Russia 1, Forex 1, Aggregator 1) = ~29
        assert!(trading.len() >= 20, "Should have at least 20 connectors with trading");

        // Verify all have trading enabled
        for meta in &trading {
            assert!(
                meta.supported_features.trading,
                "{} should have trading feature enabled",
                meta.name
            );
        }
    }

    /// Test list_with_websocket returns connectors with WebSocket feature
    #[test]
    fn test_registry_list_with_websocket() {
        let registry = ConnectorRegistry::new();

        let websocket = registry.list_with_websocket();

        // Should include CEX with WS + some brokers and data providers
        assert!(websocket.len() >= 15, "Should have at least 15 connectors with WebSocket");

        // Verify all have websocket enabled
        for meta in &websocket {
            assert!(
                meta.supported_features.websocket,
                "{} should have websocket feature enabled",
                meta.name
            );
        }
    }

    /// Test that all metadata has non-empty name and base_url
    #[test]
    fn test_registry_metadata_fields() {
        let registry = ConnectorRegistry::new();

        for meta in registry.iter() {
            // All metadata should have non-empty name
            assert!(!meta.name.is_empty(), "Connector should have non-empty name");

            // All metadata should have non-empty base_url
            assert!(!meta.base_url.is_empty(), "Connector {} should have non-empty base_url", meta.name);

            // If websocket is supported, should have websocket_url
            if meta.supported_features.websocket {
                assert!(
                    meta.websocket_url.is_some(),
                    "Connector {} has websocket feature but no websocket_url",
                    meta.name
                );
            }
        }
    }

    /// Test that all ConnectorCategory variants are used
    #[test]
    fn test_registry_all_categories_covered() {
        let registry = ConnectorRegistry::new();

        // Get all unique categories
        use std::collections::HashSet;
        let categories: HashSet<_> = registry.iter().map(|m| m.category).collect();

        // Verify all categories are represented
        assert!(categories.contains(&ConnectorCategory::CryptoExchangeCex));
        assert!(categories.contains(&ConnectorCategory::CryptoExchangeDex));
        assert!(categories.contains(&ConnectorCategory::StockMarketUS));
        assert!(categories.contains(&ConnectorCategory::StockMarketIndia));
        assert!(categories.contains(&ConnectorCategory::StockMarketJapan));
        assert!(categories.contains(&ConnectorCategory::StockMarketKorea));
        assert!(categories.contains(&ConnectorCategory::StockMarketRussia));
        assert!(categories.contains(&ConnectorCategory::Forex));
        assert!(categories.contains(&ConnectorCategory::DataFeed));
        assert!(categories.contains(&ConnectorCategory::Broker));
        assert!(categories.contains(&ConnectorCategory::DataProvider));

        // Should have exactly 11 categories
        assert_eq!(categories.len(), 11, "Should use all 11 ConnectorCategory variants");
    }

    /// Test that there are no duplicate ExchangeIds
    #[test]
    fn test_registry_no_duplicates() {
        use std::collections::HashSet;

        let ids: HashSet<ExchangeId> = CONNECTOR_METADATA_ARRAY.iter().map(|m| m.id).collect();

        assert_eq!(
            ids.len(),
            CONNECTOR_METADATA_ARRAY.len(),
            "Duplicate ExchangeId found in registry"
        );
    }

    /// Test Features::full() preset
    #[test]
    fn test_features_full() {
        let features = Features::full();

        assert!(features.market_data);
        assert!(features.trading);
        assert!(features.account);
        assert!(features.positions);
        assert!(features.websocket);
    }

    /// Test Features::data_only() preset
    #[test]
    fn test_features_data_only() {
        let features = Features::data_only();

        assert!(features.market_data);
        assert!(!features.trading);
        assert!(!features.account);
        assert!(!features.positions);
        assert!(!features.websocket);
    }

    /// Test Features::dex() preset
    #[test]
    fn test_features_dex() {
        let features = Features::dex();

        assert!(features.market_data);
        assert!(features.trading);
        assert!(!features.account);
        assert!(!features.positions);
        assert!(!features.websocket);
    }

    /// Test RateLimits::none() preset
    #[test]
    fn test_rate_limits_none() {
        let limits = RateLimits::none();

        assert!(limits.requests_per_second.is_none());
        assert!(limits.requests_per_minute.is_none());
        assert!(limits.weight_per_minute.is_none());
    }

    /// Test RateLimits::standard() preset
    #[test]
    fn test_rate_limits_standard() {
        let limits = RateLimits::standard(10, 600);

        assert_eq!(limits.requests_per_second, Some(10));
        assert_eq!(limits.requests_per_minute, Some(600));
        assert!(limits.weight_per_minute.is_none());
    }

    /// Test that registry lookup is O(1) via HashMap
    #[test]
    fn test_registry_o1_lookup() {
        let registry = ConnectorRegistry::new();

        // Multiple lookups should be fast (O(1))
        for _ in 0..100 {
            let _ = registry.get(&ExchangeId::Binance);
            let _ = registry.get(&ExchangeId::OKX);
            let _ = registry.get(&ExchangeId::Lighter);
        }

        // Test should complete quickly if lookup is O(1)
    }

    /// Test that static metadata array lives in .rodata (zero heap allocations)
    #[test]
    fn test_static_metadata_no_heap() {
        // CONNECTOR_METADATA_ARRAY is a static slice
        // This test verifies the reference is valid
        assert_eq!(CONNECTOR_METADATA_ARRAY.len(), 47);

        // Static data should be accessible without allocation
        let first = &CONNECTOR_METADATA_ARRAY[0];
        assert!(!first.name.is_empty());
    }

    /// Test filtering by multiple criteria
    #[test]
    fn test_registry_complex_filtering() {
        let registry = ConnectorRegistry::new();

        // Find CEX connectors with trading and websocket
        let cex_ws_trading: Vec<_> = registry
            .list_by_category(ConnectorCategory::CryptoExchangeCex)
            .into_iter()
            .filter(|m| m.supported_features.websocket && m.supported_features.trading)
            .collect();

        // Most major CEX should have both
        assert!(cex_ws_trading.len() >= 10, "Should have at least 10 CEX with WS + trading");
    }

    /// Test that free tier information is accurate
    #[test]
    fn test_registry_free_tier() {
        let registry = ConnectorRegistry::new();

        let free_connectors: Vec<_> = registry.iter().filter(|m| m.free_tier).collect();

        // Most connectors should have free tier
        assert!(free_connectors.len() >= 35, "Should have at least 35 connectors with free tier");

        // Verify some known free connectors
        let binance = registry.get(&ExchangeId::Binance).unwrap();
        assert!(binance.free_tier, "Binance should have free tier");

        let lighter = registry.get(&ExchangeId::Lighter).unwrap();
        assert!(lighter.free_tier, "Lighter should have free tier");
    }

    /// Test AuthType variants are used correctly
    #[test]
    fn test_auth_types() {
        let registry = ConnectorRegistry::new();

        // Count connectors by auth type
        let api_key_count = registry.iter().filter(|m| m.authentication == AuthType::ApiKey).count();
        let none_count = registry.iter().filter(|m| m.authentication == AuthType::None).count();
        let oauth_count = registry.iter().filter(|m| m.authentication == AuthType::OAuth2).count();

        // Most should use API key
        assert!(api_key_count >= 30, "Should have at least 30 connectors with API key auth");

        // Some should require no auth (DEX, public data)
        assert!(none_count >= 5, "Should have at least 5 connectors with no auth");

        // Some brokers use OAuth
        assert!(oauth_count >= 1, "Should have at least 1 connector with OAuth2");
    }
}
