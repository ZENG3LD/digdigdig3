//! INTERPOL connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    InterpolParser, InterpolNotice, InterpolSearchResult, InterpolImage,
};

/// INTERPOL connector
///
/// Provides access to INTERPOL Red Notices (wanted persons), Yellow Notices
/// (missing persons), and UN Security Council special notices.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::interpol::InterpolConnector;
///
/// let connector = InterpolConnector::from_env();
///
/// // Search for red notices
/// let results = connector.search_red_notices(
///     Some("John"),
///     Some("US"),
///     Some(30),
///     Some(50),
///     Some(1)
/// ).await?;
///
/// // Get individual red notice
/// let notice = connector.get_red_notice("2023/12345").await?;
///
/// // Get images for a notice
/// let images = connector.get_red_notice_images("2023/12345").await?;
/// ```
pub struct InterpolConnector {
    client: Client,
    _auth: InterpolAuth,
    endpoints: InterpolEndpoints,
}

impl InterpolConnector {
    /// Create new INTERPOL connector
    pub fn new(auth: InterpolAuth) -> Self {
        Self {
            client: Client::new(),
            _auth: auth,
            endpoints: InterpolEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// No environment variables needed - INTERPOL API is public
    pub fn from_env() -> Self {
        Self::new(InterpolAuth::from_env())
    }

    /// Internal: Make GET request to INTERPOL API
    async fn get(
        &self,
        endpoint: InterpolEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors
        InterpolParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RED NOTICES (WANTED PERSONS)
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for red notices (wanted persons)
    ///
    /// # Arguments
    /// - `name` - Optional last name search
    /// - `forename` - Optional first name search
    /// - `nationality` - Optional nationality (ISO 3166-1 alpha-2 code)
    /// - `age_min` - Optional minimum age
    /// - `age_max` - Optional maximum age
    /// - `sex_id` - Optional sex ID ("M" or "F")
    /// - `arrest_warrant_country_id` - Optional arrest warrant issuing country
    /// - `page` - Optional page number (default 1)
    /// - `result_per_page` - Optional results per page (default 20, max 160)
    ///
    /// # Returns
    /// Search results with total count and notices
    #[allow(clippy::too_many_arguments)]
    pub async fn search_red_notices(
        &self,
        name: Option<&str>,
        forename: Option<&str>,
        nationality: Option<&str>,
        age_min: Option<u32>,
        age_max: Option<u32>,
        sex_id: Option<&str>,
        arrest_warrant_country_id: Option<&str>,
        page: Option<u32>,
        result_per_page: Option<u32>,
    ) -> ExchangeResult<InterpolSearchResult> {
        let mut params = HashMap::new();

        if let Some(n) = name {
            params.insert("name".to_string(), n.to_string());
        }
        if let Some(f) = forename {
            params.insert("forename".to_string(), f.to_string());
        }
        if let Some(nat) = nationality {
            params.insert("nationality".to_string(), nat.to_string());
        }
        if let Some(min) = age_min {
            params.insert("ageMin".to_string(), min.to_string());
        }
        if let Some(max) = age_max {
            params.insert("ageMax".to_string(), max.to_string());
        }
        if let Some(sex) = sex_id {
            params.insert("sexId".to_string(), sex.to_string());
        }
        if let Some(country) = arrest_warrant_country_id {
            params.insert("arrestWarrantCountryId".to_string(), country.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(rpp) = result_per_page {
            params.insert("resultPerPage".to_string(), rpp.to_string());
        }

        let response = self.get(InterpolEndpoint::RedNotices, params).await?;
        InterpolParser::parse_search_result(&response)
    }

    /// Get individual red notice details
    ///
    /// # Arguments
    /// - `notice_id` - Notice ID (e.g., "2023/12345")
    ///
    /// # Returns
    /// Full notice details
    pub async fn get_red_notice(&self, notice_id: &str) -> ExchangeResult<InterpolNotice> {
        let params = HashMap::new();
        let response = self.get(
            InterpolEndpoint::RedNoticeDetail {
                notice_id: notice_id.to_string(),
            },
            params,
        )
        .await?;
        InterpolParser::parse_notice(&response)
    }

    /// Get images for a red notice
    ///
    /// # Arguments
    /// - `notice_id` - Notice ID (e.g., "2023/12345")
    ///
    /// # Returns
    /// List of image URLs
    pub async fn get_red_notice_images(&self, notice_id: &str) -> ExchangeResult<Vec<InterpolImage>> {
        let params = HashMap::new();
        let response = self.get(
            InterpolEndpoint::RedNoticeImages {
                notice_id: notice_id.to_string(),
            },
            params,
        )
        .await?;
        InterpolParser::parse_images(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // YELLOW NOTICES (MISSING PERSONS)
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for yellow notices (missing persons)
    ///
    /// # Arguments
    /// - `name` - Optional last name search
    /// - `forename` - Optional first name search
    /// - `nationality` - Optional nationality (ISO 3166-1 alpha-2 code)
    /// - `age_min` - Optional minimum age
    /// - `age_max` - Optional maximum age
    /// - `sex_id` - Optional sex ID ("M" or "F")
    /// - `page` - Optional page number (default 1)
    /// - `result_per_page` - Optional results per page (default 20, max 160)
    ///
    /// # Returns
    /// Search results with total count and notices
    #[allow(clippy::too_many_arguments)]
    pub async fn search_yellow_notices(
        &self,
        name: Option<&str>,
        forename: Option<&str>,
        nationality: Option<&str>,
        age_min: Option<u32>,
        age_max: Option<u32>,
        sex_id: Option<&str>,
        page: Option<u32>,
        result_per_page: Option<u32>,
    ) -> ExchangeResult<InterpolSearchResult> {
        let mut params = HashMap::new();

        if let Some(n) = name {
            params.insert("name".to_string(), n.to_string());
        }
        if let Some(f) = forename {
            params.insert("forename".to_string(), f.to_string());
        }
        if let Some(nat) = nationality {
            params.insert("nationality".to_string(), nat.to_string());
        }
        if let Some(min) = age_min {
            params.insert("ageMin".to_string(), min.to_string());
        }
        if let Some(max) = age_max {
            params.insert("ageMax".to_string(), max.to_string());
        }
        if let Some(sex) = sex_id {
            params.insert("sexId".to_string(), sex.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(rpp) = result_per_page {
            params.insert("resultPerPage".to_string(), rpp.to_string());
        }

        let response = self.get(InterpolEndpoint::YellowNotices, params).await?;
        InterpolParser::parse_search_result(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UN SECURITY COUNCIL NOTICES
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for UN Security Council special notices
    ///
    /// # Arguments
    /// - `name` - Optional last name search
    /// - `forename` - Optional first name search
    /// - `nationality` - Optional nationality (ISO 3166-1 alpha-2 code)
    /// - `page` - Optional page number (default 1)
    /// - `result_per_page` - Optional results per page (default 20, max 160)
    ///
    /// # Returns
    /// Search results with total count and notices
    pub async fn search_un_notices(
        &self,
        name: Option<&str>,
        forename: Option<&str>,
        nationality: Option<&str>,
        page: Option<u32>,
        result_per_page: Option<u32>,
    ) -> ExchangeResult<InterpolSearchResult> {
        let mut params = HashMap::new();

        if let Some(n) = name {
            params.insert("name".to_string(), n.to_string());
        }
        if let Some(f) = forename {
            params.insert("forename".to_string(), f.to_string());
        }
        if let Some(nat) = nationality {
            params.insert("nationality".to_string(), nat.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(rpp) = result_per_page {
            params.insert("resultPerPage".to_string(), rpp.to_string());
        }

        let response = self.get(InterpolEndpoint::UnNotices, params).await?;
        InterpolParser::parse_search_result(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search red notices by name only
    ///
    /// Convenience method for simple name search.
    ///
    /// # Arguments
    /// - `name` - Last name to search
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results
    pub async fn search_by_name(
        &self,
        name: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<InterpolSearchResult> {
        self.search_red_notices(
            Some(name),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            limit,
        )
        .await
    }

    /// Search red notices by nationality
    ///
    /// Convenience method to find all wanted persons of a specific nationality.
    ///
    /// # Arguments
    /// - `nationality` - ISO 3166-1 alpha-2 country code (e.g., "US", "RU")
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results for specified nationality
    pub async fn search_by_nationality(
        &self,
        nationality: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<InterpolSearchResult> {
        self.search_red_notices(
            None,
            None,
            Some(nationality),
            None,
            None,
            None,
            None,
            None,
            limit,
        )
        .await
    }

    /// Search red notices by age range
    ///
    /// Convenience method to find wanted persons in a specific age range.
    ///
    /// # Arguments
    /// - `age_min` - Minimum age
    /// - `age_max` - Maximum age
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results in specified age range
    pub async fn search_by_age_range(
        &self,
        age_min: u32,
        age_max: u32,
        limit: Option<u32>,
    ) -> ExchangeResult<InterpolSearchResult> {
        self.search_red_notices(
            None,
            None,
            None,
            Some(age_min),
            Some(age_max),
            None,
            None,
            None,
            limit,
        )
        .await
    }
}
