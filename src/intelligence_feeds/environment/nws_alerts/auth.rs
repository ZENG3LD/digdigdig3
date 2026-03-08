//! NWS Weather Alerts authentication
//!
//! Authentication type: None (User-Agent required)
//!
//! NWS API requires a User-Agent header but no API key.

use std::collections::HashMap;

/// NWS Alerts authentication (User-Agent only)
#[derive(Clone)]
pub struct NwsAlertsAuth {
    user_agent: String,
}

impl NwsAlertsAuth {
    /// Create new auth with custom User-Agent
    pub fn new(user_agent: String) -> Self {
        Self { user_agent }
    }

    /// Add User-Agent header to request
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        headers.insert("User-Agent".to_string(), self.user_agent.clone());
    }
}

impl Default for NwsAlertsAuth {
    fn default() -> Self {
        Self::new("NemoTrading/1.0 (nemo@trading.system)".to_string())
    }
}
