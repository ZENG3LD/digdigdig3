//! NASA EONET API endpoints

/// Base URLs for NASA EONET API
pub struct NasaEonetEndpoints {
    pub rest_base: String,
}

impl Default for NasaEonetEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://eonet.gsfc.nasa.gov/api/v3".to_string(),
        }
    }
}

/// NASA EONET API endpoint enum
#[derive(Debug, Clone)]
pub enum NasaEonetEndpoint {
    /// Events endpoint
    Events,
    /// Categories endpoint
    Categories,
    /// Sources endpoint
    Sources,
}

impl NasaEonetEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Events => "/events",
            Self::Categories => "/categories",
            Self::Sources => "/sources",
        }
    }
}
