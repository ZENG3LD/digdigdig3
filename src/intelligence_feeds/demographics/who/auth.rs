//! WHO GHO authentication
//!
//! Authentication type: None
//!
//! WHO GHO API is completely free and does not require authentication.

use std::collections::HashMap;

/// WHO GHO authentication (no auth required)
#[derive(Clone, Default)]
pub struct WhoAuth;

impl WhoAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for WHO)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for WHO)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // WHO GHO API does not require authentication
    }

    /// Check if authentication is configured (always true for WHO)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
