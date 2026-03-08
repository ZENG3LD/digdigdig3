//! OpenFIGI API endpoints

/// Base URLs for OpenFIGI API
pub struct OpenFigiEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OpenFigiEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.openfigi.com",
            ws_base: None, // OpenFIGI does not support WebSocket
        }
    }
}

/// OpenFIGI API endpoint enum
#[derive(Debug, Clone)]
pub enum OpenFigiEndpoint {
    /// Map identifiers to FIGIs (MAIN endpoint)
    /// POST /v3/mapping
    Mapping,

    /// Search by text query
    /// POST /v3/search
    Search,

    /// Get enum values for a field
    /// GET /v3/mapping/values/{key}
    MappingValues(String),
}

impl OpenFigiEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Mapping => "/v3/mapping".to_string(),
            Self::Search => "/v3/search".to_string(),
            Self::MappingValues(key) => format!("/v3/mapping/values/{}", key),
        }
    }
}
