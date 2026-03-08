//! Feodo Tracker authentication
//!
//! Authentication type: None required
//!
//! Feodo Tracker API is completely open and does not require authentication.

use std::collections::HashMap;

/// Feodo Tracker authentication credentials (none required)
#[derive(Clone)]
pub struct FeodoTrackerAuth;

impl FeodoTrackerAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for FeodoTrackerAuth {
    fn default() -> Self {
        Self::new()
    }
}
