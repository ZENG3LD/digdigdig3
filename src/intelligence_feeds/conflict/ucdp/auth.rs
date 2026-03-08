//! UCDP authentication
//!
//! Authentication type: None
//!
//! UCDP API is completely public and requires no authentication.

use std::collections::HashMap;

/// UCDP authentication credentials (stub - no auth required)
#[derive(Clone)]
pub struct UcdpAuth;

impl UcdpAuth {
    /// Create new auth (no-op for UCDP)
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for UCDP)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // UCDP API requires no authentication
    }

    /// Check if authentication is configured (always true for UCDP)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for UcdpAuth {
    fn default() -> Self {
        Self::new()
    }
}
