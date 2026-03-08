//! Bank of England authentication
//!
//! Authentication type: None
//!
//! BoE data is public and requires no authentication.
//! This module exists for consistency with the connector pattern.

use std::collections::HashMap;

/// Bank of England authentication (none required)
#[derive(Clone, Default)]
pub struct BoeAuth;

impl BoeAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for BoE)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Add authentication to query parameters (no-op for BoE)
    ///
    /// BoE does not require authentication, so this does nothing.
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured (always true for BoE)
    pub fn is_authenticated(&self) -> bool {
        true // No auth needed
    }
}
