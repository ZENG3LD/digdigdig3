//! SpaceX authentication
//!
//! Authentication type: None (Public API)
//!
//! SpaceX API is completely public and requires no authentication.
//! All endpoints are freely accessible without API keys.

use reqwest::header::HeaderMap;

/// SpaceX authentication credentials (empty - no auth required)
#[derive(Clone)]
pub struct SpaceXAuth;

impl SpaceXAuth {
    /// Create new auth from environment variables
    ///
    /// SpaceX API doesn't require authentication, so this returns empty auth
    pub fn from_env() -> Self {
        Self
    }

    /// Create new auth instance
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to request headers
    ///
    /// SpaceX API doesn't require authentication, so this is a no-op
    pub fn sign_headers(&self, _headers: &mut HeaderMap) {
        // No authentication required for SpaceX API
    }

    /// Check if authentication is configured
    ///
    /// Always returns false since SpaceX API doesn't use authentication
    pub fn is_authenticated(&self) -> bool {
        false
    }
}

impl Default for SpaceXAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
