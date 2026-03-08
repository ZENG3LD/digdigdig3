//! EU TED response parsers
//!
//! Parse JSON responses to domain types based on EU TED API response formats.

use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct EuTedParser;

impl EuTedParser {
    /// Parse notice search results
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "total": 100,
    ///   "page": 1,
    ///   "limit": 20,
    ///   "results": [
    ///     {
    ///       "noticeId": "...",
    ///       "title": "...",
    ///       "publicationDate": "...",
    ///       "submissionDeadline": "...",
    ///       "buyerName": "...",
    ///       "buyerCountry": "...",
    ///       "totalValue": 1000000,
    ///       "currency": "EUR",
    ///       "cpvCodes": ["45000000"],
    ///       "noticeType": "...",
    ///       "status": "...",
    ///       "description": "...",
    ///       "lotsCount": 1
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_search_results(response: &Value) -> ExchangeResult<TedSearchResult> {
        let total = response
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let page = response
            .get("page")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        let limit = response
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as u32;

        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let notices = results
            .iter()
            .map(Self::parse_notice)
            .collect::<ExchangeResult<Vec<TedNotice>>>()?;

        Ok(TedSearchResult {
            total,
            page,
            limit,
            notices,
        })
    }

    /// Parse single notice
    pub fn parse_notice(notice: &Value) -> ExchangeResult<TedNotice> {
        Ok(TedNotice {
            notice_id: Self::get_str(notice, "noticeId").map(|s| s.to_string()),
            title: Self::get_str(notice, "title").map(|s| s.to_string()),
            publication_date: Self::get_str(notice, "publicationDate").map(|s| s.to_string()),
            submission_deadline: Self::get_str(notice, "submissionDeadline").map(|s| s.to_string()),
            buyer_name: Self::get_str(notice, "buyerName").map(|s| s.to_string()),
            buyer_country: Self::get_str(notice, "buyerCountry").map(|s| s.to_string()),
            total_value: notice
                .get("totalValue")
                .and_then(|v| v.as_f64()),
            currency: Self::get_str(notice, "currency").map(|s| s.to_string()),
            cpv_codes: Self::parse_string_array(notice.get("cpvCodes")),
            notice_type: Self::get_str(notice, "noticeType").map(|s| s.to_string()),
            status: Self::get_str(notice, "status").map(|s| s.to_string()),
            description: Self::get_str(notice, "description").map(|s| s.to_string()),
            lots_count: notice
                .get("lotsCount")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        })
    }

    /// Parse entity search results
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "results": [
    ///     {
    ///       "entityId": "...",
    ///       "name": "...",
    ///       "country": "...",
    ///       "address": "...",
    ///       "type": "...",
    ///       "contractsCount": 10
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_entities(response: &Value) -> ExchangeResult<Vec<TedEntity>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(Self::parse_entity)
            .collect()
    }

    /// Parse single entity
    pub fn parse_entity(entity: &Value) -> ExchangeResult<TedEntity> {
        Ok(TedEntity {
            entity_id: Self::get_str(entity, "entityId").map(|s| s.to_string()),
            name: Self::get_str(entity, "name").map(|s| s.to_string()),
            country: Self::get_str(entity, "country").map(|s| s.to_string()),
            address: Self::get_str(entity, "address").map(|s| s.to_string()),
            type_name: Self::get_str(entity, "type").map(|s| s.to_string()),
            contracts_count: entity
                .get("contractsCount")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        })
    }

    /// Parse string array from JSON
    fn parse_string_array(arr_value: Option<&Value>) -> Vec<String> {
        arr_value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error_code) = response.get("error_code") {
            let code = error_code.as_i64().unwrap_or(0) as i32;
            let message = response
                .get("error_message")
                .or_else(|| response.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api { code, message });
        }

        // Check for other error formats
        if let Some(error) = response.get("error") {
            let message = error.as_str().unwrap_or("Unknown error").to_string();
            return Err(ExchangeError::Api { code: 0, message });
        }

        Ok(())
    }

    // Helper methods
    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EU TED-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// EU TED procurement notice
#[derive(Debug, Clone)]
pub struct TedNotice {
    pub notice_id: Option<String>,
    pub title: Option<String>,
    pub publication_date: Option<String>,
    pub submission_deadline: Option<String>,
    pub buyer_name: Option<String>,
    pub buyer_country: Option<String>,
    pub total_value: Option<f64>,
    pub currency: Option<String>,
    pub cpv_codes: Vec<String>,
    pub notice_type: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub lots_count: Option<u32>,
}

/// EU TED business entity (contracting authority or economic operator)
#[derive(Debug, Clone)]
pub struct TedEntity {
    pub entity_id: Option<String>,
    pub name: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub type_name: Option<String>,
    pub contracts_count: Option<u32>,
}

/// EU TED search result
#[derive(Debug, Clone)]
pub struct TedSearchResult {
    pub total: u32,
    pub page: u32,
    pub limit: u32,
    pub notices: Vec<TedNotice>,
}
