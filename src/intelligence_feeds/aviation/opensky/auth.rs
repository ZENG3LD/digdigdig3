//! OpenSky Network authentication
//!
//! Authentication type: Optional Basic Auth
//!
//! OpenSky supports both anonymous and authenticated access:
//! - Anonymous: 10 requests per 10 seconds
//! - Authenticated: 4000 credits per day (credit cost varies by endpoint)
//!
//! Authentication uses HTTP Basic Auth with username and password.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use base64::{Engine as _, engine::general_purpose};

/// OpenSky Network authentication credentials
#[derive(Clone)]
pub struct OpenskyAuth {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl OpenskyAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variables: `OPENSKY_USERNAME`, `OPENSKY_PASSWORD`
    /// If not present, will use anonymous access
    pub fn from_env() -> Self {
        Self {
            username: std::env::var("OPENSKY_USERNAME").ok(),
            password: std::env::var("OPENSKY_PASSWORD").ok(),
        }
    }

    /// Create auth with explicit credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: Some(username.into()),
            password: Some(password.into()),
        }
    }

    /// Create anonymous auth (no credentials)
    pub fn anonymous() -> Self {
        Self {
            username: None,
            password: None,
        }
    }

    /// Add authentication to request headers
    ///
    /// OpenSky uses HTTP Basic Authentication:
    /// `Authorization: Basic base64(username:password)`
    pub fn sign_headers(&self, headers: &mut HeaderMap) {
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            let credentials = format!("{}:{}", username, password);
            let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
            let auth_value = format!("Basic {}", encoded);

            if let Ok(header_value) = HeaderValue::from_str(&auth_value) {
                headers.insert(AUTHORIZATION, header_value);
            }
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }

    /// Get username (for debugging/logging)
    pub fn get_username(&self) -> Option<&str> {
        self.username.as_deref()
    }
}

impl Default for OpenskyAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
