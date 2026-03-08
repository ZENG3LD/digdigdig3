//! US Census Bureau API endpoints

/// Base URLs for Census API
pub struct CensusEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CensusEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.census.gov/data",
            ws_base: None, // Census does not support WebSocket
        }
    }
}

/// Census API endpoint enum
#[derive(Debug, Clone)]
pub enum CensusEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // CORE DATA QUERY
    // ═══════════════════════════════════════════════════════════════════════
    /// Generic dataset query: /{year}/{dataset}
    Dataset { year: String, dataset: String },

    // ═══════════════════════════════════════════════════════════════════════
    // ECONOMIC INDICATORS TIME SERIES (EITS)
    // ═══════════════════════════════════════════════════════════════════════
    /// Economic Indicators Time Series: /timeseries/eits/{indicator}
    EconomicIndicator { indicator: String },

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA / DISCOVERY
    // ═══════════════════════════════════════════════════════════════════════
    /// List all available datasets: /{year}.json
    ListDatasets { year: String },

    /// List all available datasets (current year): .json
    ListDatasetsAll,
}

impl CensusEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Dataset { year, dataset } => format!("/{}/{}", year, dataset),
            Self::EconomicIndicator { indicator } => format!("/timeseries/eits/{}", indicator),
            Self::ListDatasets { year } => format!("/{}.json", year),
            Self::ListDatasetsAll => ".json".to_string(),
        }
    }
}

/// Economic Indicator dataset IDs (for convenience)
pub mod indicators {
    /// Advance Monthly Sales for Retail and Food Services
    pub const RETAIL_SALES: &str = "advm";

    /// Manufacturers' Shipments, Inventories, and Orders (M3)
    pub const MANUFACTURERS_SHIPMENTS: &str = "mhs";

    /// Manufacturing and Trade Inventories and Sales
    pub const TRADE_INVENTORIES: &str = "mtis";

    /// New Residential Sales
    pub const NEW_HOME_SALES: &str = "ressales";

    /// New Residential Construction (Housing Starts)
    pub const HOUSING_STARTS: &str = "resconst";

    /// U.S. International Trade in Goods and Services
    pub const FOREIGN_TRADE: &str = "ftd";

    /// Quarterly Financial Report
    pub const QUARTERLY_FINANCIAL: &str = "qfr";

    /// Value of Construction Put in Place
    pub const CONSTRUCTION_SPENDING: &str = "vip";
}

/// Common dataset paths
pub mod datasets {
    /// American Community Survey 1-Year Data
    pub const ACS1: &str = "acs/acs1";

    /// American Community Survey 5-Year Data
    pub const ACS5: &str = "acs/acs5";

    /// Decennial Census
    pub const DECENNIAL: &str = "dec/sf1";

    /// Population Estimates Program
    pub const PEP: &str = "pep/population";
}

/// Format geography parameter for Census API
///
/// Examples:
/// - "state:01" - Alabama
/// - "state:*" - All states
/// - "us:*" - United States
/// - "county:*" - All counties
pub fn format_geography(geo_type: &str, code: &str) -> String {
    format!("{}:{}", geo_type, code)
}

/// Parse Census response array to extract column values
///
/// Census returns data as array of arrays:
/// [["NAME","B01001_001E","state"],["Alabama","5024279","01"]]
///
/// First row is header, subsequent rows are data
pub fn parse_census_response(
    data: &serde_json::Value,
) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let array = data
        .as_array()
        .ok_or_else(|| "Response is not an array".to_string())?;

    if array.is_empty() {
        return Err("Empty response array".to_string());
    }

    // First row is header
    let header_row = &array[0];
    let headers: Vec<String> = header_row
        .as_array()
        .ok_or_else(|| "Header row is not an array".to_string())?
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();

    // Remaining rows are data
    let mut rows = Vec::new();
    for row_value in array.iter().skip(1) {
        let row_array = row_value
            .as_array()
            .ok_or_else(|| "Data row is not an array".to_string())?;

        let row: Vec<String> = row_array
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();

        rows.push(row);
    }

    Ok((headers, rows))
}
