//! UNHCR authentication
//!
//! Authentication type: None (public API)
//!
//! The UNHCR API is completely public and requires no authentication.

/// UNHCR authentication credentials
#[derive(Clone, Debug)]
pub struct UnhcrAuth;

impl UnhcrAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnhcrAuth {
    fn default() -> Self {
        Self::new()
    }
}
