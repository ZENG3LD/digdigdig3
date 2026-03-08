//! ACLED authentication
//!
//! Authentication type: API Key + Email (query parameters)
//!
//! ACLED requires both an API key and email address as query parameters.
//! Additionally, all requests must include `terms=accept` to acknowledge terms of use.

use std::collections::HashMap;

/// ACLED authentication credentials
#[derive(Clone)]
pub struct AcledAuth {
    pub api_key: Option<String>,
    pub email: Option<String>,
}

impl AcledAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variables: `ACLED_API_KEY` and `ACLED_EMAIL`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ACLED_API_KEY").ok(),
            email: std::env::var("ACLED_EMAIL").ok(),
        }
    }

    /// Create auth with explicit API key and email
    pub fn new(api_key: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            email: Some(email.into()),
        }
    }

    /// Add authentication to query parameters
    ///
    /// ACLED requires:
    /// 1. API key as `key` parameter
    /// 2. Email as `email` parameter
    /// 3. Terms acceptance as `terms=accept` (mandatory)
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        // Add terms acceptance (required for all requests)
        params.insert("terms".to_string(), "accept".to_string());

        // Add API key
        if let Some(key) = &self.api_key {
            params.insert("key".to_string(), key.clone());
        }

        // Add email
        if let Some(email) = &self.email {
            params.insert("email".to_string(), email.clone());
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some() && self.email.is_some()
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Get email (for debugging/logging - use carefully)
    pub fn get_email(&self) -> Option<&str> {
        self.email.as_deref()
    }
}

impl Default for AcledAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
