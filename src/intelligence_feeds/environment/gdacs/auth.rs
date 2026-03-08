//! GDACS authentication
//!
//! Authentication type: None required
//!
//! GDACS API is completely open and does not require authentication.

use std::collections::HashMap;

/// GDACS authentication credentials (none required)
#[derive(Clone)]
pub struct GdacsAuth;

impl GdacsAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for GdacsAuth {
    fn default() -> Self {
        Self::new()
    }
}
