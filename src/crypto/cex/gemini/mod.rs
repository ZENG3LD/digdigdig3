//! # Gemini Exchange Connector
//!
//! Полная реализация коннектора для Gemini.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (HMAC-SHA384)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - GeminiConnector + impl трейтов
//! - `websocket` - WebSocket подключение
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::gemini::GeminiConnector;
//!
//! let connector = GeminiConnector::new(Some(credentials), false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended методы (Gemini-специфичные)
//! let symbols = connector.get_symbols().await?;
//! let volume = connector.get_notional_volume().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{GeminiEndpoint, GeminiUrls};
pub use auth::GeminiAuth;
pub use parser::GeminiParser;
pub use connector::GeminiConnector;
pub use websocket::{GeminiWebSocket, WebSocketType};
