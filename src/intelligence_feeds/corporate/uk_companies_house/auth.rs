//! UK Companies House authentication
//!
//! Authentication type: HTTP Basic Auth (API key as username, empty password)
//!
//! Companies House uses HTTP Basic Authentication where the API key is the username
//! and the password is an empty string.

/// UK Companies House authentication credentials
#[derive(Clone)]
pub struct UkCompaniesHouseAuth {
    pub api_key: Option<String>,
}

impl UkCompaniesHouseAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `COMPANIES_HOUSE_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("COMPANIES_HOUSE_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Get Basic Auth credentials for reqwest
    ///
    /// Returns API key as username, empty string as password
    pub fn get_basic_auth(&self) -> Option<String> {
        self.api_key.clone()
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

impl Default for UkCompaniesHouseAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
