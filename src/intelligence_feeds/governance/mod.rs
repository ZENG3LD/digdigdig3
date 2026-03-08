//! Parliamentary and legislative data

pub mod uk_parliament;
pub mod eu_parliament;

pub use uk_parliament::{UkParliamentConnector, UkParliamentAuth};
pub use eu_parliament::{EuParliamentConnector, EuParliamentAuth, EuParliamentParser};
