//! # BingX Exchange Connector
//!
//! Полная реализация коннектора для BingX.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (HMAC-SHA256)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - BingxConnector + impl трейтов
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::bingx::BingxConnector;
//!
//! let connector = BingxConnector::new(credentials, false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{BingxEndpoint, BingxUrls};
pub use auth::BingxAuth;
pub use parser::BingxParser;
pub use connector::BingxConnector;
pub use websocket::BingxWebSocket;

#[cfg(test)]
mod _tests_websocket;
