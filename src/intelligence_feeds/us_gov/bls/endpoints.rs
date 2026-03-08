//! BLS API endpoints

/// Base URLs for BLS API
pub struct BlsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for BlsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.bls.gov/publicAPI/v2",
            ws_base: None, // BLS does not support WebSocket
        }
    }
}

/// BLS API endpoint enum
#[derive(Debug, Clone)]
pub enum BlsEndpoint {
    /// Get time series data (POST)
    TimeSeriesData,
    /// Get latest numbers (GET)
    LatestNumbers,
}

impl BlsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::TimeSeriesData => "/timeseries/data/",
            Self::LatestNumbers => "/timeseries/data/",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// POPULAR BLS SERIES IDS
// ═══════════════════════════════════════════════════════════════════════

/// Consumer Price Index - All Urban Consumers (CPI-U)
pub const CPI_ALL_URBAN: &str = "CUSR0000SA0";

/// Unemployment Rate
pub const UNEMPLOYMENT_RATE: &str = "LNS14000000";

/// Total Nonfarm Payroll Employment
pub const NONFARM_PAYROLLS: &str = "CES0000000001";

/// Average Hourly Earnings of All Employees, Total Private
pub const AVG_HOURLY_EARNINGS: &str = "CES0500000003";

/// Producer Price Index - Finished Goods
pub const PPI_FINISHED_GOODS: &str = "WPSFD4";

/// CPI - Energy
pub const CPI_ENERGY: &str = "CUSR0000SA0E";

/// CPI - Food and Beverages
pub const CPI_FOOD: &str = "CUSR0000SAF1";

/// Employment Cost Index - Total Compensation - All Workers
pub const EMPLOYMENT_COST_INDEX: &str = "CIU1010000000000A";

/// Productivity - Nonfarm Business Sector
pub const PRODUCTIVITY: &str = "PRS85006092";

/// Import Price Index - All Commodities
pub const IMPORT_PRICES: &str = "EIUIR";

/// Export Price Index - All Commodities
pub const EXPORT_PRICES: &str = "EIUIQ";

/// Job Openings: Total Nonfarm (JOLTS)
pub const JOLTS_JOB_OPENINGS: &str = "JTS000000000000000JOL";

/// Format series ID for BLS API
///
/// BLS uses series IDs like "CUSR0000SA0", "LNS14000000"
/// Similar to FRED - there's no base/quote concept.
/// Series IDs are unique identifiers in the BLS database.
///
/// For compatibility with the Symbol type, we'll use:
/// - base = series_id
/// - quote = "" (empty)
pub fn format_series_id(symbol: &crate::core::types::Symbol) -> String {
    // For BLS, the "base" field contains the series ID
    symbol.base.to_uppercase()
}

/// Parse series ID from BLS response to domain Symbol
///
/// BLS series IDs become the "base" field, with empty "quote"
pub fn parse_series_id(series_id: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(series_id, "")
}
