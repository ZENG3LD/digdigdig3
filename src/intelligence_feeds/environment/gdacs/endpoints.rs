//! GDACS API endpoints

/// Base URLs for GDACS API
pub struct GdacsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for GdacsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.gdacs.org/gdacsapi/api",
            ws_base: None, // GDACS does not support WebSocket
        }
    }
}

/// GDACS API endpoint enum
#[derive(Debug, Clone)]
pub enum GdacsEndpoint {
    /// Get event list with filters
    EventList,
    /// Get event by ID
    EventById,
}

impl GdacsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::EventList => "/events/geteventlist/SEARCH",
            Self::EventById => "/events/geteventdata/GetByEventId",
        }
    }
}
