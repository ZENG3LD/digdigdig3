//! # On-Chain Analytics
//!
//! Cross-chain blockchain analytics and monitoring providers.

pub mod bitquery;
pub mod whale_alert;

pub use bitquery::{BitqueryConnector, BitqueryAuth, BitqueryParser};
pub use whale_alert::{WhaleAlertConnector, WhaleAlertAuth, WhaleAlertParser};
