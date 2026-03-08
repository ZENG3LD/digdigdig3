//! PredictIt authentication
//!
//! Authentication type: None (public API)
//!
//! PredictIt API is completely public and requires no authentication.

use std::collections::HashMap;

/// PredictIt authentication credentials (none required)
#[derive(Clone)]
pub struct PredictItAuth;

impl PredictItAuth {
    /// Create new auth (no-op for PredictIt)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for PredictIt)
    pub fn from_env() -> Self {
        Self
    }

    /// Sign query (no-op for PredictIt - no authentication needed)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authenticated (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for PredictItAuth {
    fn default() -> Self {
        Self::new()
    }
}
