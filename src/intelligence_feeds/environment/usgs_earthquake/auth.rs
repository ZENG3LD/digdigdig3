//! USGS Earthquake authentication
//!
//! Authentication type: None
//!
//! USGS Earthquake API is completely public - no authentication required.

use std::collections::HashMap;

/// USGS Earthquake authentication credentials (none required)
#[derive(Clone)]
pub struct UsgsEarthquakeAuth;

impl UsgsEarthquakeAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to query parameters (no-op)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required for USGS
    }

    /// Check if authentication is configured (always true)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for UsgsEarthquakeAuth {
    fn default() -> Self {
        Self::new()
    }
}
