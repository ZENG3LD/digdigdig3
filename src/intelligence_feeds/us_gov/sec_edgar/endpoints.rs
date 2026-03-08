//! SEC EDGAR API endpoints

/// Base URLs for SEC EDGAR API
pub struct SecEdgarEndpoints {
    pub rest_base: &'static str,
    pub search_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SecEdgarEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://data.sec.gov",
            search_base: "https://efts.sec.gov/LATEST",
            ws_base: None, // SEC EDGAR does not support WebSocket
        }
    }
}

/// SEC EDGAR API endpoint enum
#[derive(Debug, Clone)]
pub enum SecEdgarEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // COMPANY DATA ENDPOINTS (data.sec.gov)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all filings for a company
    CompanyFilings { cik: String },
    /// Get XBRL financial data for a company
    CompanyFacts { cik: String },
    /// Get specific financial concept for a company
    CompanyConcept { cik: String, taxonomy: String, tag: String },

    // ═══════════════════════════════════════════════════════════════════════
    // FULL-TEXT SEARCH ENDPOINTS (efts.sec.gov)
    // ═══════════════════════════════════════════════════════════════════════
    /// Search filings by keywords and filters
    SearchFilings,

    // ═══════════════════════════════════════════════════════════════════════
    // BULK DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all companies with their CIKs
    CompanyTickers,
    /// Get all mutual funds with their CIKs
    MutualFundTickers,

    // ═══════════════════════════════════════════════════════════════════════
    // XBRL FRAMES (AGGREGATE DATA)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get aggregate data across all filers for a specific financial concept
    XbrlFrames { taxonomy: String, tag: String, unit: String, period: String },
}

impl SecEdgarEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Company data
            Self::CompanyFilings { cik } => format!("/submissions/CIK{}.json", pad_cik(cik)),
            Self::CompanyFacts { cik } => format!("/api/xbrl/companyfacts/CIK{}.json", pad_cik(cik)),
            Self::CompanyConcept { cik, taxonomy, tag } => {
                format!("/api/xbrl/companyconcept/CIK{}/{}/{}.json", pad_cik(cik), taxonomy, tag)
            }

            // Search
            Self::SearchFilings => "/search-index".to_string(),

            // Bulk data
            Self::CompanyTickers => "/files/company_tickers.json".to_string(),
            Self::MutualFundTickers => "/files/company_tickers_mf.json".to_string(),

            // XBRL frames
            Self::XbrlFrames { taxonomy, tag, unit, period } => {
                format!("/api/xbrl/frames/{}/{}/{}/CY{}.json", taxonomy, tag, unit, period)
            }
        }
    }

    /// Get base URL for this endpoint
    pub fn base_url(&self, endpoints: &SecEdgarEndpoints) -> &str {
        match self {
            Self::SearchFilings => endpoints.search_base,
            _ => endpoints.rest_base,
        }
    }
}

/// Pad CIK number to 10 digits with leading zeros
///
/// SEC EDGAR requires CIK numbers to be zero-padded to 10 digits.
/// Examples:
/// - "320193" -> "0000320193" (Apple)
/// - "1318605" -> "0001318605" (Tesla)
pub fn pad_cik(cik: &str) -> String {
    // Remove any existing leading zeros and whitespace
    let trimmed = cik.trim().trim_start_matches('0');
    
    // Pad to 10 digits
    format!("{:0>10}", trimmed)
}

/// Format CIK from Symbol
///
/// For SEC EDGAR, the "base" field contains the CIK number
pub fn _format_cik(symbol: &crate::core::types::Symbol) -> String {
    pad_cik(&symbol.base)
}

/// Parse CIK to Symbol
///
/// CIK numbers become the "base" field, with empty "quote"
pub fn _parse_cik(cik: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(pad_cik(cik), "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_cik() {
        assert_eq!(pad_cik("320193"), "0000320193"); // Apple
        assert_eq!(pad_cik("1318605"), "0001318605"); // Tesla
        assert_eq!(pad_cik("0000320193"), "0000320193"); // Already padded
        assert_eq!(pad_cik("789019"), "0000789019"); // Microsoft
    }
}
