//! Futu OpenAPI authentication
//!
//! Authentication type: OpenD login + trade unlock
//!
//! Futu uses two-tier authentication:
//! 1. OpenD ↔ Futu Servers: OpenD logs in with account credentials
//! 2. Client ↔ OpenD: Client connects to OpenD (local = no auth, remote = RSA key)
//!
//! This is NOT compatible with standard API key authentication.

use std::collections::HashMap;

/// Futu authentication credentials
///
/// Note: These are NOT API keys. They are OpenD connection parameters.
#[derive(Clone)]
pub struct FutuAuth {
    /// OpenD host (127.0.0.1 for local, or remote server IP)
    pub host: String,
    /// OpenD port (default: 11111)
    pub port: u16,
    /// Trade password (for unlocking trading operations)
    pub trade_password: Option<String>,
    /// RSA public key (for remote OpenD connections)
    pub rsa_key: Option<String>,
}

impl FutuAuth {
    /// Create auth from environment variables
    ///
    /// Expected environment variables:
    /// - FUTU_OPEND_HOST (default: 127.0.0.1)
    /// - FUTU_OPEND_PORT (default: 11111)
    /// - FUTU_TRADE_PASSWORD (optional, for trading)
    /// - FUTU_RSA_KEY (optional, for remote OpenD)
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("FUTU_OPEND_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("FUTU_OPEND_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(11111),
            trade_password: std::env::var("FUTU_TRADE_PASSWORD").ok(),
            rsa_key: std::env::var("FUTU_RSA_KEY").ok(),
        }
    }

    /// Create auth with explicit parameters
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            trade_password: None,
            rsa_key: None,
        }
    }

    /// Set trade password
    pub fn with_trade_password(mut self, password: impl Into<String>) -> Self {
        self.trade_password = Some(password.into());
        self
    }

    /// This method is a stub - Futu does NOT use HTTP headers
    pub fn sign_headers(&self, _headers: &mut HashMap<String, String>) {
        // Not applicable - Futu uses Protocol Buffers, not HTTP
    }
}
