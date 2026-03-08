//! # Paradex Exchange Connector
//!
//! Полная реализация коннектора для Paradex.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - JWT-based аутентификация
//! - `parser` - Парсинг JSON ответов (REST + WebSocket)
//! - `connector` - ParadexConnector + impl трейтов
//! - `websocket` - WebSocket подключение с broadcast channel
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::paradex::ParadexConnector;
//!
//! let credentials = Credentials::new("jwt_token", ""); // JWT token in api_key
//! let connector = ParadexConnector::new(credentials, false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::FuturesCross).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::FuturesCross).await?;
//!
//! // Extended методы (Paradex-специфичные)
//! let markets = connector.get_markets_summary(None).await?;
//! let symbols = connector.get_symbols().await?;
//! ```
//!
//! ## Note
//!
//! Paradex работает только с perpetual futures. Нет spot trading.
//! Все приватные endpoint'ы требуют JWT token.

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{ParadexEndpoint, ParadexUrls};
pub use auth::ParadexAuth;
pub use parser::ParadexParser;
pub use connector::ParadexConnector;
pub use websocket::ParadexWebSocket;
