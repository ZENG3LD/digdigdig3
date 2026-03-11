//! # KuCoin Exchange Connector
//!
//! Полная реализация коннектора для KuCoin.
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - Подпись запросов (ExchangeAuth)
//! - `parser` - Парсинг JSON ответов
//! - `connector` - KuCoinConnector + impl трейтов
//! - `websocket` - WebSocket подключение
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::exchanges::kucoin::KuCoinConnector;
//!
//! let connector = KuCoinConnector::new(credentials, false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended методы (KuCoin-специфичные)
//! let tickers = connector.get_all_tickers(AccountType::Spot).await?;
//! let sub_accounts = connector.get_sub_accounts().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{KuCoinEndpoint, KuCoinUrls};
pub use auth::KuCoinAuth;
pub use parser::KuCoinParser;
pub use connector::KuCoinConnector;
pub use websocket::KuCoinWebSocket;
