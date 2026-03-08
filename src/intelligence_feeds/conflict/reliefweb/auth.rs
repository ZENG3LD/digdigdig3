//! ReliefWeb authentication
//!
//! Authentication type: Optional (appname query parameter)
//!
//! ReliefWeb is a public API but recommends using an appname parameter
//! to identify your application for tracking purposes.

use std::collections::HashMap;

/// ReliefWeb authentication credentials
#[derive(Clone)]
pub struct ReliefWebAuth {
    pub appname: Option<String>,
}

impl ReliefWebAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variable: `RELIEFWEB_APPNAME` (optional)
    pub fn from_env() -> Self {
        Self {
            appname: std::env::var("RELIEFWEB_APPNAME").ok(),
        }
    }

    /// Create auth with explicit appname
    pub fn new(appname: impl Into<String>) -> Self {
        Self {
            appname: Some(appname.into()),
        }
    }

    /// Create auth without appname (anonymous access)
    pub fn anonymous() -> Self {
        Self {
            appname: None,
        }
    }

    /// Add authentication to query parameters
    ///
    /// ReliefWeb accepts optional appname parameter for app identification
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(appname) = &self.appname {
            params.insert("appname".to_string(), appname.clone());
        }
    }

    /// Get appname (for debugging/logging - use carefully)
    pub fn get_appname(&self) -> Option<&str> {
        self.appname.as_deref()
    }
}

impl Default for ReliefWebAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
