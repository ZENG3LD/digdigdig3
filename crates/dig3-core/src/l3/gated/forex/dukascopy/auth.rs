//! Dukascopy authentication
//!
//! Authentication type: None (public datafeed)
//!
//! Dukascopy's historical tick data is publicly accessible via direct HTTP downloads.
//! No API keys, tokens, or authentication headers are required.

use std::collections::HashMap;

/// Authentication credentials for Dukascopy
///
/// Note: No authentication is required for historical tick data downloads.
/// This struct exists for API consistency but holds no credentials.
#[derive(Clone, Debug, Default)]
pub struct DukascopyAuth;

impl DukascopyAuth {
    /// Create new auth instance (no credentials needed)
    pub fn new() -> Self {
        Self
    }

    /// Create auth from environment (no-op for Dukascopy)
    pub fn from_env() -> Self {
        Self
    }

    /// Add authentication headers (no-op for Dukascopy)
    ///
    /// Historical tick data is publicly accessible, no headers needed.
    pub fn sign_headers(&self, _headers: &mut HashMap<String, String>) {
        // No authentication required for public datafeed
    }

    /// Add authentication to query params (no-op for Dukascopy)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required for public datafeed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let auth = DukascopyAuth::new();
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);
        assert!(headers.is_empty(), "No headers should be added");
    }

    #[test]
    fn test_auth_from_env() {
        let auth = DukascopyAuth::from_env();
        let mut params = HashMap::new();
        auth.sign_query(&mut params);
        assert!(params.is_empty(), "No query params should be added");
    }
}
