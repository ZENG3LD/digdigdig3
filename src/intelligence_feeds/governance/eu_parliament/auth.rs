//! EU Parliament authentication
//!
//! Authentication type: None (public API)
//!
//! EU Parliament Open Data API is completely public and requires no authentication.

use std::collections::HashMap;

/// EU Parliament authentication credentials (stub - no auth required)
#[derive(Clone)]
pub struct EuParliamentAuth;

impl EuParliamentAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// No authentication to add
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Always returns true (public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for EuParliamentAuth {
    fn default() -> Self {
        Self::new()
    }
}
