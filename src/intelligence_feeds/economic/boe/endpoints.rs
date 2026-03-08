//! Bank of England API endpoints

/// Base URLs for BoE API
pub struct BoeEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for BoeEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.bankofengland.co.uk/boeapps/database",
            ws_base: None, // BoE does not support WebSocket
        }
    }
}

/// Bank of England API endpoint enum
#[derive(Debug, Clone)]
pub enum BoeEndpoint {
    /// Get CSV data for one or more series codes
    /// URL pattern: /_iadb-fromshowcolumns.asp?csv.x=yes&SeriesCodes={codes}&Datefrom={from}&Dateto={to}&CSVF=TN
    GetData,

    /// Get series information
    /// URL pattern: /fromshowcolumns.asp?SeriesCodes={code}&CSVF=TN
    GetSeriesInfo,
}

impl BoeEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::GetData => "/_iadb-fromshowcolumns.asp",
            Self::GetSeriesInfo => "/fromshowcolumns.asp",
        }
    }
}

/// Format series code for BoE API
///
/// BoE uses series codes like "IUDBEDR", "LPMAUZI", "D7BT"
/// This is similar to FRED - there's no base/quote concept.
/// Series codes are unique identifiers in the BoE database.
///
/// For compatibility with the Symbol type:
/// - base = series_code
/// - quote = "" (empty)
pub fn _format_series_code(symbol: &crate::core::types::Symbol) -> String {
    // For BoE, the "base" field contains the series code
    symbol.base.to_uppercase()
}

/// Parse series code from BoE response to domain Symbol
///
/// BoE series codes become the "base" field, with empty "quote"
pub fn _parse_series_code(series_code: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(series_code, "")
}
