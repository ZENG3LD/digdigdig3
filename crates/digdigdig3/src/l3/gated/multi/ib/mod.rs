//! # Interactive Brokers Client Portal Web API Connector
//!
//! Category: Aggregator (Multi-Asset Broker)
//! Type: Full-Service Broker with Trading and Market Data
//!
//! ## Features
//! - **REST API**: Yes (Client Portal Web API v1.0)
//! - **WebSocket**: Yes (real-time streaming)
//! - **Authentication**: Gateway Session (manual browser login) or OAuth 2.0
//! - **Free tier**: No (requires active IBKR Pro account)
//!
//! ## Asset Classes Supported
//! - Stocks (Equities) - Global stocks on 150+ markets
//! - Forex (FX/CASH) - Spot FX trading
//! - Futures - Commodity, index, currency futures
//! - Options - Equity options, index options, futures options
//! - Bonds - Government and corporate bonds
//! - Mutual Funds & ETFs
//! - Cryptocurrencies (limited)
//! - CFDs (non-US accounts)
//!
//! ## Data Types
//! - Price data: Yes (real-time and delayed)
//! - Historical data: Yes (from 1 second to monthly bars)
//! - Derivatives data: Yes (futures, options)
//! - Fundamentals: Limited
//! - Market depth: Yes (Level 2)
//!
//! ## Trading Capabilities
//! - Order placement: Yes (Market, Limit, Stop, Stop-Limit, Trailing Stop)
//! - Order modification: Yes
//! - Order cancellation: Yes
//! - Account management: Yes (positions, balances, P&L)
//! - Portfolio analytics: Yes
//!
//! ## Authentication
//! - **Individual Accounts**: Client Portal Gateway (Java-based local proxy)
//!   - Requires manual browser login (cannot be automated)
//!   - Default URL: `https://localhost:5000/v1/api/`
//!   - Two-factor authentication required
//!   - Session timeout: ~6 minutes (requires periodic tickle)
//!
//! - **Enterprise Accounts**: OAuth 2.0 (Private Key JWT)
//!   - Production URL: `https://api.ibkr.com/v1/api/`
//!   - Fully automated authentication
//!   - RSA key pair required
//!
//! ## Key Differences from Crypto Exchanges
//! - Uses Contract ID (conid) instead of symbols for all operations
//! - Requires contract search before trading
//! - Order confirmation flow (some orders require explicit confirmation)
//! - Session management with periodic tickle (keep-alive) required
//! - WebSocket uses text-based subscription format (not JSON)
//!
//! ## Rate Limits
//! - **Gateway (Individual)**: 10 requests per second
//! - **OAuth (Enterprise)**: 50 requests per second
//! - **Specific Endpoints**:
//!   - `/tickle`: 1 req/s
//!   - `/iserver/account/{accountId}/orders`: 1 req/5s
//!   - `/iserver/marketdata/snapshot`: 10 req/s
//!   - `/iserver/marketdata/history`: 5 concurrent max
//!
//! ## Limitations
//! - **Manual Authentication**: Individual accounts require browser login
//! - **Single Session**: Only one session per username
//! - **Market Data**: Requires active subscriptions (typically 100 concurrent)
//! - **Geographic**: Canadian residents cannot trade Canadian exchanges programmatically
//!
//! ## Usage Example
//! ```ignore
//! use connectors_v5::aggregators::ib::IBConnector;
//! use connectors_v5::core::types::{Symbol, AccountType};
//!
//! // Create connector (assumes Gateway is running and authenticated)
//! let connector = IBConnector::from_gateway("https://localhost:5000/v1/api", "DU12345").await?;
//!
//! // Get market data (automatically resolves symbol to conid)
//! let symbol = Symbol::new("AAPL", "USD");
//! let ticker = connector.get_ticker(symbol, AccountType::Spot).await?;
//!
//! // Place order
//! let order_result = connector.place_limit_order(
//!     symbol,
//!     OrderSide::Buy,
//!     100.0,
//!     185.50,
//!     AccountType::Spot,
//! ).await?;
//! ```
//!
//! ## References
//! - [Client Portal Web API Documentation](https://interactivebrokers.github.io/cpwebapi/)
//! - [IBKR Campus API Page](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/)
//! - [WebSocket Streaming](https://www.interactivebrokers.com/campus/trading-lessons/websockets/)

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(feature = "websocket")]
mod websocket;

pub use connector::IBConnector;

#[cfg(feature = "websocket")]
pub use websocket::IBWebSocket;
