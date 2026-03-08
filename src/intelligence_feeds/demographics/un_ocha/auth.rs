//! UN OCHA HAPI authentication
//!
//! Authentication type: Optional app_identifier header
//!
//! HAPI API is mostly open but supports optional app identification.

use std::collections::HashMap;

/// UN OCHA HAPI authentication credentials
#[derive(Clone)]
pub struct UnOchaAuth {
    /// Optional application identifier for tracking/rate limiting
    app_identifier: Option<String>,
}

impl UnOchaAuth {
    /// Create new auth with optional app identifier
    pub fn new(app_identifier: Option<String>) -> Self {
        Self { app_identifier }
    }

    /// Create auth without identifier (public access)
    pub fn public() -> Self {
        Self {
            app_identifier: None,
        }
    }

    /// Add custom headers if app identifier is provided
    pub fn add_headers(&self, headers: &mut HashMap<String, String>) {
        // Add User-Agent for identification
        headers.insert(
            "User-Agent".to_string(),
            "NEMO-Trading-System/1.0".to_string(),
        );

        // Add app identifier if provided
        if let Some(ref app_id) = self.app_identifier {
            headers.insert("X-App-Identifier".to_string(), app_id.clone());
        }
    }

    /// No-op signing method for API consistency
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No query signing required for HAPI
    }
}

impl Default for UnOchaAuth {
    fn default() -> Self {
        Self::public()
    }
}
