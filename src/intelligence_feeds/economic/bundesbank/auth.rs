//! Bundesbank authentication
//!
//! Authentication type: NONE (public API)
//!
//! The Deutsche Bundesbank SDMX REST API is publicly accessible
//! and does not require authentication or API keys.

use std::collections::HashMap;

/// Bundesbank authentication (no credentials required)
#[derive(Clone, Default)]
pub struct BundesbankAuth;

impl BundesbankAuth {
    /// Create new auth instance
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for Bundesbank)
    pub fn from_env() -> Self {
        Self
    }

    /// Sign query parameters (no-op for Bundesbank)
    ///
    /// Bundesbank API does not require authentication.
    /// This method exists for API compatibility with other connectors.
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
