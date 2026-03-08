//! Central Bank of Russia (CBR) API connector
//!
//! Provides access to Russian economic data including:
//! - Key interest rate
//! - Currency exchange rates against RUB
//! - Precious metal prices (gold, silver, platinum, palladium)
//! - International reserves
//! - Monetary base
//! - Interbank lending rates
//!
//! # Authentication
//! CBR API is public and does not require authentication.
//!
//! # Example
//! ```ignore
//! use connectors_v5::data_feeds::cbr::CbrConnector;
//!
//! let cbr = CbrConnector::new();
//!
//! // Get current key rate
//! let key_rates = cbr.get_key_rate().await?;
//! println!("Current CBR key rate: {}", key_rates.first().unwrap().rate);
//!
//! // Get today's exchange rates
//! let rates = cbr.get_daily_rates(None).await?;
//! for rate in rates.rates {
//!     println!("{}: {}", rate.char_code, rate.value);
//! }
//!
//! // Get historical USD exchange rate
//! let usd_history = cbr.get_exchange_rate_dynamic(
//!     "R01235",  // USD currency ID
//!     "2024-01-01",
//!     "2024-12-31"
//! ).await?;
//! ```

pub mod endpoints;
pub mod auth;
pub mod parser;
pub mod connector;

// Re-exports
pub use endpoints::{CbrEndpoints, CbrEndpoint, format_date_cbr, parse_date_cbr};
pub use auth::CbrAuth;
pub use parser::{
    CbrParser, KeyRate, CurrencyRate, Currency, DailyRates,
    RatePoint, MetalPrice, RepoRate, ReserveData,
};
pub use connector::CbrConnector;
