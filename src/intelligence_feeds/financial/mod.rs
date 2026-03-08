//! Financial data providers (multi-asset, news, instruments)

pub mod alpha_vantage;
pub mod finnhub;
pub mod newsapi;
pub mod openfigi;

pub use alpha_vantage::{AlphaVantageConnector, AlphaVantageAuth};
pub use finnhub::{FinnhubConnector, FinnhubAuth};
pub use newsapi::{NewsApiConnector, NewsApiAuth};
pub use openfigi::{OpenFigiConnector, OpenFigiAuth};
