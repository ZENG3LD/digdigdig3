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
    ExchangeError, ExchangeResult,
    Price, Quantity, Asset, Timestamp,
    OrderSide, OrderType, OrderStatus, Order, Position, PositionSide, Balance,
    ExchangeCredentials,
    SymbolInfo,
    // Capabilities
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,

    // WebSocket types
    ConnectionStatus, StreamType, SubscriptionRequest, StreamEvent,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,

    // Utils
    hmac_sha256, hmac_sha512, sha256, sha512,
    encode_base64, encode_hex, encode_hex_lower,
    timestamp_millis, timestamp_seconds, timestamp_iso8601,

    // Precision utilities
    safe_price, safe_qty, format_price, format_qty,
    PrecisionCache, PrecisionInfo,

    // Transport
    HttpClient,
};
