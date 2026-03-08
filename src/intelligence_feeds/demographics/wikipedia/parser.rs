//! Wikipedia Pageviews response parsers
//!
//! Parse JSON responses to domain types based on Wikipedia Pageviews API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct WikipediaParser;

impl WikipediaParser {
    // ═══════════════════════════════════════════════════════════════════════
    // WIKIPEDIA-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse per-article pageviews response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [
    ///     {
    ///       "project": "en.wikipedia",
    ///       "article": "Bitcoin",
    ///       "granularity": "daily",
    ///       "timestamp": "2024010100",
    ///       "access": "all-access",
    ///       "agent": "all-agents",
    ///       "views": 12345
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_pageviews(response: &Value) -> ExchangeResult<Vec<PageviewsEntry>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|item| {
                Ok(PageviewsEntry {
                    project: Self::require_str(item, "project")?.to_string(),
                    article: Self::get_str(item, "article").map(|s| s.to_string()),
                    granularity: Self::require_str(item, "granularity")?.to_string(),
                    timestamp: Self::require_str(item, "timestamp")?.to_string(),
                    access: Self::require_str(item, "access")?.to_string(),
                    agent: Self::require_str(item, "agent")?.to_string(),
                    views: Self::require_u64(item, "views")?,
                })
            })
            .collect()
    }

    /// Parse top articles response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [
    ///     {
    ///       "project": "en.wikipedia",
    ///       "access": "all-access",
    ///       "year": "2024",
    ///       "month": "01",
    ///       "day": "15",
    ///       "articles": [
    ///         {
    ///           "article": "Main_Page",
    ///           "views": 8000000,
    ///           "rank": 1
    ///         }
    ///       ]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_top_articles(response: &Value) -> ExchangeResult<Vec<TopArticle>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        let first_item = items
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'items' array".to_string()))?;

        let articles = first_item
            .get("articles")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'articles' array".to_string()))?;

        articles
            .iter()
            .map(|art| {
                Ok(TopArticle {
                    article: Self::require_str(art, "article")?.to_string(),
                    views: Self::require_u64(art, "views")?,
                    rank: Self::require_u32(art, "rank")?,
                })
            })
            .collect()
    }

    /// Parse top-per-country response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [
    ///     {
    ///       "countries": [
    ///         {
    ///           "country": "US",
    ///           "views": 1000000,
    ///           "rank": 1
    ///         }
    ///       ]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_top_by_country(response: &Value) -> ExchangeResult<Vec<TopCountry>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        let first_item = items
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'items' array".to_string()))?;

        let countries = first_item
            .get("countries")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'countries' array".to_string()))?;

        countries
            .iter()
            .map(|country| {
                Ok(TopCountry {
                    country: Self::require_str(country, "country")?.to_string(),
                    views: Self::require_u64(country, "views")?,
                    rank: Self::require_u32(country, "rank")?,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error_type) = response.get("type").and_then(|v| v.as_str()) {
            if error_type.contains("error") {
                let message = response
                    .get("detail")
                    .or_else(|| response.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return Err(ExchangeError::Api {
                    code: 0,
                    message,
                });
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn require_u64(obj: &Value, field: &str) -> ExchangeResult<u64> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WIKIPEDIA-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Wikipedia pageviews entry (single data point)
#[derive(Debug, Clone)]
pub struct PageviewsEntry {
    pub project: String,           // e.g., "en.wikipedia"
    pub article: Option<String>,   // e.g., "Bitcoin" (None for aggregate)
    pub granularity: String,       // "daily" or "monthly"
    pub timestamp: String,         // YYYYMMDD or YYYYMMDDHH
    pub access: String,            // "all-access", "desktop", "mobile-app", "mobile-web"
    pub agent: String,             // "all-agents", "user", "spider", "automated"
    pub views: u64,
}

/// Top article entry
#[derive(Debug, Clone)]
pub struct TopArticle {
    pub article: String,
    pub views: u64,
    pub rank: u32,
}

/// Top country entry
#[derive(Debug, Clone)]
pub struct TopCountry {
    pub country: String,  // Country code (e.g., "US", "GB")
    pub views: u64,
    pub rank: u32,
}
