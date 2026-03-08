//! # Forex Brokers & Data Providers Module
//!
//! Forex broker and data provider connectors:
//! - Brokers: OANDA (trading + data)
//! - Data Providers: AlphaVantage, Dukascopy (historical tick data)

pub mod oanda;
pub mod alphavantage;
pub mod dukascopy;

pub use oanda::{OandaConnector, OandaUrls, OandaEndpoint, OandaAuth};
pub use alphavantage::{AlphaVantageConnector, AlphaVantageEndpoints, AlphaVantageFunction, AlphaVantageAuth};
pub use dukascopy::{DukascopyConnector, DukascopyUrls, DukascopyEndpoint, DukascopyAuth};
