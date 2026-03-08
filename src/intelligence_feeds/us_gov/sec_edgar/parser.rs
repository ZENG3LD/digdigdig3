//! SEC EDGAR response parsers
//!
//! Parse JSON responses to domain types based on SEC EDGAR API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct SecEdgarParser;

impl SecEdgarParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ERROR CHECKING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for SEC EDGAR API errors
    ///
    /// SEC EDGAR returns HTTP error codes rather than JSON error objects
    pub fn check_error(json: &Value) -> ExchangeResult<()> {
        // SEC EDGAR doesn't return structured error objects
        // Errors are handled via HTTP status codes
        if let Some(error) = json.get("error") {
            return Err(ExchangeError::Api {
                code: -1,
                message: error.as_str().unwrap_or("Unknown error").to_string(),
            });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Extract required string field
    pub fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing field: {}", field)))
    }

    /// Extract optional string field
    pub fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    /// Extract required u64 field
    pub fn require_u64(obj: &Value, field: &str) -> ExchangeResult<u64> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing field: {}", field)))
    }

    /// Extract optional u64 field
    pub fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    /// Extract required f64 field
    pub fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing field: {}", field)))
    }

    /// Extract optional f64 field
    pub fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMPANY DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse company filings response
    ///
    /// Returns the full JSON as CompanyFiling struct
    pub fn parse_company_filings(response: &Value) -> ExchangeResult<CompanyFiling> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse company filings: {}", e)))
    }

    /// Parse company facts response
    ///
    /// Returns XBRL financial data
    pub fn parse_company_facts(response: &Value) -> ExchangeResult<CompanyFacts> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse company facts: {}", e)))
    }

    /// Parse company concept response
    ///
    /// Returns data for a specific financial concept (e.g., Revenues)
    pub fn parse_company_concept(response: &Value) -> ExchangeResult<CompanyConcept> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse company concept: {}", e)))
    }

    /// Parse XBRL frames response
    ///
    /// Returns aggregate data across all filers
    pub fn parse_xbrl_frames(response: &Value) -> ExchangeResult<XbrlFrame> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse XBRL frames: {}", e)))
    }

    /// Parse company tickers response
    ///
    /// Returns list of all companies with CIKs
    pub fn parse_company_tickers(response: &Value) -> ExchangeResult<Vec<CompanyTicker>> {
        // Response format: {"0": {...}, "1": {...}, ...}
        // Each object has: cik_str, ticker, title
        let mut tickers = Vec::new();

        if let Some(obj) = response.as_object() {
            for (_, value) in obj {
                if let Ok(ticker) = serde_json::from_value::<CompanyTicker>(value.clone()) {
                    tickers.push(ticker);
                }
            }
        }

        Ok(tickers)
    }

    /// Parse search results
    pub fn parse_search_results(response: &Value) -> ExchangeResult<Vec<SearchResult>> {
        let hits = response
            .get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing search results".to_string()))?;

        hits.iter()
            .map(|hit| {
                let source = hit
                    .get("_source")
                    .ok_or_else(|| ExchangeError::Parse("Missing _source field".to_string()))?;

                Ok(SearchResult {
                    cik: Self::get_str(source, "cik").map(|s| s.to_string()),
                    company_name: Self::get_str(source, "display_names").map(|s| s.to_string()),
                    form: Self::get_str(source, "form").map(|s| s.to_string()),
                    filing_date: Self::get_str(source, "file_date").map(|s| s.to_string()),
                    accession_number: Self::get_str(source, "adsh").map(|s| s.to_string()),
                })
            })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Company filing metadata (simplified - full structure is very large)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyFiling {
    pub cik: String,
    pub entity_type: Option<String>,
    pub sic: Option<String>,
    pub sic_description: Option<String>,
    pub name: Option<String>,
    pub tickers: Option<Vec<String>>,
    pub exchanges: Option<Vec<String>>,
    pub filings: Option<Value>, // Complex nested structure
}

/// XBRL company facts (simplified - full structure is very nested)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFacts {
    pub cik: u64,
    #[serde(rename = "entityName")]
    pub entity_name: String,
    pub facts: Option<Value>, // Complex nested structure by taxonomy
}

/// Company concept for a specific financial metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConcept {
    pub cik: u64,
    #[serde(rename = "entityName")]
    pub entity_name: String,
    pub taxonomy: String,
    pub tag: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub units: Option<Value>, // Contains time series data by unit
}

/// XBRL frame (aggregate data across all filers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XbrlFrame {
    pub taxonomy: String,
    pub tag: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub data: Option<Vec<Value>>, // Array of company data points
}

/// Company ticker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyTicker {
    pub cik_str: String,
    pub ticker: String,
    pub title: String,
}

/// Filing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilingMetadata {
    pub accession_number: String,
    pub filing_date: String,
    pub report_date: Option<String>,
    pub form: String,
    pub file_number: Option<String>,
    pub film_number: Option<String>,
    pub items: Option<String>,
    pub size: Option<u64>,
    pub is_xbrl: Option<bool>,
    pub is_inline_xbrl: Option<bool>,
}

/// Financial fact from XBRL data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialFact {
    pub end: String, // End date (YYYY-MM-DD)
    pub val: f64,    // Value
    pub accn: Option<String>, // Accession number
    pub fy: Option<u32>, // Fiscal year
    pub fp: Option<String>, // Fiscal period (Q1, Q2, Q3, Q4, FY)
    pub form: Option<String>, // Form type (10-K, 10-Q)
    pub filed: Option<String>, // Filed date
    pub frame: Option<String>, // Frame identifier
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub cik: Option<String>,
    pub company_name: Option<String>,
    pub form: Option<String>,
    pub filing_date: Option<String>,
    pub accession_number: Option<String>,
}
