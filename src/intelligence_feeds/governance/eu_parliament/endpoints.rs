//! EU Parliament API endpoints

/// Base URLs for EU Parliament API
pub struct EuParliamentEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for EuParliamentEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://data.europarl.europa.eu/api/v1",
            ws_base: None, // EU Parliament does not support WebSocket
        }
    }
}

/// EU Parliament API endpoint enum
#[derive(Debug, Clone)]
pub enum EuParliamentEndpoint {
    // MEP endpoints
    /// Get list of Members of European Parliament
    Meps,
    /// Get MEP details by ID
    MepById,

    // Document endpoints
    /// Get list of plenary documents
    PlenaryDocuments,
    /// Get document details by ID
    DocumentById,

    // Meeting endpoints
    /// Get list of meetings
    Meetings,

    // Committee endpoints
    /// Get list of committees
    Committees,
}

impl EuParliamentEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Meps => "/meps",
            Self::MepById => "/meps", // ID appended in connector
            Self::PlenaryDocuments => "/plenary-documents",
            Self::DocumentById => "/plenary-documents", // ID appended in connector
            Self::Meetings => "/meetings",
            Self::Committees => "/committees",
        }
    }
}
