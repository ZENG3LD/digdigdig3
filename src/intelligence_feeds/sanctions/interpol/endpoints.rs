//! INTERPOL API endpoints

/// Base URLs for INTERPOL API
pub struct InterpolEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for InterpolEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://ws-public.interpol.int/notices/v1",
            ws_base: None, // INTERPOL does not support WebSocket
        }
    }
}

/// INTERPOL API endpoint enum
#[derive(Debug, Clone)]
pub enum InterpolEndpoint {
    /// Search red notices (wanted persons)
    RedNotices,
    /// Search yellow notices (missing persons)
    YellowNotices,
    /// Search UN Security Council notices
    UnNotices,
    /// Get individual red notice details
    RedNoticeDetail { notice_id: String },
    /// Get images for a red notice
    RedNoticeImages { notice_id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get individual yellow notice details by ID
    YellowNoticeDetail { notice_id: String },
    /// Get images for a yellow notice
    YellowNoticeImages { notice_id: String },
    /// Get individual UN notice persons (individuals) by entity ID
    UnPersons,
    /// Get individual UN notice entities (organizations) by entity ID
    UnEntities,
}

impl InterpolEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::RedNotices => "/red".to_string(),
            Self::YellowNotices => "/yellow".to_string(),
            Self::UnNotices => "/un".to_string(),
            Self::RedNoticeDetail { notice_id } => format!("/red/{}", notice_id),
            Self::RedNoticeImages { notice_id } => format!("/red/{}/images", notice_id),

            // C7 additions
            Self::YellowNoticeDetail { notice_id } => format!("/yellow/{}", notice_id),
            Self::YellowNoticeImages { notice_id } => format!("/yellow/{}/images", notice_id),
            Self::UnPersons => "/un/persons".to_string(),
            Self::UnEntities => "/un/entities".to_string(),
        }
    }
}
