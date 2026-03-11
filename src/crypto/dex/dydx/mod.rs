//! # dYdX v4 Exchange Connector
//!
//! Полная реализация коннектора для dYdX v4.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Аутентификация (placeholder для будущего gRPC)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - DydxConnector + impl трейтов
//! - `websocket` - WebSocket подключение
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::dydx::DydxConnector;
//!
//! let connector = DydxConnector::public(false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(Symbol::new("BTC", "USD"), AccountType::FuturesCross).await?;
//! let ticker = connector.get_ticker(Symbol::new("ETH", "USD"), AccountType::FuturesCross).await?;
//!
//! // Extended методы (dYdX-специфичные)
//! let markets = connector.get_all_markets().await?;
//! let market_info = connector.get_market_info("BTC-USD").await?;
//! ```
//!
//! ## Особенности dYdX v4
//!
//! - **Только perpetual futures** (нет spot markets)
//! - **Read-only через Indexer API** (текущая реализация)
//! - **Write операции через Node API (gRPC)** - будущая реализация
//! - **Wallet-based auth** (не API keys с HMAC)
//! - **Symbol format**: `BTC-USD` (uppercase, hyphen-separated)

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{DydxEndpoint, DydxUrls};
pub use auth::DydxAuth;
pub use parser::DydxParser;
pub use connector::DydxConnector;
pub use websocket::DydxWebSocket;
