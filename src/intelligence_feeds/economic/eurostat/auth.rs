//! Eurostat authentication
//!
//! Authentication type: NONE
//!
//! Eurostat API is completely open and requires no authentication.
//! No API keys, no signatures, no registration required.

use std::collections::HashMap;

/// Eurostat authentication (none required)
#[derive(Clone, Default)]
pub struct EurostatAuth;

impl EurostatAuth {
    /// Create new auth (no-op, no authentication needed)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op, no authentication needed)
    pub fn from_env() -> Self {
        Self
    }

    /// Sign query parameters (no-op for Eurostat)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // Eurostat requires no authentication
    }

    /// Check if authenticated (always true - no auth required)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
