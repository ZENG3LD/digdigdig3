//! USASpending.gov authentication
//!
//! Authentication type: None (completely public API)
//!
//! USASpending.gov is a completely public API with no authentication required.

/// USASpending.gov authentication credentials (empty - no auth required)
#[derive(Clone)]
pub struct UsaSpendingAuth;

impl UsaSpendingAuth {
    /// Create new auth from environment variable
    ///
    /// No environment variables needed - public API
    pub fn from_env() -> Self {
        Self
    }

    /// Create new auth instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for UsaSpendingAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
