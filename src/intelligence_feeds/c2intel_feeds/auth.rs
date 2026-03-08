//! C2IntelFeeds authentication
//!
//! Authentication type: None required
//!
//! C2IntelFeeds is a public GitHub repository and does not require authentication.

use std::collections::HashMap;

/// C2IntelFeeds authentication credentials (none required)
#[derive(Clone)]
pub struct C2IntelFeedsAuth;

impl C2IntelFeedsAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for C2IntelFeedsAuth {
    fn default() -> Self {
        Self::new()
    }
}
