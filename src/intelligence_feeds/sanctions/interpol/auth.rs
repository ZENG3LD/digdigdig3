//! INTERPOL authentication
//!
//! Authentication type: None (completely public API)
//!
//! INTERPOL API is completely public and requires no authentication.

use std::collections::HashMap;

/// INTERPOL authentication (none required)
#[derive(Clone)]
pub struct InterpolAuth;

impl InterpolAuth {
    /// Create new auth from environment variable
    ///
    /// No environment variables needed - INTERPOL API is public
    pub fn from_env() -> Self {
        Self
    }

    /// Create auth instance
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for INTERPOL)
    ///
    /// INTERPOL API requires no authentication
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication needed
    }

    /// Add authentication to headers (no-op for INTERPOL)
    ///
    /// INTERPOL API requires no authentication
    pub fn sign_headers(&self, _headers: &mut HashMap<String, String>) {
        // No authentication needed
    }
}

impl Default for InterpolAuth {
    fn default() -> Self {
        Self::new()
    }
}
