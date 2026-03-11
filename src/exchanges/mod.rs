//! # Exchange Connectors
//!
//! Реализации коннекторов для конкретных бирж.
//!
//! ## Структура модуля биржи
//!
//! ```text
//! exchanges/
//! ├── mod.rs           <- этот файл
//! ├── kucoin/
//! │   ├── mod.rs       <- экспорты
//! │   ├── endpoints.rs <- URL'ы и endpoint enum
//! │   ├── auth.rs      <- подпись запросов
//! │   ├── parser.rs    <- парсинг JSON
//! │   ├── connector.rs <- KuCoinConnector + impl трейтов
//! │   └── websocket.rs <- WebSocket (опционально)
//! └── ...
//! ```
//!
//! ## Создание нового коннектора
//!
//! 1. Создать директорию: `exchanges/yourexchange/`
//! 2. Скопировать структуру из `kucoin/`
//! 3. Заменить endpoints, auth, parser по документации биржи
//! 4. Реализовать все методы трейтов
//! 5. Добавить `pub mod yourexchange;` сюда
//!
//! ## Пример использования
//!
//! ```ignore
//! use connectors_v5::exchanges::kucoin::KuCoinConnector;
//!
//! let connector = KuCoinConnector::new(Some(credentials), false).await?;
//!
//! // Core методы (из трейтов)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//!
//! // Extended методы (биржа-специфичные)
//! let tickers = connector.get_all_tickers(AccountType::Spot).await?;
//! ```

pub mod kucoin;
pub mod binance;
pub mod bingx;
pub mod bitfinex;
pub mod bitget;
pub mod bitstamp;
pub mod bybit;
pub mod coinbase;
pub mod crypto_com;
pub mod deribit;
pub mod gateio;
pub mod gemini;
pub mod htx;
pub mod kraken;
pub mod mexc;
pub mod okx;
pub mod phemex;
pub mod hyperliquid;
pub mod lighter;
pub mod jupiter;
// Uniswap → moved to onchain::ethereum::uniswap
// Raydium → moved to onchain::solana::raydium
pub mod gmx;

pub mod upbit;
pub mod dydx;
pub mod paradex;

// ═══════════════════════════════════════════════════════════════════════════════
// DISABLED EXCHANGES
// ═══════════════════════════════════════════════════════════════════════════════
// The following exchanges are disabled but code is kept for reference.
// See src/exchanges/DISABLED_EXCHANGES.md for details and re-enable instructions.

// DISABLED: Vertex Protocol shut down August 14, 2025 (acquired by Ink Foundation)
// See: research/vertex/ for shutdown announcement
// pub mod vertex;

// DISABLED: Bithumb has persistent infrastructure issues (SSL hangs, 403 geo-blocking, timeouts)
// See: src/exchanges/bithumb/research/504_investigation.md for detailed analysis
// pub mod bithumb;
