//! KRX authentication
//!
//! Authentication type: API Key (simple header-based or query parameter)
//!
//! KRX supports two API key types:
//! 1. Open API Key (openapi.krx.co.kr) - AUTH_KEY header
//! 2. Public Data Portal Service Key (data.go.kr) - serviceKey query param

use std::collections::HashMap;

/// KRX authentication credentials
#[derive(Clone)]
pub struct KrxAuth {
    /// API key for Open API (AUTH_KEY header)
    pub auth_key: Option<String>,
    /// Service key for Public Data Portal (optional)
    pub public_data_portal_key: Option<String>,
}

impl KrxAuth {
    /// Create new auth from environment variables
    ///
    /// Expected environment variables:
    /// - `KRX_AUTH_KEY`: Open API authentication key
    /// - `KRX_DATA_PORTAL_KEY`: Public Data Portal service key
    pub fn from_env() -> Self {
        Self {
            auth_key: std::env::var("KRX_AUTH_KEY").ok(),
            public_data_portal_key: std::env::var("KRX_DATA_PORTAL_KEY").ok(),
        }
    }

    /// Create auth with Open API key
    pub fn new_openapi(auth_key: impl Into<String>) -> Self {
        Self {
            auth_key: Some(auth_key.into()),
            public_data_portal_key: None,
        }
    }

    /// Create auth with Public Data Portal service key
    pub fn new_portal(service_key: impl Into<String>) -> Self {
        Self {
            auth_key: None,
            public_data_portal_key: Some(service_key.into()),
        }
    }

    /// Create auth with both keys
    pub fn new_full(auth_key: impl Into<String>, portal_key: impl Into<String>) -> Self {
        Self {
            auth_key: Some(auth_key.into()),
            public_data_portal_key: Some(portal_key.into()),
        }
    }

    /// Add authentication headers for Open API
    ///
    /// The new KRX Open API uses simple AUTH_KEY header authentication.
    /// No browser headers or HMAC signing required.
    pub fn sign_openapi_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.auth_key {
            headers.insert("AUTH_KEY".to_string(), key.clone());
        }

        // Add standard JSON headers for Open API
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());
    }

    /// Add authentication to query params for Public Data Portal
    ///
    /// Public Data Portal uses `serviceKey` query parameter
    pub fn sign_portal_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.public_data_portal_key {
            params.insert("serviceKey".to_string(), key.clone());
        }
    }

    /// Check if we have Open API authentication
    pub fn has_openapi_auth(&self) -> bool {
        self.auth_key.is_some()
    }

    /// Check if we have Public Data Portal authentication
    pub fn has_portal_auth(&self) -> bool {
        self.public_data_portal_key.is_some()
    }
}

impl Default for KrxAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
