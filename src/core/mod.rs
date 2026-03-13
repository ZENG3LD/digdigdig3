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
//! │  │ Trading          │     │   hmac_sha256    │     │ WebSocket           │ │
//! │  │ Account          │     │   hmac_sha512    │     │                     │ │
//! │  │ Positions        │     │                  │     │                     │ │
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

// Re-exports types
pub use types::{
    // Common
    ExchangeId, ExchangeType, AccountType, Symbol,
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
    // WebSocket
    ConnectionStatus, StreamType, SubscriptionRequest, StreamEvent,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    BalanceChangeReason, PositionChangeReason,
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
};

// Re-exports utils
pub use utils::{
    hmac_sha256, hmac_sha256_hex, hmac_sha384, hmac_sha512, sha256, sha512,
    encode_base64, encode_hex, encode_hex_lower,
    timestamp_millis, timestamp_seconds, timestamp_iso8601,
    SimpleRateLimiter, WeightRateLimiter,
};

// Re-exports transport
pub use http::HttpClient;
