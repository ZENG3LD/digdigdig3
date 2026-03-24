//! # Fyers Securities Connector
//!
//! Complete implementation of Fyers API v3 connector for Indian markets.
//!
//! ## Features
//!
//! - **FREE API** - No subscription fees for basic access
//! - **F&O Specialization** - Strong support for Futures & Options trading
//! - **Multi-Exchange** - NSE, BSE, MCX, NCDEX support
//! - **High Rate Limits** - 100,000 requests/day, 10/sec, 200/min
//! - **WebSocket Support** - Data, Order, and TBT (Tick-by-Tick) streams
//! - **Fast Execution** - Orders execute under 50ms
//!
//! ## Market Coverage
//!
//! - **Equities**: NSE, BSE stocks
//! - **Futures & Options**: Index and stock derivatives
//! - **Commodities**: MCX, NCDEX
//! - **Currency Derivatives**: NSE currency futures/options
//!
//! ## Authentication
//!
//! Fyers uses OAuth 2.0 flow with SHA-256 hashing:
//!
//! 1. Generate authorization URL
//! 2. User logs in via browser (username, password, TOTP)
//! 3. Receive auth_code via redirect
//! 4. Exchange for access_token
//! 5. Use access_token in API requests
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::stocks::india::fyers::{FyersConnector, FyersAuth};
//!
//! // Create connector with access token
//! let auth = FyersAuth::with_token("APP_ID", "APP_SECRET", "ACCESS_TOKEN");
//! let connector = FyersConnector::new(auth)?;
//!
//! // Or from environment variables
//! let connector = FyersConnector::from_env()?;
//!
//! // Market Data
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let orderbook = connector.get_orderbook(symbol, None, AccountType::Spot).await?;
//! let klines = connector.get_klines(symbol, "5m", Some(100), AccountType::Spot).await?;
//!
//! // Trading
//! let order = connector.market_order(symbol, OrderSide::Buy, qty, AccountType::Spot).await?;
//! let order = connector.limit_order(symbol, OrderSide::Buy, qty, price, AccountType::Spot).await?;
//! let order = connector.cancel_order(symbol, order_id, AccountType::Spot).await?;
//! let orders = connector.get_open_orders(Some(symbol), AccountType::Spot).await?;
//!
//! // Account
//! let balances = connector.get_balance(None, AccountType::Spot).await?;
//! let account_info = connector.get_account_info(AccountType::Spot).await?;
//!
//! // Positions
//! let positions = connector.get_positions(None, AccountType::Spot).await?;
//!
//! // Extended Methods
//! let holdings = connector.get_holdings().await?;
//! let trades = connector.get_tradebook().await?;
//! connector.convert_position("NSE:SBIN-EQ", 1, 100.0, "INTRADAY", "CNC").await?;
//! ```
//!
//! ## Symbol Format
//!
//! Fyers uses format: `EXCHANGE:SYMBOL-SERIES`
//!
//! - Equity: `NSE:SBIN-EQ`
//! - Futures: `NSE:NIFTY24JANFUT`
//! - Options: `NSE:NIFTY2411921500CE`
//! - BSE: `BSE:SENSEX-EQ`
//! - MCX: `MCX:GOLDM24JANFUT`
//!
//! ## Rate Limits
//!
//! - 10 requests per second
//! - 200 requests per minute
//! - 100,000 requests per day
//!
//! ## Error Handling
//!
//! - `401/-1600`: Authentication failed (re-authenticate)
//! - `429`: Rate limit exceeded (wait and retry)
//! - `-100`: Invalid parameters
//! - `-351`: Symbol limit exceeded (WebSocket)
//!
//! ## Notes
//!
//! 1. Access tokens expire after trading day (~24 hours)
//! 2. No refresh token mechanism - must re-authenticate
//! 3. WebSocket supports up to 5,000 symbol subscriptions
//! 4. Historical data availability varies by symbol/timeframe
//! 5. Options historical data may be limited to daily bars
//! 6. F&O contracts don't have funding rates (use expiry dates)
//! 7. Leverage is product-specific (INTRADAY/MARGIN), not per-symbol

mod endpoints;
mod auth;
mod parser;
mod connector;

// TODO: Re-enable when research directory is created
// pub mod research;

pub use endpoints::{FyersEndpoint, FyersUrls, format_symbol, map_kline_interval};
pub use auth::FyersAuth;
pub use parser::FyersParser;
pub use connector::FyersConnector;

// Re-export commonly used items for convenience
pub use connector::FyersConnector as Connector;
pub use auth::FyersAuth as Auth;
