//! BIS authentication
//!
//! Authentication type: None (public API)
//!
//! BIS SDMX API is completely public and does not require authentication.

use std::collections::HashMap;

/// BIS authentication credentials (not needed - public API)
#[derive(Clone, Default)]
pub struct BisAuth;

impl BisAuth {
    /// Create new auth (no-op for BIS)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for BIS)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for BIS)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // BIS API is public - no authentication needed
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
