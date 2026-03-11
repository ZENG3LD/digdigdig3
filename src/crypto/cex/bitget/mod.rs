//! # Bitget Exchange Connector
//!
//! Полная реализация коннектора для Bitget.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (ExchangeAuth)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - BitgetConnector + impl трейтов
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::bitget::BitgetConnector;
//!
//! let connector = BitgetConnector::new(credentials, false).await?;
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

pub use endpoints::{BitgetEndpoint, BitgetUrls};
pub use auth::BitgetAuth;
pub use parser::BitgetParser;
pub use connector::BitgetConnector;
pub use websocket::BitgetWebSocket;
