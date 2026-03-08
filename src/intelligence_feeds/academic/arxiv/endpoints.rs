//! arXiv API endpoints

/// Base URLs for arXiv API
pub struct ArxivEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ArxivEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://export.arxiv.org",
            ws_base: None, // arXiv does not support WebSocket
        }
    }
}

/// arXiv API endpoint enum
#[derive(Debug, Clone)]
pub enum ArxivEndpoint {
    /// Query/search endpoint
    Query,
}

impl ArxivEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Query => "/api/query",
        }
    }
}
