//! NGA Maritime Warnings authentication
//!
//! Authentication type: None required
//!
//! NGA MSI API is completely open and does not require authentication.

use std::collections::HashMap;

/// NGA Maritime Warnings authentication credentials (none required)
#[derive(Clone)]
pub struct NgaWarningsAuth;

impl NgaWarningsAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for NgaWarningsAuth {
    fn default() -> Self {
        Self::new()
    }
}
