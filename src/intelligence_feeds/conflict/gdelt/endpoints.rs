//! GDELT API endpoints

/// Base URLs for GDELT API
pub struct GdeltEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for GdeltEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.gdeltproject.org/api/v2",
            ws_base: None, // GDELT does not support WebSocket
        }
    }
}

/// GDELT API endpoint enum
#[derive(Debug, Clone)]
pub enum GdeltEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DOC API - News/Events Search
    // ═══════════════════════════════════════════════════════════════════════
    /// Search articles/events
    DocApi,

    // ═══════════════════════════════════════════════════════════════════════
    // GEO API - Geographic Events
    // ═══════════════════════════════════════════════════════════════════════
    /// Search geographic events
    GeoApi,

    // ═══════════════════════════════════════════════════════════════════════
    // TV API - Television Monitoring
    // ═══════════════════════════════════════════════════════════════════════
    /// Search television content
    TvApi,

    // ═══════════════════════════════════════════════════════════════════════
    // CONTEXT API
    // ═══════════════════════════════════════════════════════════════════════
    /// Get context for query
    ContextApi,
}

impl GdeltEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::DocApi => "/doc/doc",
            Self::GeoApi => "/geo/geo",
            Self::TvApi => "/tv/tv",
            Self::ContextApi => "/context/context",
        }
    }
}

/// DOC API modes
#[derive(Debug, Clone)]
pub enum DocMode {
    /// Article list view
    ArtList,
    /// Article gallery with images
    ArtGallery,
    /// Timeline of article volume
    TimelineVol,
    /// Timeline of article tone
    TimelineTone,
    /// Timeline by language
    TimelineLang,
    /// Timeline by source country
    TimelineSourceCountry,
}

impl DocMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ArtList => "artlist",
            Self::ArtGallery => "artgallery",
            Self::TimelineVol => "timelinevol",
            Self::TimelineTone => "timelinetone",
            Self::TimelineLang => "timelinelang",
            Self::TimelineSourceCountry => "timelinesourcecountry",
        }
    }
}

/// GEO API modes
#[derive(Debug, Clone)]
pub enum GeoMode {
    /// Point data
    PointData,
    /// Point heatmap
    PointHeatmap,
    /// Point pattern
    PointPattern,
}

impl GeoMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PointData => "PointData",
            Self::PointHeatmap => "PointHeatmap",
            Self::PointPattern => "PointPattern",
        }
    }
}

/// TV API modes
#[derive(Debug, Clone)]
pub enum TvMode {
    /// Raw timeline volume
    TimelineVolRaw,
    /// Timeline volume
    TimelineVol,
    /// Tone chart
    ToneChart,
    /// Word cloud
    WordCloud,
    /// Clip gallery
    ClipGallery,
}

impl TvMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TimelineVolRaw => "timelinevolraw",
            Self::TimelineVol => "timelinevol",
            Self::ToneChart => "tonechart",
            Self::WordCloud => "wordcloud",
            Self::ClipGallery => "clipgallery",
        }
    }
}

/// Sort order for DOC API
#[derive(Debug, Clone)]
pub enum SortOrder {
    /// Most recent first
    DateDesc,
    /// Oldest first
    DateAsc,
    /// Highest tone first
    ToneDesc,
    /// Lowest tone first
    ToneAsc,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DateDesc => "datedesc",
            Self::DateAsc => "dateasc",
            Self::ToneDesc => "tonedesc",
            Self::ToneAsc => "toneasc",
        }
    }
}

/// Format datetime for GDELT API (YYYYMMDDHHMMSS)
///
/// # Arguments
/// - `datetime` - ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)
///
/// # Returns
/// GDELT format: YYYYMMDDHHMMSS
pub fn format_gdelt_datetime(datetime: &str) -> String {
    // Remove hyphens, colons, and 'T'
    let cleaned: String = datetime
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    // Pad with zeros if needed
    format!("{:0<14}", cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_gdelt_datetime() {
        assert_eq!(format_gdelt_datetime("2024-01-01"), "20240101000000");
        assert_eq!(format_gdelt_datetime("2024-01-01T12:30:45"), "20240101123045");
        assert_eq!(format_gdelt_datetime("20240101"), "20240101000000");
    }
}
