//! # Raydium Authentication
//!
//! **IMPORTANT**: Raydium REST APIs require NO authentication.
//!
//! All Raydium API V3 endpoints are public and read-only. Unlike centralized
//! exchanges that use HMAC-SHA256 signatures with API keys, Raydium DEX:
//!
//! - Provides public market data without authentication
//! - Does NOT execute trades via API (trades happen on-chain)
//! - Uses wallet signatures for on-chain transactions (not REST API)
//!
//! ## Why No Auth?
//!
//! - **DEX Architecture**: State is on Solana blockchain, not centralized servers
//! - **Public Data**: Pool info, prices, token lists are publicly accessible
//! - **No Trading Execution**: API only provides data and serializes transactions
//! - **Wallet-Based**: Trading requires signing transactions with Solana wallet
//!
//! ## For On-Chain Trading
//!
//! If implementing actual swap execution, you would need:
//! - `solana-sdk` for wallet/keypair management
//! - Ed25519 signatures (not HMAC)
//! - Direct interaction with Solana RPC and Raydium programs
//!
//! This is out of scope for a REST API connector focused on market data.

use std::collections::HashMap;

/// Raydium "authentication" handler (no-op)
///
/// This struct exists for consistency with other exchange connectors,
/// but all methods are no-ops since Raydium requires no authentication.
#[derive(Debug, Clone)]
pub struct RaydiumAuth;

impl RaydiumAuth {
    /// Create new RaydiumAuth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Add auth headers to request (no-op for Raydium)
    ///
    /// Returns empty HashMap since no auth headers are required.
    pub fn add_auth_headers(
        &self,
        _method: &str,
        _endpoint: &str,
        _body: Option<&str>,
    ) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Check if authentication is required (always false)
    pub fn is_authenticated(&self) -> bool {
        false
    }
}

impl Default for RaydiumAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth_required() {
        let auth = RaydiumAuth::new();

        // Should return empty headers
        let headers = auth.add_auth_headers("GET", "/pools/info/list", None);
        assert!(headers.is_empty());

        // Should never be authenticated
        assert!(!auth.is_authenticated());
    }
}
