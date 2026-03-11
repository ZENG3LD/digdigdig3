//! # Upbit Exchange Connector
//!
//! Полная реализация коннектора для Upbit (Korean exchange).
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - JWT-based подпись запросов
//! - `parser` - Парсинг JSON ответов (REST и WebSocket)
//! - `connector` - UpbitConnector + impl трейтов
//! - `websocket` - WebSocket подключение
//!
//! ## Регионы
//!
//! Upbit operates in multiple regions:
//! - Singapore (sg) - default
//! - Indonesia (id)
//! - Thailand (th)
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::upbit::UpbitConnector;
//!
//! // Create connector for Singapore region
//! let connector = UpbitConnector::new(credentials, "sg").await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended методы (Upbit-специфичные)
//! let trading_pairs = connector.get_trading_pairs().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{UpbitEndpoint, UpbitUrls};
pub use auth::UpbitAuth;
pub use parser::UpbitParser;
pub use connector::UpbitConnector;
pub use websocket::UpbitWebSocket;
