//! FAA Airport Status authentication
//!
//! Authentication type: None required
//!
//! FAA NASSTATUS API is completely open and does not require authentication.

use std::collections::HashMap;

/// FAA Airport Status authentication credentials (none required)
#[derive(Clone)]
pub struct FaaStatusAuth;

impl FaaStatusAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for FaaStatusAuth {
    fn default() -> Self {
        Self::new()
    }
}
