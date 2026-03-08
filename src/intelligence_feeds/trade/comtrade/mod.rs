//! UN COMTRADE (International Trade Statistics) connector
//!
//! Provides access to international trade data from the United Nations.
//!
//! # API Documentation
//! - Comtrade Plus API: https://comtradeplus.un.org/
//! - API Documentation: https://comtradeapi.un.org/
//!
//! # Authentication
//! Requires API key via `COMTRADE_API_KEY` environment variable.
//! Header: `Ocp-Apim-Subscription-Key: YOUR_KEY`
//!
//! # Rate Limits
//! - Free guest: 500 requests/day
//! - Registered: 250 requests/hour
//!
//! # Example Usage
//! ```ignore
//! use connectors_v5::data_feeds::comtrade::{ComtradeConnector, COUNTRY_US, COUNTRY_CHINA, FLOW_IMPORT};
//!
//! let connector = ComtradeConnector::from_env();
//!
//! // Get US imports from China in 2024
//! let imports = connector.get_bilateral_trade(COUNTRY_US, COUNTRY_CHINA, "2024").await?;
//!
//! // Get list of countries
//! let countries = connector.get_reporters().await?;
//! ```

pub mod endpoints;
pub mod auth;
pub mod parser;
pub mod connector;

pub use endpoints::*;
pub use auth::*;
pub use parser::*;
pub use connector::*;
