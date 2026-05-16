//! SupportLevel — describes what level of support a connector has for a given stream.

use serde::{Deserialize, Serialize};

/// What level of support a connector has for a given (StreamKind, AccountType) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportLevel {
    /// Parser registered in TopicRegistry; events flow.
    Native,
    /// dig3 has not yet implemented this exchange's channel for this stream kind.
    /// The channel likely exists on the exchange; it just hasn't been wired.
    NotImplemented,
    /// Exchange itself has no such channel for this account type.
    UnsupportedByExchange,
    /// Channel exists but requires authentication credentials (e.g. Binance forceOrders).
    RequiresAuth,
}
