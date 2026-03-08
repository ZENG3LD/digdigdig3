//! RSS Feed Proxy authentication
//!
//! Authentication type: None required
//!
//! RSS feeds are public and do not require authentication.
//! We only add a User-Agent header to identify ourselves.

use std::collections::HashMap;

/// RSS Feed Proxy authentication (none required, just User-Agent)
#[derive(Clone)]
pub struct RssProxyAuth;

impl RssProxyAuth {
    /// Create new auth (no-op)
    pub fn new() -> Self {
        Self
    }

    /// Add User-Agent header for polite crawling
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (compatible; NEMO-Terminal/1.0; +https://github.com/nemo)".to_string(),
        );
    }
}

impl Default for RssProxyAuth {
    fn default() -> Self {
        Self::new()
    }
}
