//! NOAA Climate Data Online API v2 endpoints

/// Base URLs for NOAA CDO API
pub struct NoaaEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for NoaaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.ncei.noaa.gov/cdo-web/api/v2",
            ws_base: None, // NOAA CDO does not support WebSocket
        }
    }
}

/// NOAA CDO API endpoint enum
#[derive(Debug, Clone)]
pub enum NoaaEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get climate data observations
    Data,

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS - Datasets
    // ═══════════════════════════════════════════════════════════════════════
    /// List all available datasets
    Datasets,
    /// Get a specific dataset by ID
    Dataset,

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS - Datatypes
    // ═══════════════════════════════════════════════════════════════════════
    /// List available datatypes (temperature, precipitation, etc.)
    Datatypes,
    /// Get a specific datatype by ID
    Datatype,

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS - Locations
    // ═══════════════════════════════════════════════════════════════════════
    /// List location categories
    LocationCategories,
    /// List locations
    Locations,
    /// Get a specific location by ID
    Location,

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS - Stations
    // ═══════════════════════════════════════════════════════════════════════
    /// List weather stations
    Stations,
    /// Get a specific station by ID
    Station,
}

impl NoaaEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Data
            Self::Data => "/data",

            // Datasets
            Self::Datasets => "/datasets",
            Self::Dataset => "/datasets", // ID appended in connector

            // Datatypes
            Self::Datatypes => "/datatypes",
            Self::Datatype => "/datatypes", // ID appended in connector

            // Locations
            Self::LocationCategories => "/locationcategories",
            Self::Locations => "/locations",
            Self::Location => "/locations", // ID appended in connector

            // Stations
            Self::Stations => "/stations",
            Self::Station => "/stations", // ID appended in connector
        }
    }

    /// Build full path with ID (for specific resource endpoints)
    pub fn path_with_id(&self, id: &str) -> String {
        format!("{}/{}", self.path(), id)
    }
}

/// Format location ID for NOAA API
///
/// NOAA uses location IDs like "FIPS:37", "CITY:US370007", "ZIP:28801"
/// For compatibility with Symbol type, we use the base field for the location ID.
pub fn _format_location_id(symbol: &crate::core::types::Symbol) -> String {
    symbol.base.to_uppercase()
}

/// Parse location ID from NOAA response to domain Symbol
pub fn _parse_location_id(location_id: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(location_id, "")
}
