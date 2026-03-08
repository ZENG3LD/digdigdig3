//! ECB authentication
//!
//! Authentication type: None (public API)
//!
//! ECB API does not require authentication. All data is publicly available.

use std::collections::HashMap;

/// ECB authentication credentials
///
/// ECB API does not require authentication, so this is a placeholder struct
#[derive(Clone, Default)]
pub struct EcbAuth;

impl EcbAuth {
    /// Create new auth instance
    ///
    /// No credentials needed for ECB
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to query parameters
    ///
    /// ECB does not require authentication, so this is a no-op
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }

    /// Check if authentication is configured
    ///
    /// Always returns true since ECB doesn't require auth
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
