//! # MOEX (Moscow Exchange) ISS API Connector
//!
//! Category: stocks/russia
//! Type: Data Provider (Market Data Only)
//!
//! ## Features
//! - REST API: Yes (ISS - Informational & Statistical Server)
//! - WebSocket: Yes (STOMP protocol over WebSocket)
//! - Authentication: Optional (Basic Auth for real-time data)
//! - Free tier: Yes (15-minute delayed data)
//!
//! ## Data Types
//! - Price data: Yes (real-time with subscription, delayed for free)
//! - Historical data: Yes (extensive history from 1997+)
//! - OHLC/Candles: Yes (1m, 10m, 1h, 1d, 1w, 1M, 1Q intervals)
//! - Orderbook: Yes (10x10 depth for paid subscribers)
//! - Recent trades: Yes
//! - Indices: Yes (IMOEX, RTSI, sector indices)
//! - Corporate data: Yes (CCI - Corporate Information Services)
//! - Fundamentals: Yes (IFRS and Russian accounting standards)
//! - Corporate actions: Yes (dividends, splits, meetings)
//! - Derivatives: Yes (futures and options on FORTS)
//!
//! ## Markets
//! - Equities: Russian stocks (MOEX, RTS)
//! - Bonds: Government (OFZ) and corporate bonds
//! - Derivatives: Futures and options (FORTS market)
//! - Currency: FX pairs (USD/RUB, EUR/RUB, etc.)
//! - Commodities: Limited
//!
//! ## Limitations
//! - No trading via ISS API (data only)
//! - 15-minute delay for free tier
//! - Real-time requires paid subscription
//! - Documentation mostly in Russian
//! - Rate limits not publicly documented
//!
//! ## Engines and Markets
//! MOEX operates 11 trading engines with 120+ markets:
//! - **stock**: Equities and deposits
//! - **currency**: Foreign exchange
//! - **futures**: Derivatives market (FORTS)
//! - **state**: Government securities
//! - **commodity**: Commodity market
//! - **money**: Money market
//! - **otc**: OTC markets
//! - And others...
//!
//! ## Usage Example
//! ```ignore
//! use connectors_v5::stocks::russia::moex::{MoexConnector, MoexAuth};
//!
//! // Public data (no auth, 15-minute delay)
//! let connector = MoexConnector::new_public();
//!
//! // With authentication (real-time data)
//! let auth = MoexAuth::new("username", "password");
//! let connector = MoexConnector::new(auth);
//!
//! // Get current price for SBER
//! let symbol = Symbol::new("SBER", "RUB");
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//!
//! // Get historical candles
//! let klines = connector.get_klines(symbol, "60", Some(100), AccountType::Spot).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

mod websocket;

pub use endpoints::{MoexEndpoint, MoexEndpoints};
pub use auth::MoexAuth;
pub use parser::MoexParser;
pub use connector::MoexConnector;

pub use websocket::MoexWebSocket;
