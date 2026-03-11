//! # Phemex Exchange Connector
//!
//! Полная реализация коннектора для Phemex.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (HMAC SHA256)
//! - `parser` - Парсинг JSON ответов (REST + WebSocket)
//! - `connector` - PhemexConnector + impl трейтов
//! - `websocket` - WebSocket подключение
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::phemex::PhemexConnector;
//!
//! let connector = PhemexConnector::new(Some(credentials), false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended методы (Phemex-специфичные)
//! // TODO: Add extended methods as needed
//! ```
//!
//! ## Value Scaling
//!
//! Phemex uses integer representation with scale factors:
//! - Ep (Price): priceScale (typically 4 or 8)
//! - Er (Ratio): ratioScale (typically 8)
//! - Ev (Value): valueScale (typically 4 or 8)
//!
//! **CRITICAL**: Always fetch `/public/products` on startup to get scale factors!

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{PhemexEndpoint, PhemexUrls};
pub use auth::PhemexAuth;
pub use parser::PhemexParser;
pub use connector::PhemexConnector;
pub use websocket::PhemexWebSocket;
