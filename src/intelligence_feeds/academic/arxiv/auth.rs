//! arXiv authentication
//!
//! Authentication type: None required
//!
//! arXiv API is completely open and does not require authentication.

use std::collections::HashMap;

/// arXiv authentication credentials (none required)
#[derive(Clone)]
pub struct ArxivAuth;

impl ArxivAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for ArxivAuth {
    fn default() -> Self {
        Self::new()
    }
}
