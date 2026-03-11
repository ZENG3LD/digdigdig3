//! Crypto/Market data providers

pub mod coinglass;
pub mod coingecko;

pub use coinglass::{CoinglassConnector, CoinglassAuth, CoinglassParser};
pub use coingecko::{CoinGeckoConnector, CoinGeckoAuth};

// Bitquery → moved to onchain::analytics::bitquery
// WhaleAlert → moved to onchain::analytics::whale_alert
// Etherscan → moved to onchain::ethereum::etherscan
// Compat re-exports so existing consumers don't break:
pub use crate::onchain::analytics::bitquery::{BitqueryConnector, BitqueryAuth, BitqueryParser};
pub use crate::onchain::analytics::whale_alert::{WhaleAlertConnector, WhaleAlertAuth, WhaleAlertParser};
pub use crate::onchain::ethereum::etherscan::{EtherscanConnector, EtherscanAuth};
