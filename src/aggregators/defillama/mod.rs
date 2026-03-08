//! # DefiLlama Aggregator Connector
//!
//! Complete implementation of DefiLlama DeFi data aggregator.
//!
//! ## Overview
//!
//! DefiLlama is a DeFi TVL (Total Value Locked) aggregator providing:
//! - Protocol TVL data across multiple chains
//! - Token prices (hourly updates)
//! - Stablecoin analytics
//! - Yield farming data
//! - DeFi protocol metadata
//!
//! ## Key Differences from Exchange Connectors
//!
//! 1. **NO WebSocket** - REST API only with hourly updates
//! 2. **NO Trading** - Data aggregator, not an exchange
//! 3. **URL-based auth** - API key in URL path (not headers)
//! 4. **Protocol-centric** - Uses protocol slugs instead of trading pairs
//! 5. **Polling strategy** - Hourly data refresh (not tick-by-tick)
//!
//! ## API Tiers
//!
//! - **Free tier**: 29 endpoints (TVL, prices, stablecoins)
//! - **Pro tier ($300/mo)**: 64 endpoints (advanced analytics)
//!
//! ## Structure
//!
//! - `endpoints` - URL building and endpoint enum
//! - `auth` - URL-based authentication (no HMAC)
//! - `parser` - JSON parsing for DeFi data types
//! - `connector` - DefiLlamaConnector + trait implementations
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::aggregators::defillama::DefiLlamaConnector;
//! use connectors_v5::Credentials;
//!
//! // Free tier
//! let connector = DefiLlamaConnector::new(None, false).await?;
//!
//! // Pro tier
//! let credentials = Credentials::new("your_api_key", "");
//! let connector = DefiLlamaConnector::new(Some(credentials), false).await?;
//!
//! // Get protocol data
//! let aave = connector.get_protocol("aave").await?;
//! println!("Aave TVL: ${}", aave.tvl);
//!
//! // Get token prices
//! let coins = vec![
//!     ("ethereum".to_string(), "0x6b175474e89094c44da98b954eedeac495271d0f".to_string()), // DAI
//! ];
//! let prices = connector.get_token_prices(coins).await?;
//!
//! // Get all protocols
//! let protocols = connector.get_protocols().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{DefiLlamaEndpoint, DefiLlamaUrls, EndpointCategory, format_protocol_slug, format_chain_name, format_coin_id};
pub use auth::DefiLlamaAuth;
pub use parser::{
    DefiLlamaParser, ProtocolData, TvlDataPoint, PriceData, PriceResponse,
    ChainData, StablecoinData, YieldPoolData, CoinPrice,
};
pub use connector::DefiLlamaConnector;
