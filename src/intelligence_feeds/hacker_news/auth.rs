//! Hacker News authentication
//!
//! Authentication type: None required
//!
//! Hacker News Firebase API is completely open and does not require authentication.

use std::collections::HashMap;

/// Hacker News authentication credentials (none required)
#[derive(Clone)]
pub struct HackerNewsAuth;

impl HackerNewsAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for HackerNewsAuth {
    fn default() -> Self {
        Self::new()
    }
}
