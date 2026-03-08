//! UK Parliament authentication
//!
//! Authentication type: None
//!
//! UK Parliament API is public and does not require authentication.

use std::collections::HashMap;

/// UK Parliament authentication credentials (none required)
#[derive(Clone)]
pub struct UkParliamentAuth;

impl UkParliamentAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for UkParliamentAuth {
    fn default() -> Self {
        Self::new()
    }
}
