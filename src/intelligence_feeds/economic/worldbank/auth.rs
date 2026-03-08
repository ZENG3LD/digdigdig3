//! World Bank authentication
//!
//! Authentication type: NONE
//!
//! The World Bank API is completely free and open - no API key required.
//! This makes it one of the easiest economic data APIs to use.

use std::collections::HashMap;

/// World Bank authentication (none required)
#[derive(Clone, Default)]
pub struct WorldBankAuth;

impl WorldBankAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for World Bank)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for World Bank)
    ///
    /// World Bank API is completely free and open - no authentication needed.
    /// This method exists for API compatibility with other connectors.
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured (always true for World Bank)
    pub fn is_authenticated(&self) -> bool {
        true // No auth needed
    }
}
