//! IMF authentication
//!
//! Authentication type: None
//!
//! IMF API is completely public - no authentication required.

use std::collections::HashMap;

/// IMF authentication (none required)
#[derive(Clone, Default)]
pub struct ImfAuth;

impl ImfAuth {
    /// Create new auth instance
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for IMF)
    pub fn from_env() -> Self {
        Self
    }

    /// Sign query parameters (no-op for IMF - no auth needed)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // IMF requires no authentication
    }

    /// Check if authentication is configured (always true for IMF)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
