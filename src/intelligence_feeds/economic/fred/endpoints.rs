//! FRED API endpoints

/// Base URLs for FRED API
pub struct FredEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for FredEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.stlouisfed.org",
            ws_base: None, // FRED does not support WebSocket
        }
    }
}

/// FRED API endpoint enum
#[derive(Debug, Clone)]
pub enum FredEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // CATEGORY ENDPOINTS (6)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get a category by ID
    Category,
    /// Get child categories for a parent category
    CategoryChildren,
    /// Get related categories for a category
    CategoryRelated,
    /// Get economic data series in a category
    CategorySeries,
    /// Get FRED tags for a category
    CategoryTags,
    /// Get related FRED tags within a category
    CategoryRelatedTags,

    // ═══════════════════════════════════════════════════════════════════════
    // RELEASE ENDPOINTS (9)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all releases of economic data
    Releases,
    /// Get release dates for all releases
    ReleasesDates,
    /// Get a release by ID
    Release,
    /// Get release dates for a specific release
    ReleaseDates,
    /// Get economic data series in a release
    ReleaseSeries,
    /// Get sources for a release
    ReleaseSources,
    /// Get FRED tags for a release
    ReleaseTags,
    /// Get related FRED tags for a release
    ReleaseRelatedTags,
    /// Get hierarchical table tree for a release
    ReleaseTables,

    // ═══════════════════════════════════════════════════════════════════════
    // SERIES ENDPOINTS (10) - CORE DATA ACCESS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get an economic data series (metadata)
    Series,
    /// Get categories for a series
    SeriesCategories,
    /// Get observations (data values) for a series - **MOST IMPORTANT**
    SeriesObservations,
    /// Get release for a series
    SeriesRelease,
    /// Search for economic data series matching keywords
    SeriesSearch,
    /// Get FRED tags for a series search
    SeriesSearchTags,
    /// Get related FRED tags for a series search
    SeriesSearchRelatedTags,
    /// Get FRED tags for a series
    SeriesTags,
    /// Get economic data series sorted by when observations were updated
    SeriesUpdates,
    /// Get vintage dates for a series (ALFRED - revision history)
    SeriesVintageDates,

    // ═══════════════════════════════════════════════════════════════════════
    // SOURCE ENDPOINTS (3)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all sources of economic data
    Sources,
    /// Get a source by ID
    Source,
    /// Get releases for a source
    SourceReleases,

    // ═══════════════════════════════════════════════════════════════════════
    // TAG ENDPOINTS (3)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get FRED tags (attributes assigned to series)
    Tags,
    /// Get related FRED tags for one or more FRED tags
    RelatedTags,
    /// Get series matching tags
    TagsSeries,

    // ═══════════════════════════════════════════════════════════════════════
    // GEOFRED ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get GeoFRED series group metadata
    GeoSeriesGroup,
    /// Get GeoFRED series data for mapping
    GeoSeriesData,
    /// Get GeoFRED regional data across geographies
    GeoRegionalData,
    /// Get GeoJSON boundary data for geographical regions
    GeoShapesFile,
}

impl FredEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Categories
            Self::Category => "/fred/category",
            Self::CategoryChildren => "/fred/category/children",
            Self::CategoryRelated => "/fred/category/related",
            Self::CategorySeries => "/fred/category/series",
            Self::CategoryTags => "/fred/category/tags",
            Self::CategoryRelatedTags => "/fred/category/related_tags",

            // Releases
            Self::Releases => "/fred/releases",
            Self::ReleasesDates => "/fred/releases/dates",
            Self::Release => "/fred/release",
            Self::ReleaseDates => "/fred/release/dates",
            Self::ReleaseSeries => "/fred/release/series",
            Self::ReleaseSources => "/fred/release/sources",
            Self::ReleaseTags => "/fred/release/tags",
            Self::ReleaseRelatedTags => "/fred/release/related_tags",
            Self::ReleaseTables => "/fred/release/tables",

            // Series (CORE)
            Self::Series => "/fred/series",
            Self::SeriesCategories => "/fred/series/categories",
            Self::SeriesObservations => "/fred/series/observations",
            Self::SeriesRelease => "/fred/series/release",
            Self::SeriesSearch => "/fred/series/search",
            Self::SeriesSearchTags => "/fred/series/search/tags",
            Self::SeriesSearchRelatedTags => "/fred/series/search/related_tags",
            Self::SeriesTags => "/fred/series/tags",
            Self::SeriesUpdates => "/fred/series/updates",
            Self::SeriesVintageDates => "/fred/series/vintagedates",

            // Sources
            Self::Sources => "/fred/sources",
            Self::Source => "/fred/source",
            Self::SourceReleases => "/fred/source/releases",

            // Tags
            Self::Tags => "/fred/tags",
            Self::RelatedTags => "/fred/related_tags",
            Self::TagsSeries => "/fred/tags/series",

            // GeoFRED
            Self::GeoSeriesGroup => "/geofred/series/group",
            Self::GeoSeriesData => "/geofred/series/data",
            Self::GeoRegionalData => "/geofred/regional/data",
            Self::GeoShapesFile => "/geofred/shapes/file",
        }
    }
}

/// Format series ID for FRED API
///
/// FRED uses series IDs like "GNPCA", "UNRATE", "GDP"
/// This is different from crypto exchanges - there's no base/quote concept.
/// Series IDs are unique identifiers in the FRED database.
///
/// For compatibility with the Symbol type, we'll use:
/// - base = series_id
/// - quote = "" (empty)
pub fn format_series_id(symbol: &crate::core::types::Symbol) -> String {
    // For FRED, the "base" field contains the series ID
    symbol.base.to_uppercase()
}

/// Parse series ID from FRED response to domain Symbol
///
/// FRED series IDs become the "base" field, with empty "quote"
pub fn _parse_series_id(series_id: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(series_id, "")
}
