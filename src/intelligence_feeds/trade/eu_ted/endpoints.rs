//! EU TED API endpoints

/// Base URLs for EU TED API
pub struct EuTedEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for EuTedEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://ted.europa.eu/api/v3.0",
            ws_base: None, // EU TED does not support WebSocket
        }
    }
}

/// EU TED API endpoint enum
#[derive(Debug, Clone)]
pub enum EuTedEndpoint {
    /// Search procurement notices (POST)
    SearchNotices,
    /// Get specific notice by ID
    NoticeDetail { notice_id: String },
    /// Search business entities (POST)
    SearchEntities,
    /// Get specific entity by ID
    EntityDetail { entity_id: String },
    /// Get codelist values
    Codelist { codelist_id: String },
}

impl EuTedEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::SearchNotices => "/notices/search".to_string(),
            Self::NoticeDetail { notice_id } => format!("/notices/{}", notice_id),
            Self::SearchEntities => "/business-entities/search".to_string(),
            Self::EntityDetail { entity_id } => format!("/business-entities/{}", entity_id),
            Self::Codelist { codelist_id } => format!("/codelists/{}", codelist_id),
        }
    }
}
