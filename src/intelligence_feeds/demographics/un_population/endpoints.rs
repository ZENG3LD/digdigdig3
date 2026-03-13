//! UN Population API endpoints

/// Base URLs for UN Population API
pub struct UnPopEndpoints {
    pub rest_base: &'static str,
    pub testnet_base: Option<&'static str>,
}

impl UnPopEndpoints {
    pub fn new(_testnet: bool) -> Self {
        Self {
            rest_base: "https://population.un.org/dataportalapi/api/v1",
            testnet_base: None,
        }
    }

    pub fn url(&self, endpoint: &UnPopEndpoint) -> String {
        format!("{}{}", self.rest_base, endpoint.path())
    }
}

impl Default for UnPopEndpoints {
    fn default() -> Self {
        Self::new(false)
    }
}

/// UN Population API endpoint enum
#[derive(Debug, Clone)]
pub enum UnPopEndpoint {
    /// Get all locations (countries and regions)
    Locations,
    /// Get indicator data for a specific location (legacy metadata endpoint)
    LocationIndicatorData { location_id: u32, indicator_id: u32 },
    /// Get all available indicators
    Indicators,
    /// Get details for a specific indicator
    IndicatorDetails { id: u32 },
    /// Core data endpoint: fetch actual time-series values for an indicator and location
    ///
    /// Path: `/data/indicators/{indicator_code}/locations/{location_code}`
    ///
    /// This is the primary endpoint for retrieving demographic data values.
    /// Use indicator codes (e.g., "49" for total population, "19" for fertility rate)
    /// and location codes (e.g., "900" for World, "840" for USA).
    DataByIndicatorLocation {
        indicator_code: String,
        location_code: String,
    },
}

impl UnPopEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Locations => "/locations".to_string(),
            Self::LocationIndicatorData { location_id, indicator_id } => {
                format!("/locations/{}/indicators/{}", location_id, indicator_id)
            }
            Self::Indicators => "/indicators".to_string(),
            Self::IndicatorDetails { id } => format!("/indicators/{}", id),
            Self::DataByIndicatorLocation { indicator_code, location_code } => {
                format!("/data/indicators/{}/locations/{}", indicator_code, location_code)
            }
        }
    }
}

/// Format query parameters for UN Population API
///
/// # Arguments
/// - `start_year` - Optional start year
/// - `end_year` - Optional end year
/// - `page_size` - Optional page size (default 100)
/// - `page_number` - Optional page number (default 1)
pub fn format_params(
    start_year: Option<u32>,
    end_year: Option<u32>,
    page_size: Option<u32>,
    page_number: Option<u32>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();

    if let Some(start) = start_year {
        params.push(("startYear".to_string(), start.to_string()));
    }
    if let Some(end) = end_year {
        params.push(("endYear".to_string(), end.to_string()));
    }

    params.push(("pageSize".to_string(), page_size.unwrap_or(100).to_string()));
    params.push(("pageNumber".to_string(), page_number.unwrap_or(1).to_string()));

    params
}
