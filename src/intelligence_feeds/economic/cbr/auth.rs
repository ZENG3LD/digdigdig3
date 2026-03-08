//! CBR authentication
//!
//! Authentication type: NONE
//!
//! The Central Bank of Russia API is public and does not require authentication.
//! All endpoints are open and free to use.

use std::collections::HashMap;

/// CBR authentication (no-op - CBR doesn't require auth)
#[derive(Clone, Default)]
pub struct CbrAuth;

impl CbrAuth {
    /// Create new auth (no-op for CBR)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for CBR)
    pub fn from_env() -> Self {
        Self
    }

    /// Sign query parameters (no-op for CBR)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // CBR doesn't require authentication
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}
