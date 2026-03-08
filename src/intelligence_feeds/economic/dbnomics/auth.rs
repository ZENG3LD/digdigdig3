//! DBnomics authentication
//!
//! Authentication type: None
//!
//! DBnomics API is completely open and does not require any authentication.
//! No API keys, no headers, no signatures - just plain HTTP GET requests.

use std::collections::HashMap;

/// DBnomics authentication (no auth required)
#[derive(Clone, Default)]
pub struct DBnomicsAuth;

impl DBnomicsAuth {
    /// Create new auth (no-op, as DBnomics doesn't require auth)
    pub fn new() -> Self {
        Self
    }

    /// Sign query parameters (no-op for DBnomics)
    ///
    /// This method exists for API consistency with other connectors,
    /// but does nothing since DBnomics doesn't require authentication.
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured (always false for DBnomics)
    pub fn is_authenticated(&self) -> bool {
        false
    }
}
