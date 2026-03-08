//! RIPE NCC authentication
//!
//! Authentication type: None (public API)
//!
//! RIPE NCC RIPEstat API is completely open and requires no authentication.
//! This is a stub implementation for consistency with other connectors.

use std::collections::HashMap;

/// RIPE NCC authentication credentials (stub - no auth required)
#[derive(Clone, Default)]
pub struct RipeNccAuth;

impl RipeNccAuth {
    /// Create new auth (no-op for RIPE NCC)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for RIPE NCC)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for RIPE NCC)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required - this is a no-op
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true // Always authenticated since no auth is required
    }
}
