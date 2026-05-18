//! # Polymarket Prediction Markets Connector
//!
//! Category: prediction/polymarket
//! Type: Prediction Market (CLOB-based probability trading)
//!
//! ## Overview
//!
//! Polymarket is a prediction market platform where users trade on the
//! probability of real-world events. Prices represent probabilities (0.0-1.0).
//!
//! ## APIs
//!
//! | API | URL | Purpose |
//! |-----|-----|---------|
//! | CLOB | `https://clob.polymarket.com` | Order books, prices, markets |
//! | Gamma | `https://gamma-api.polymarket.com` | Events, enhanced market metadata |
//! | Data | `https://data-api.polymarket.com` | User positions, trades |
//! | WS CLOB | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | Real-time data |
//!
//! ## Key Concepts
//!
//! - `condition_id` — 0x-prefixed hex string identifying a market on-chain
//! - `token_id` — numeric string identifying a YES or NO outcome token
//! - Prices are probabilities: 0.65 = 65% chance the event occurs
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::prediction::polymarket::PolymarketConnector;
//! use connectors_v5::core::traits::MarketData;
//! use connectors_v5::core::types::{AccountType, Symbol};
//!
//! // Create public connector
//! let connector = PolymarketConnector::public();
//!
//! // Ping to check connectivity
//! connector.ping().await?;
//!
//! // Get all active markets as SymbolInfo
//! let symbols = connector.get_exchange_info(AccountType::Spot).await?;
//!
//! // Get price for a specific market (condition_id as symbol.base)
//! let symbol = Symbol::new("0xABCDEF1234...", "USDC");
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! println!("YES probability: {:.1}%", price * 100.0);
//!
//! // Get klines (price history)
//! let klines = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot, None).await?;
//! ```
//!
//! ## WebSocket
//!
//! ```ignore
//! use connectors_v5::prediction::polymarket::{ClobWebSocket, WsEvent};
//!
//! let token_ids = vec!["TOKEN_ID".to_string()];
//! let mut ws = ClobWebSocket::new(token_ids, false);
//!
//! ws.connect().await?;
//!
//! while let Ok(Some(event)) = ws.recv().await {
//!     match event {
//!         WsEvent::LastTradePrice(trade) => println!("Price: {}", trade.price),
//!         _ => {}
//!     }
//! }
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

// Public API
pub use endpoints::{PolymarketEndpoint, PolymarketEndpoints};
pub use auth::{PolymarketAuth, PolymarketCredentials};
pub use parser::{
    PolymarketParser,
    // Domain types from Gamma API
    PolyMarket, PolyEvent, PolyTag,
    // Domain types from CLOB API
    ClobMarket, PolyToken, PolyOrderBook, PolyPriceLevel,
    PriceHistoryPoint, PolyMidpoint, PolyOrder, PolyTrade,
    // WebSocket types
    WsSubscription, WsBookSnapshot, WsLastTradePrice, WsPriceChange,
    WsTickSizeChange, WsBestBidAsk,
    // Conversion functions
    clob_market_to_symbol_info, poly_market_to_symbol_info,
    price_history_to_klines, poly_orderbook_to_v5,
    clob_market_to_ticker, poly_market_to_ticker,
    interval_to_ms,
};
pub use connector::PolymarketConnector;
pub use websocket::{ClobWebSocket, WsEvent, WsError, WsReconnectInfo, WsUnknownEvent, parse_event, normalize_price};
