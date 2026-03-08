//! Crypto/Market data providers

pub mod coinglass;
pub mod bitquery;
pub mod whale_alert;
pub mod coingecko;
pub mod etherscan;

pub use coinglass::{CoinglassConnector, CoinglassAuth, CoinglassParser};
pub use bitquery::{BitqueryConnector, BitqueryAuth, BitqueryParser};
pub use whale_alert::{WhaleAlertConnector, WhaleAlertAuth, WhaleAlertParser};
pub use coingecko::{CoinGeckoConnector, CoinGeckoAuth};
pub use etherscan::{EtherscanConnector, EtherscanAuth};
