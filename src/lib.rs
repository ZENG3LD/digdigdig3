//! # V5 Exchange Connectors - Traits + Utils Architecture
//!
//! ## Архитектура
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        v5/core                                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │  traits/      - Core трейты (MarketData, Trading, Account, ExchangeAuth)    │
//! │  utils/       - Утилиты (crypto, encoding, time)                            │
//! │  http/        - HTTP клиент                                                 │
//! │  websocket/   - WebSocket                                                   │
//! │  types/       - Общие типы                                                  │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                  v5/exchanges                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │  kucoin/      - KuCoinConnector (impl MarketData, Trading, ExchangeAuth)    │
//! │  binance/     - BinanceConnector                                            │
//! │  ...                                                                        │
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
//! - `utils::crypto` - hmac_sha256, hmac_sha512
//! - `utils::encoding` - encode_base64, encode_hex
//! - `utils::time` - timestamp_millis, timestamp_iso8601

pub mod core;
pub mod l1;
pub mod l2;
pub mod l3;
pub mod connector_manager;
pub mod testing;

pub use core::storage::{EventLog, EventLogIter, EventRecord, StorageManager, StorageConfig, StreamKey};
pub use core::replay::{ReplayHub, ReplayConfig, ReplayRate};
pub use core::orderbook::{OrderBookTracker, OrderBookError};
pub use core::rest_cache::RestCache;
pub use core::cure::{
    IntegrityChecker, IntegrityReport,
    Deduper,
    GapDetector, GapInfo,
    RepairPipeline, RepairReport,
};

// Re-exports для удобства
pub use core::{
    // Traits
    ExchangeIdentity, MarketData, Trading, Positions, Account,
    CoreConnector,
    WebSocketConnector, WebSocketExt,
    Authenticated, CredentialKind,
    Credentials, AuthRequest, SignatureLocation, ExchangeAuth,
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,

    // Types
    ExchangeId, ExchangeType, AccountType, Symbol,
    SymbolInput, OwnedSymbolInput,
    ExchangeError, ExchangeResult,
    Price, Quantity, Asset, Timestamp,
    OrderSide, OrderType, OrderStatus, Order, Position, PositionSide, Balance,
    ExchangeCredentials,
    SymbolInfo,
    // Capabilities
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
    RateLimitCapabilities, LimitModel,
    EndpointWeight, RestLimitPool, DecayingLimitConfig, WsLimits,
    // Empirical validation
    ValidationStamp, FieldValidation,

    // WebSocket types
    ConnectionStatus, StreamType, SubscriptionRequest, StreamEvent,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,

    // Extended market data types (source of truth for derivatives/options feeds)
    AggTrade,
    HistoricalVolatility, VolatilityIndex, Basis, IndexPrice, CompositeIndex,
    InsuranceFund, SettlementEvent, BlockTrade,
    OrderBookSide, L3Action, OrderbookL3Event,
    RiskLimit, PredictedFunding, FundingSettlement,
    AuctionEvent, MarketWarning, OptionGreeks,

    // Utils
    hmac_sha256, hmac_sha512, sha256, sha512,
    encode_base64, encode_hex, encode_hex_lower,
    timestamp_millis, timestamp_seconds, timestamp_iso8601,

    // Precision utilities
    safe_price, safe_qty, format_price, format_qty,
    PrecisionCache, PrecisionInfo,

    // Transport
    HttpClient,

    // Normalization (Phase λ.A)
    Canonicalize, CanonicalEvent,
    CanonicalTrade, CanonicalTicker,
    CanonicalOrderbook, CanonicalOrderbookDelta, CanonicalKline,
    CanonicalLevel,
    normalize_ts_to_ms,
};
