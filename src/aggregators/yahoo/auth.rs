//! Yahoo Finance authentication
//!
//! Yahoo Finance uses a cookie-crumb authentication system for historical data downloads.
//! Most endpoints don't require authentication.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication credentials for Yahoo Finance
#[derive(Clone)]
pub struct YahooFinanceAuth {
    /// Cookie string (obtained from visiting finance.yahoo.com)
    pub cookie: Option<String>,
    /// Crumb token (obtained from /v1/test/getcrumb with valid cookie)
    pub crumb: Option<String>,
    /// Timestamp when crumb was obtained (for caching)
    pub crumb_timestamp: Option<u64>,
}

impl Default for YahooFinanceAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl YahooFinanceAuth {
    /// Create new empty auth (most endpoints don't need authentication)
    pub fn new() -> Self {
        Self {
            cookie: None,
            crumb: None,
            crumb_timestamp: None,
        }
    }

    /// Create auth from environment variables
    ///
    /// Reads:
    /// - `YAHOO_FINANCE_COOKIE` - Full cookie string
    /// - `YAHOO_FINANCE_CRUMB` - Crumb token
    pub fn from_env() -> Self {
        let cookie = std::env::var("YAHOO_FINANCE_COOKIE").ok();
        let crumb = std::env::var("YAHOO_FINANCE_CRUMB").ok();
        let crumb_timestamp = crumb.as_ref().map(|_| current_timestamp());

        Self {
            cookie,
            crumb,
            crumb_timestamp,
        }
    }

    /// Create auth with explicit cookie and crumb
    pub fn with_cookie_crumb(cookie: impl Into<String>, crumb: impl Into<String>) -> Self {
        Self {
            cookie: Some(cookie.into()),
            crumb: Some(crumb.into()),
            crumb_timestamp: Some(current_timestamp()),
        }
    }

    /// Set cookie
    pub fn set_cookie(&mut self, cookie: impl Into<String>) {
        self.cookie = Some(cookie.into());
    }

    /// Set crumb
    pub fn set_crumb(&mut self, crumb: impl Into<String>) {
        self.crumb = Some(crumb.into());
        self.crumb_timestamp = Some(current_timestamp());
    }

    /// Check if crumb is expired (older than 30 minutes)
    pub fn is_crumb_expired(&self) -> bool {
        match self.crumb_timestamp {
            Some(ts) => {
                let current = current_timestamp();
                current - ts > 1800 // 30 minutes in seconds
            }
            None => true, // No timestamp means expired
        }
    }

    /// Add authentication headers to request
    ///
    /// Adds:
    /// - Cookie header (if cookie is set)
    /// - User-Agent (required to avoid bot detection)
    /// - Additional headers to mimic browser behavior
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        // Add cookie if available
        if let Some(cookie) = &self.cookie {
            headers.insert("Cookie".to_string(), cookie.clone());
        }

        // Add required User-Agent (critical for Yahoo Finance)
        if !headers.contains_key("User-Agent") {
            headers.insert(
                "User-Agent".to_string(),
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            );
        }

        // Add other headers to mimic browser
        headers.entry("Accept".to_string())
            .or_insert("application/json".to_string());

        headers.entry("Accept-Language".to_string())
            .or_insert("en-US,en;q=0.9".to_string());

        headers.entry("Referer".to_string())
            .or_insert("https://finance.yahoo.com/".to_string());
    }

    /// Add authentication to query params (for endpoints that need crumb in URL)
    ///
    /// Adds crumb parameter if available
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(crumb) = &self.crumb {
            params.insert("crumb".to_string(), crumb.clone());
        }
    }

    /// Check if we have valid authentication for download endpoint
    pub fn has_download_auth(&self) -> bool {
        self.cookie.is_some() && self.crumb.is_some() && !self.is_crumb_expired()
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let auth = YahooFinanceAuth::new();
        assert!(auth.cookie.is_none());
        assert!(auth.crumb.is_none());
    }

    #[test]
    fn test_auth_with_cookie_crumb() {
        let auth = YahooFinanceAuth::with_cookie_crumb("test_cookie", "test_crumb");
        assert_eq!(auth.cookie, Some("test_cookie".to_string()));
        assert_eq!(auth.crumb, Some("test_crumb".to_string()));
        assert!(auth.crumb_timestamp.is_some());
    }

    #[test]
    fn test_sign_headers() {
        let auth = YahooFinanceAuth::with_cookie_crumb("test_cookie", "test_crumb");
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);

        assert_eq!(headers.get("Cookie"), Some(&"test_cookie".to_string()));
        assert!(headers.contains_key("User-Agent"));
        assert!(headers.contains_key("Accept"));
    }

    #[test]
    fn test_sign_query() {
        let auth = YahooFinanceAuth::with_cookie_crumb("test_cookie", "test_crumb");
        let mut params = HashMap::new();
        auth.sign_query(&mut params);

        assert_eq!(params.get("crumb"), Some(&"test_crumb".to_string()));
    }

    #[test]
    fn test_has_download_auth() {
        let auth = YahooFinanceAuth::new();
        assert!(!auth.has_download_auth());

        let auth = YahooFinanceAuth::with_cookie_crumb("cookie", "crumb");
        assert!(auth.has_download_auth());
    }
}
