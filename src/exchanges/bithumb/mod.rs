//! # Bithumb Exchange Connector
//!
//! Полная реализация коннектора для Bithumb Pro (Global Platform).
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (Parameter signing)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - BithumbConnector + impl трейтов
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::bithumb::BithumbConnector;
//!
//! let connector = BithumbConnector::new(credentials, false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended методы (Bithumb-специфичные)
//! let config = connector.get_config().await?;
//! let server_time = connector.get_server_time().await?;
//! ```
//!
//! ## Особенности Bithumb Pro
//!
//! - Hyphen-separated symbols: `BTC-USDT`, `ETH-USDT`
//! - Parameter signing authentication (HMAC-SHA256)
//! - Response format: `{"code": "0", "data": {...}}`
//! - Primary quote currency: USDT
//! - Primarily spot trading (limited futures support)

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{BithumbEndpoint, BithumbUrls};
pub use auth::BithumbAuth;
pub use parser::BithumbParser;
pub use connector::BithumbConnector;
pub use websocket::BithumbWebSocket;
