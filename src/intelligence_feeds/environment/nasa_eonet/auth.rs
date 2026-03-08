//! NASA EONET authentication
//!
//! Authentication type: None required
//!
//! NASA EONET API is completely open and does not require authentication.
//! Optional NASA API key can be used for intensive use but not implemented here.

use std::collections::HashMap;

/// NASA EONET authentication credentials (none required)
#[derive(Clone)]
pub struct NasaEonetAuth;

impl NasaEonetAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required
    }
}

impl Default for NasaEonetAuth {
    fn default() -> Self {
        Self::new()
    }
}
