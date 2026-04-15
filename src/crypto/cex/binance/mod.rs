//! # Binance Exchange Connector
//!
//! Полная реализация коннектора для Binance.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (HMAC-SHA256)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - BinanceConnector + impl трейтов
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::binance::BinanceConnector;
//!
//! let connector = BinanceConnector::new(credentials, false).await?;
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

pub use endpoints::{BinanceEndpoint, BinanceUrls};
pub use auth::BinanceAuth;
pub use parser::BinanceParser;
pub use connector::BinanceConnector;
pub use websocket::BinanceWebSocket;

#[cfg(test)]
mod _tests_websocket;
