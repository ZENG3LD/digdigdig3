//! GDELT authentication
//!
//! Authentication type: None
//!
//! GDELT API is completely public and requires no authentication.
//! This module exists for consistency with the connector pattern.

use std::collections::HashMap;

/// GDELT authentication (no auth required)
#[derive(Clone, Default)]
pub struct GdeltAuth;

impl GdeltAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for GDELT)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op for GDELT)
    ///
    /// GDELT API is public and requires no authentication
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication needed for GDELT
    }

    /// Check if authentication is configured (always true for GDELT)
    pub fn is_authenticated(&self) -> bool {
        true // GDELT is public
    }
}
