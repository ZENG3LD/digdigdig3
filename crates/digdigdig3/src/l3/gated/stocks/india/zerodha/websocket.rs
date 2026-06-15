//! # Zerodha WebSocket connector (stub)
//!
//! Zerodha Kite Connect WebSocket uses a binary protocol (KiteTicker).
//! This stub satisfies the module declaration; full implementation is deferred.

use crate::core::types::{AccountType, ExchangeError, ExchangeResult};

/// Zerodha WebSocket connector.
///
/// Zerodha KiteTicker uses a binary framing protocol (not JSON).
/// Full implementation is deferred. Constructing this type returns an
/// `NotImplemented` error via `new()`.
pub struct ZerodhaWebSocket;

impl ZerodhaWebSocket {
    /// Create a new ZerodhaWebSocket.
    ///
    /// Returns `NotImplemented` — full KiteTicker implementation is pending.
    pub fn new(_account_type: AccountType) -> ExchangeResult<Self> {
        Err(ExchangeError::NotImplemented(
            "Zerodha WebSocket (KiteTicker binary protocol) is not yet implemented".to_string(),
        ))
    }
}
