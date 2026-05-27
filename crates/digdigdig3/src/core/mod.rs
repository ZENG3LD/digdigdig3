//! # V5 Core - минимальная архитектура коннекторов
//!
//! ## Архитектура (Traits + Utils)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                              CORE MODULE                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐     ┌──────────────────┐     ┌─────────────────────┐ │
//! │  │     TRAITS       │     │      UTILS       │     │     TRANSPORT       │ │
//! │  │                  │     │                  │     │                     │ │
//! │  │ MarketData       │     │ crypto:          │     │ HttpClient          │ │
//! │  │ Trading          │     │   hmac_sha256    │     │ GraphQlClient       │ │
//! │  │ Account          │     │   hmac_sha512    │     │ WebSocket           │ │
//! │  │ Positions        │     │                  │     │ GrpcClient (grpc)   │ │
//! │  │ ExchangeAuth     │     │ encoding:        │     │                     │ │
//! │  │                  │     │   encode_base64  │     │                     │ │
//! │  │ ────────────────│     │   encode_hex     │     │                     │ │
//! │  │ CoreConnector   │     │                  │     │                     │ │
//! │  │ (combined)      │     │ time:            │     │                     │ │
//! │  │                  │     │   timestamp_*    │     │                     │ │
//! │  └──────────────────┘     └──────────────────┘     └─────────────────────┘ │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//!                                    │
//!                                    ▼
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    EXCHANGE CONNECTORS                                       │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────────────────────────────────────────────────────────┐  │
//! │  │                        KuCoinConnector                                │  │
//! │  │                                                                       │  │
//! │  │  impl MarketData for KuCoinConnector { ... }                         │  │
//! │  │  impl Trading for KuCoinConnector { ... }                            │  │
//! │  │  impl ExchangeAuth for KuCoinAuth { ... }                            │  │
//! │  │                                                                       │  │
//! │  │  + extended methods as struct methods                                │  │
//! │  │  + KuCoin-specific logic                                             │  │
//! │  │                                                                       │  │
//! │  └──────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core трейты
//!
//! | Трейт | Описание |
//! |-------|----------|
//! | `MarketData` | price, orderbook, klines, ticker, ping |
//! | `Trading` | market_order, limit_order, cancel, get_order, open_orders |
//! | `Account` | balance, account_info |
//! | `Positions` | positions, funding_rate, set_leverage |
//! | `ExchangeAuth` | sign_request (каждая биржа реализует свою логику) |
//!
//! ## Утилиты
//!
//! - `utils::crypto` - hmac_sha256, hmac_sha512, sha256, sha512
//! - `utils::encoding` - encode_base64, encode_hex, encode_hex_lower
//! - `utils::time` - timestamp_millis, timestamp_seconds, timestamp_iso8601
//! - `utils::rate_limiter` - SimpleRateLimiter, WeightRateLimiter

pub mod types;
pub mod traits;
pub mod utils;
pub mod http;
pub mod websocket;
pub mod chain;
pub mod macros;
pub mod normalization;

#[cfg(feature = "grpc")]
pub mod grpc;

/// Install the process-level rustls `CryptoProvider` (ring).
///
/// rustls 0.23 panics at TLS init unless exactly one provider is registered.
/// `HttpClient::new` calls this implicitly; callers that only need WebSocket
/// (e.g. `digdigdig3-station`) should call this before opening any TLS
/// connection. Idempotent: returns `Err(())` if a provider is already set.
///
/// On wasm32, this is a no-op (no native TLS stack needed).
pub fn install_default_crypto_provider() -> std::result::Result<(), ()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| ())
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(())
    }
}

// Re-exports types
pub use types::{
    // Common
    ExchangeId, ExchangeType, AccountType, Symbol,
    SymbolInput, OwnedSymbolInput,
    ExchangeError, ExchangeResult,
    // Market data
    Kline, Ticker, OrderBook, PublicTrade, FundingRate,
    // Trading
    Price, Quantity, Asset, Timestamp,
    OrderSide, OrderType, TriggerDirection, OrderStatus, TimeInForce, Order,
    OrderRequest, CancelRequest, CancelScope,
    AmendRequest, AmendFields, OrderHistoryFilter, OrdersQuery,
    PositionMode, PositionSide, Position, PositionModification, PositionQuery,
    UserTrade, UserTradeFilter,
    Balance, AccountInfo, BalanceQuery, MarginType, SymbolInfo,
    ExchangeCredentials,
    // Responses
    PlaceOrderResponse, OrderResult, CancelAllResponse,
    FeeInfo, TransferResponse, DepositAddress, WithdrawResponse, FundsRecord,
    ClosedPnlRecord, LongShortRatio,
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerEntryType, LedgerFilter,
    // Extended market data (derivatives/options feeds — source of truth)
    AggTrade,
    HistoricalVolatility, VolatilityIndex, Basis, IndexPrice, CompositeIndex,
    InsuranceFund, SettlementEvent, BlockTrade,
    OrderBookSide, L3Action, OrderbookL3Event,
    RiskLimit, PredictedFunding, FundingSettlement,
    AuctionEvent, MarketWarning, OptionGreeks,
    // WebSocket
    ConnectionStatus, StreamType, SubscriptionRequest, StreamEvent,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    BalanceChangeReason, PositionChangeReason,
    OrderbookCapabilities,
    // Capabilities
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
    // Rate limit capabilities
    RateLimitCapabilities, LimitModel,
    EndpointWeight, RestLimitPool, DecayingLimitConfig, WsLimits,
    // Empirical validation
    ValidationStamp, FieldValidation,
};

// Re-exports traits
pub use traits::{
    ExchangeIdentity, MarketData, Trading, Positions, Account,
    CoreConnector,
    WebSocketConnector, WebSocketExt,
    Authenticated, CredentialKind,
    // Backward compat for connector constructors/auth
    Credentials, AuthRequest, SignatureLocation, ExchangeAuth,
    // Optional operation traits
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};

// Re-exports utils
pub use utils::{
    hmac_sha256, hmac_sha256_hex, hmac_sha384, hmac_sha512, sha256, sha512,
    encode_base64, encode_hex, encode_hex_lower,
    timestamp_millis, timestamp_seconds, timestamp_iso8601,
    SimpleRateLimiter, WeightRateLimiter,
    RuntimeLimiter, RateLimitPressure, RateLimitMonitor,
    safe_price, safe_qty, format_price, format_qty,
    PrecisionCache, PrecisionInfo,
};

// Re-exports transport
pub use http::HttpClient;
pub use http::GraphQlClient;

// Re-exports chain types
pub use chain::{ChainFamily, ChainProvider, TxStatus};

// Re-exports normalization
pub use normalization::{
    Canonicalize, CanonicalEvent,
    CanonicalTrade, CanonicalTicker,
    CanonicalOrderbook, CanonicalOrderbookDelta, CanonicalKline,
    CanonicalLevel,
    normalize_ts_to_ms,
};
