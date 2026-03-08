//! OECD authentication
//!
//! Authentication type: None
//!
//! OECD SDMX REST API does not require authentication.
//! All data is publicly accessible without API keys.

use std::collections::HashMap;

/// OECD authentication (no authentication needed)
#[derive(Clone, Default)]
pub struct OecdAuth;

impl OecdAuth {
    /// Create new auth (no-op for OECD)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for OECD)
    pub fn from_env() -> Self {
        Self
    }

    /// Sign query parameters (no-op for OECD)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // OECD requires no authentication
    }

    /// Check if authentication is configured (always true for OECD)
    pub fn is_authenticated(&self) -> bool {
        true // No authentication needed
    }
}
