//! GLEIF authentication
//!
//! Authentication type: None (public API)
//!
//! GLEIF API is completely free and public. No authentication required.

use std::collections::HashMap;

/// GLEIF authentication (no-op for public API)
#[derive(Clone)]
pub struct GleifAuth;

impl GleifAuth {
    /// Create auth from environment (no-op for GLEIF)
    pub fn from_env() -> Self {
        Self
    }

    /// Create new auth (no-op for GLEIF)
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to HTTP headers (no-op for GLEIF)
    pub fn sign_headers(&self, _headers: &mut HashMap<String, String>) {
        // No authentication required for GLEIF
    }
}

impl Default for GleifAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
