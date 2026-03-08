//! IMF PortWatch authentication
//!
//! Authentication type: None (public API)
//!
//! IMF PortWatch is completely free and requires no authentication.
//! This module provides a stub for consistency with other connectors.

use std::collections::HashMap;

/// IMF PortWatch authentication (stub - no auth required)
#[derive(Clone)]
pub struct ImfPortWatchAuth;

impl ImfPortWatchAuth {
    /// Create new auth (no-op for public API)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for public API)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op - API is public)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for ImfPortWatchAuth {
    fn default() -> Self {
        Self::new()
    }
}
