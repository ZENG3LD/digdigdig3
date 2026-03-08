//! FRED connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{
    ExchangeError, ExchangeResult, Symbol, AccountType, ExchangeId,
    Kline, Ticker, OrderBook, Price, FundingRate, Position,
    Order, OrderSide, Quantity, Balance, AccountInfo, Asset,
};
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    FredParser, Observation, SeriesMetadata, Category, Release, Source, Tag,
    ReleaseDate, SeriesUpdate, VintageDate, FredReleaseTable, FredGeoSeriesGroup,
    FredGeoSeriesData, FredGeoRegionalData, FredGeoShapes,
};

/// FRED (Federal Reserve Economic Data) connector
///
/// Provides access to 840,000+ economic time series from the Federal Reserve Bank of St. Louis.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::fred::FredConnector;
///
/// let connector = FredConnector::from_env();
///
/// // Get economic series data (e.g., unemployment rate)
/// let observations = connector.get_series_observations("UNRATE", None, None, None).await?;
///
/// // Search for series
/// let results = connector.search_series("GDP").await?;
///
/// // Get series metadata
/// let metadata = connector.get_series_metadata("GNPCA").await?;
/// ```
pub struct FredConnector {
    client: Client,
    auth: FredAuth,
    endpoints: FredEndpoints,
    testnet: bool,
}

impl FredConnector {
    /// Create new FRED connector with authentication
    pub fn new(auth: FredAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: FredEndpoints::default(),
            testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `FRED_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(FredAuth::from_env())
    }

    /// Internal: Make GET request to FRED API
    async fn get(
        &self,
        endpoint: FredEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

        // Always request JSON format
        params.insert("file_type".to_string(), "json".to_string());

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
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

        // Check for FRED API errors
        FredParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FRED-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get observations (data values) for an economic data series
    ///
    /// This is the CORE endpoint for retrieving time series data.
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID (e.g., "GNPCA", "UNRATE", "GDP")
    /// - `observation_start` - Optional start date (YYYY-MM-DD)
    /// - `observation_end` - Optional end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-100000, default 100000)
    ///
    /// # Returns
    /// Vector of observations with date and value
    pub async fn get_series_observations(
        &self,
        series_id: &str,
        observation_start: Option<&str>,
        observation_end: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Observation>> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        if let Some(start) = observation_start {
            params.insert("observation_start".to_string(), start.to_string());
        }
        if let Some(end) = observation_end {
            params.insert("observation_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(FredEndpoint::SeriesObservations, params).await?;
        FredParser::parse_observations(&response)
    }

    /// Get metadata for an economic data series
    ///
    /// Returns information like title, frequency, units, observation dates, etc.
    pub async fn get_series_metadata(&self, series_id: &str) -> ExchangeResult<SeriesMetadata> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        let response = self.get(FredEndpoint::Series, params).await?;
        FredParser::parse_series(&response)
    }

    /// Search for economic data series matching keywords
    ///
    /// # Arguments
    /// - `search_text` - Keywords to search for
    /// - `limit` - Optional limit (1-1000, default 1000)
    ///
    /// # Returns
    /// Vector of series IDs matching the search
    pub async fn search_series(
        &self,
        search_text: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<String>> {
        let mut params = HashMap::new();
        params.insert("search_text".to_string(), search_text.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(FredEndpoint::SeriesSearch, params).await?;
        FredParser::parse_series_search(&response)
    }

    /// Get categories
    ///
    /// If `category_id` is None, returns root categories.
    pub async fn get_categories(&self, category_id: Option<i64>) -> ExchangeResult<Vec<Category>> {
        let mut params = HashMap::new();
        if let Some(id) = category_id {
            params.insert("category_id".to_string(), id.to_string());
        }

        let response = self.get(FredEndpoint::Category, params).await?;
        FredParser::parse_categories(&response)
    }

    /// Get child categories for a parent category
    pub async fn get_category_children(&self, category_id: i64) -> ExchangeResult<Vec<Category>> {
        let mut params = HashMap::new();
        params.insert("category_id".to_string(), category_id.to_string());

        let response = self.get(FredEndpoint::CategoryChildren, params).await?;
        FredParser::parse_categories(&response)
    }

    /// Get series in a category
    pub async fn get_category_series(
        &self,
        category_id: i64,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<String>> {
        let mut params = HashMap::new();
        params.insert("category_id".to_string(), category_id.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(FredEndpoint::CategorySeries, params).await?;
        FredParser::parse_series_search(&response) // Same format as search
    }

    /// Get all releases of economic data
    pub async fn get_releases(&self, limit: Option<u32>) -> ExchangeResult<Vec<Release>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(FredEndpoint::Releases, params).await?;
        FredParser::parse_releases(&response)
    }

    /// Get a specific release by ID
    pub async fn get_release(&self, release_id: i64) -> ExchangeResult<Vec<Release>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());

        let response = self.get(FredEndpoint::Release, params).await?;
        FredParser::parse_releases(&response)
    }

    /// Get all sources of economic data
    pub async fn get_sources(&self, limit: Option<u32>) -> ExchangeResult<Vec<Source>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(FredEndpoint::Sources, params).await?;
        FredParser::parse_sources(&response)
    }

    /// Get FRED tags
    pub async fn get_tags(&self, limit: Option<u32>) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(FredEndpoint::Tags, params).await?;
        FredParser::parse_tags(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CATEGORY ENDPOINTS (Extended)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get related categories for a category
    ///
    /// # Arguments
    /// - `category_id` - Category ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    pub async fn get_category_related(
        &self,
        category_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
    ) -> ExchangeResult<Vec<Category>> {
        let mut params = HashMap::new();
        params.insert("category_id".to_string(), category_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }

        let response = self.get(FredEndpoint::CategoryRelated, params).await?;
        FredParser::parse_categories(&response)
    }

    /// Get FRED tags for a category
    ///
    /// # Arguments
    /// - `category_id` - Category ID
    /// - `tag_names` - Optional semicolon-separated tag names to filter
    /// - `tag_group_id` - Optional tag group (freq, gen, geo, geot, rls, seas, src)
    /// - `search_text` - Optional search words
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field (series_count, popularity, created, name, group_id)
    /// - `sort_order` - Optional sort order (asc, desc)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_category_tags(
        &self,
        category_id: i64,
        tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("category_id".to_string(), category_id.to_string());

        if let Some(names) = tag_names {
            params.insert("tag_names".to_string(), names.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = search_text {
            params.insert("search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::CategoryTags, params).await?;
        FredParser::parse_tags(&response)
    }

    /// Get related FRED tags within a category
    ///
    /// # Arguments
    /// - `category_id` - Category ID
    /// - `tag_names` - Semicolon-separated tag names (required)
    /// - `exclude_tag_names` - Optional semicolon-separated tag names to exclude
    /// - `tag_group_id` - Optional tag group
    /// - `search_text` - Optional search words
    /// - `limit` - Optional limit (1-1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order
    #[allow(clippy::too_many_arguments)]
    pub async fn get_category_related_tags(
        &self,
        category_id: i64,
        tag_names: &str,
        exclude_tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("category_id".to_string(), category_id.to_string());
        params.insert("tag_names".to_string(), tag_names.to_string());

        if let Some(exclude) = exclude_tag_names {
            params.insert("exclude_tag_names".to_string(), exclude.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = search_text {
            params.insert("search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::CategoryRelatedTags, params).await?;
        FredParser::parse_tags(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RELEASE ENDPOINTS (Extended)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get release dates for all releases of economic data
    ///
    /// # Arguments
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field (release_date, release_id, release_name)
    /// - `sort_order` - Optional sort order (asc, desc)
    /// - `include_release_dates_with_no_data` - Optional include dates with no data
    #[allow(clippy::too_many_arguments)]
    pub async fn get_releases_dates(
        &self,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
        include_release_dates_with_no_data: Option<bool>,
    ) -> ExchangeResult<Vec<ReleaseDate>> {
        let mut params = HashMap::new();

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }
        if let Some(include) = include_release_dates_with_no_data {
            params.insert("include_release_dates_with_no_data".to_string(), include.to_string());
        }

        let response = self.get(FredEndpoint::ReleasesDates, params).await?;
        FredParser::parse_release_dates(&response)
    }

    /// Get release dates for a specific release
    ///
    /// # Arguments
    /// - `release_id` - Release ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-10000, default 10000)
    /// - `offset` - Optional pagination offset
    /// - `sort_order` - Optional sort order (asc, desc)
    /// - `include_release_dates_with_no_data` - Optional include dates with no data
    #[allow(clippy::too_many_arguments)]
    pub async fn get_release_dates(
        &self,
        release_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        sort_order: Option<&str>,
        include_release_dates_with_no_data: Option<bool>,
    ) -> ExchangeResult<Vec<ReleaseDate>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }
        if let Some(include) = include_release_dates_with_no_data {
            params.insert("include_release_dates_with_no_data".to_string(), include.to_string());
        }

        let response = self.get(FredEndpoint::ReleaseDates, params).await?;
        FredParser::parse_release_dates(&response)
    }

    /// Get economic data series in a release
    ///
    /// # Arguments
    /// - `release_id` - Release ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order (asc, desc)
    /// - `filter_variable` - Optional filter variable (frequency, units, seasonal_adjustment)
    /// - `filter_value` - Optional filter value
    /// - `tag_names` - Optional semicolon-separated tag names
    /// - `exclude_tag_names` - Optional semicolon-separated tag names to exclude
    #[allow(clippy::too_many_arguments)]
    pub async fn get_release_series(
        &self,
        release_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
        filter_variable: Option<&str>,
        filter_value: Option<&str>,
        tag_names: Option<&str>,
        exclude_tag_names: Option<&str>,
    ) -> ExchangeResult<Vec<String>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }
        if let Some(var) = filter_variable {
            params.insert("filter_variable".to_string(), var.to_string());
        }
        if let Some(val) = filter_value {
            params.insert("filter_value".to_string(), val.to_string());
        }
        if let Some(names) = tag_names {
            params.insert("tag_names".to_string(), names.to_string());
        }
        if let Some(exclude) = exclude_tag_names {
            params.insert("exclude_tag_names".to_string(), exclude.to_string());
        }

        let response = self.get(FredEndpoint::ReleaseSeries, params).await?;
        FredParser::parse_series_search(&response)
    }

    /// Get sources for a release
    ///
    /// # Arguments
    /// - `release_id` - Release ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    pub async fn get_release_sources(
        &self,
        release_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
    ) -> ExchangeResult<Vec<Source>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }

        let response = self.get(FredEndpoint::ReleaseSources, params).await?;
        FredParser::parse_sources(&response)
    }

    /// Get FRED tags for a release
    ///
    /// # Arguments
    /// - `release_id` - Release ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `tag_names` - Optional semicolon-separated tag names to filter
    /// - `tag_group_id` - Optional tag group (freq, gen, geo, geot, rls, seas, src)
    /// - `search_text` - Optional search words
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order (asc, desc)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_release_tags(
        &self,
        release_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(names) = tag_names {
            params.insert("tag_names".to_string(), names.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = search_text {
            params.insert("search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::ReleaseTags, params).await?;
        FredParser::parse_tags(&response)
    }

    /// Get related FRED tags for a release
    ///
    /// # Arguments
    /// - `release_id` - Release ID
    /// - `tag_names` - Semicolon-separated tag names (required)
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `exclude_tag_names` - Optional semicolon-separated tag names to exclude
    /// - `tag_group_id` - Optional tag group
    /// - `search_text` - Optional search words
    /// - `limit` - Optional limit (1-1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order
    #[allow(clippy::too_many_arguments)]
    pub async fn get_release_related_tags(
        &self,
        release_id: i64,
        tag_names: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        exclude_tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());
        params.insert("tag_names".to_string(), tag_names.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(exclude) = exclude_tag_names {
            params.insert("exclude_tag_names".to_string(), exclude.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = search_text {
            params.insert("search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::ReleaseRelatedTags, params).await?;
        FredParser::parse_tags(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SERIES ENDPOINTS (Extended)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get categories for an economic data series
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID (e.g., "GNPCA", "UNRATE")
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    pub async fn get_series_categories(
        &self,
        series_id: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
    ) -> ExchangeResult<Vec<Category>> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }

        let response = self.get(FredEndpoint::SeriesCategories, params).await?;
        FredParser::parse_categories(&response)
    }

    /// Get the release for an economic data series
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    pub async fn get_series_release(
        &self,
        series_id: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
    ) -> ExchangeResult<Vec<Release>> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }

        let response = self.get(FredEndpoint::SeriesRelease, params).await?;
        FredParser::parse_releases(&response)
    }

    /// Get FRED tags for a series search
    ///
    /// # Arguments
    /// - `series_search_text` - Keywords to search (required)
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `tag_names` - Optional semicolon-separated tag names to filter
    /// - `tag_group_id` - Optional tag group (freq, gen, geo, geot, rls, seas, src)
    /// - `tag_search_text` - Optional search tag names
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order (asc, desc)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_series_search_tags(
        &self,
        series_search_text: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        tag_search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("series_search_text".to_string(), series_search_text.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(names) = tag_names {
            params.insert("tag_names".to_string(), names.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = tag_search_text {
            params.insert("tag_search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::SeriesSearchTags, params).await?;
        FredParser::parse_tags(&response)
    }

    /// Get related FRED tags for a series search
    ///
    /// # Arguments
    /// - `series_search_text` - Keywords to search (required)
    /// - `tag_names` - Semicolon-separated tag names (required)
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `exclude_tag_names` - Optional semicolon-separated tag names to exclude
    /// - `tag_group_id` - Optional tag group
    /// - `tag_search_text` - Optional search tag names
    /// - `limit` - Optional limit (1-1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order
    #[allow(clippy::too_many_arguments)]
    pub async fn get_series_search_related_tags(
        &self,
        series_search_text: &str,
        tag_names: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        exclude_tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        tag_search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("series_search_text".to_string(), series_search_text.to_string());
        params.insert("tag_names".to_string(), tag_names.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(exclude) = exclude_tag_names {
            params.insert("exclude_tag_names".to_string(), exclude.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = tag_search_text {
            params.insert("tag_search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::SeriesSearchRelatedTags, params).await?;
        FredParser::parse_tags(&response)
    }

    /// Get FRED tags for a series
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID (required)
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `order_by` - Optional order field (series_count, popularity, created, name, group_id)
    /// - `sort_order` - Optional sort order (asc, desc)
    pub async fn get_series_tags(
        &self,
        series_id: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::SeriesTags, params).await?;
        FredParser::parse_tags(&response)
    }

    /// Get economic data series sorted by when observations were updated
    ///
    /// # Arguments
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `filter_value` - Optional filter (all, macro, regional)
    /// - `start_time` - Optional start time (YYYY-MM-DD HH:MM:SS)
    /// - `end_time` - Optional end time (YYYY-MM-DD HH:MM:SS)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_series_updates(
        &self,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        filter_value: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> ExchangeResult<Vec<SeriesUpdate>> {
        let mut params = HashMap::new();

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(filter) = filter_value {
            params.insert("filter_value".to_string(), filter.to_string());
        }
        if let Some(start) = start_time {
            params.insert("start_time".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("end_time".to_string(), end.to_string());
        }

        let response = self.get(FredEndpoint::SeriesUpdates, params).await?;
        FredParser::parse_series_updates(&response)
    }

    /// Get vintage dates for a series (ALFRED - revision history)
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID (required)
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-10000, default 10000)
    /// - `offset` - Optional pagination offset
    /// - `sort_order` - Optional sort order (asc, desc)
    pub async fn get_series_vintage_dates(
        &self,
        series_id: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<VintageDate>> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::SeriesVintageDates, params).await?;
        FredParser::parse_vintage_dates(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SOURCE ENDPOINTS (Extended)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get a source of economic data by ID
    ///
    /// # Arguments
    /// - `source_id` - Source ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    pub async fn get_source(
        &self,
        source_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
    ) -> ExchangeResult<Vec<Source>> {
        let mut params = HashMap::new();
        params.insert("source_id".to_string(), source_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }

        let response = self.get(FredEndpoint::Source, params).await?;
        FredParser::parse_sources(&response)
    }

    /// Get releases for a source
    ///
    /// # Arguments
    /// - `source_id` - Source ID
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field (release_id, name, press_release, realtime_start, realtime_end)
    /// - `sort_order` - Optional sort order (asc, desc)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_source_releases(
        &self,
        source_id: i64,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Release>> {
        let mut params = HashMap::new();
        params.insert("source_id".to_string(), source_id.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::SourceReleases, params).await?;
        FredParser::parse_releases(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TAG ENDPOINTS (Extended)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get related FRED tags for one or more FRED tags
    ///
    /// # Arguments
    /// - `tag_names` - Semicolon-separated tag names (required)
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `exclude_tag_names` - Optional semicolon-separated tag names to exclude
    /// - `tag_group_id` - Optional tag group (freq, gen, geo, geot, rls, seas, src)
    /// - `search_text` - Optional search tag names
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field (series_count, popularity, created, name, group_id)
    /// - `sort_order` - Optional sort order (asc, desc)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_related_tags(
        &self,
        tag_names: &str,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        exclude_tag_names: Option<&str>,
        tag_group_id: Option<&str>,
        search_text: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<Tag>> {
        let mut params = HashMap::new();
        params.insert("tag_names".to_string(), tag_names.to_string());

        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(exclude) = exclude_tag_names {
            params.insert("exclude_tag_names".to_string(), exclude.to_string());
        }
        if let Some(group) = tag_group_id {
            params.insert("tag_group_id".to_string(), group.to_string());
        }
        if let Some(text) = search_text {
            params.insert("search_text".to_string(), text.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::RelatedTags, params).await?;
        FredParser::parse_tags(&response)
    }

    /// Get series matching tags
    ///
    /// # Arguments
    /// - `tag_names` - Semicolon-separated tag names (required)
    /// - `exclude_tag_names` - Optional semicolon-separated tag names to exclude
    /// - `realtime_start` - Optional realtime start date (YYYY-MM-DD)
    /// - `realtime_end` - Optional realtime end date (YYYY-MM-DD)
    /// - `limit` - Optional limit (1-1000, default 1000)
    /// - `offset` - Optional pagination offset
    /// - `order_by` - Optional order field
    /// - `sort_order` - Optional sort order (asc, desc)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_tags_series(
        &self,
        tag_names: &str,
        exclude_tag_names: Option<&str>,
        realtime_start: Option<&str>,
        realtime_end: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> ExchangeResult<Vec<String>> {
        let mut params = HashMap::new();
        params.insert("tag_names".to_string(), tag_names.to_string());

        if let Some(exclude) = exclude_tag_names {
            params.insert("exclude_tag_names".to_string(), exclude.to_string());
        }
        if let Some(start) = realtime_start {
            params.insert("realtime_start".to_string(), start.to_string());
        }
        if let Some(end) = realtime_end {
            params.insert("realtime_end".to_string(), end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(order) = order_by {
            params.insert("order_by".to_string(), order.to_string());
        }
        if let Some(sort) = sort_order {
            params.insert("sort_order".to_string(), sort.to_string());
        }

        let response = self.get(FredEndpoint::TagsSeries, params).await?;
        FredParser::parse_series_search(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEW ENDPOINTS - RELEASE TABLES + GEOFRED (5 endpoints)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get hierarchical table tree for a release
    ///
    /// # Arguments
    /// - `release_id` - Release ID (required)
    /// - `element_id` - Optional element ID to filter
    /// - `include_observation_values` - Optional include observation values
    /// - `observation_date` - Optional observation date (YYYY-MM-DD)
    pub async fn get_release_tables(
        &self,
        release_id: i64,
        element_id: Option<i64>,
        include_observation_values: Option<bool>,
        observation_date: Option<&str>,
    ) -> ExchangeResult<Vec<FredReleaseTable>> {
        let mut params = HashMap::new();
        params.insert("release_id".to_string(), release_id.to_string());

        if let Some(elem_id) = element_id {
            params.insert("element_id".to_string(), elem_id.to_string());
        }
        if let Some(include_obs) = include_observation_values {
            params.insert("include_observation_values".to_string(), include_obs.to_string());
        }
        if let Some(date) = observation_date {
            params.insert("observation_date".to_string(), date.to_string());
        }

        let response = self.get(FredEndpoint::ReleaseTables, params).await?;
        FredParser::parse_release_tables(&response)
    }

    /// Get GeoFRED series group metadata
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID (required)
    ///
    /// # Returns
    /// Metadata about series that have geographical data
    pub async fn get_geo_series_group(
        &self,
        series_id: &str,
    ) -> ExchangeResult<FredGeoSeriesGroup> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        let response = self.get(FredEndpoint::GeoSeriesGroup, params).await?;
        FredParser::parse_geo_series_group(&response)
    }

    /// Get GeoFRED series data for mapping
    ///
    /// # Arguments
    /// - `series_id` - FRED series ID (required)
    /// - `date` - Optional date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Data for mapping a FRED series with regional values
    pub async fn get_geo_series_data(
        &self,
        series_id: &str,
        date: Option<&str>,
    ) -> ExchangeResult<FredGeoSeriesData> {
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id.to_string());

        if let Some(d) = date {
            params.insert("date".to_string(), d.to_string());
        }

        let response = self.get(FredEndpoint::GeoSeriesData, params).await?;
        FredParser::parse_geo_series_data(&response)
    }

    /// Get GeoFRED regional data across geographies
    ///
    /// # Arguments
    /// - `series_group` - Series group (required)
    /// - `region_type` - Region type: state, county, msa, country (required)
    /// - `date` - Optional date (YYYY-MM-DD)
    /// - `start_date` - Optional start date (YYYY-MM-DD)
    /// - `frequency` - Optional frequency
    /// - `units` - Optional units
    /// - `season` - Optional season
    /// - `transformation` - Optional transformation
    ///
    /// # Returns
    /// Regional economic data across geographies
    #[allow(clippy::too_many_arguments)]
    pub async fn get_geo_regional_data(
        &self,
        series_group: &str,
        region_type: &str,
        date: Option<&str>,
        start_date: Option<&str>,
        frequency: Option<&str>,
        units: Option<&str>,
        season: Option<&str>,
        transformation: Option<&str>,
    ) -> ExchangeResult<FredGeoRegionalData> {
        let mut params = HashMap::new();
        params.insert("series_group".to_string(), series_group.to_string());
        params.insert("region_type".to_string(), region_type.to_string());

        if let Some(d) = date {
            params.insert("date".to_string(), d.to_string());
        }
        if let Some(start) = start_date {
            params.insert("start_date".to_string(), start.to_string());
        }
        if let Some(freq) = frequency {
            params.insert("frequency".to_string(), freq.to_string());
        }
        if let Some(u) = units {
            params.insert("units".to_string(), u.to_string());
        }
        if let Some(s) = season {
            params.insert("season".to_string(), s.to_string());
        }
        if let Some(trans) = transformation {
            params.insert("transformation".to_string(), trans.to_string());
        }

        let response = self.get(FredEndpoint::GeoRegionalData, params).await?;
        FredParser::parse_geo_regional_data(&response)
    }

    /// Get GeoJSON boundary data for geographical regions
    ///
    /// # Arguments
    /// - `shape` - Shape type: bea, msa, necta, county, state, country, censusregion, censusdivision (required)
    ///
    /// # Returns
    /// GeoJSON boundary data for the specified shape type
    pub async fn get_geo_shapes_file(
        &self,
        shape: &str,
    ) -> ExchangeResult<FredGeoShapes> {
        let mut params = HashMap::new();
        params.insert("shape".to_string(), shape.to_string());

        let response = self.get(FredEndpoint::GeoShapesFile, params).await?;
        FredParser::parse_geo_shapes(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for FredConnector {
    fn exchange_name(&self) -> &'static str {
        "fred"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Fred
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // FRED is a data provider, not a trading platform
        // Use Spot as a placeholder for data access
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Adapt FRED to market data interface)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for FredConnector {
    /// Get current price (latest observation value)
    ///
    /// For FRED, this returns the most recent observation for a series.
    /// Symbol.base should contain the FRED series ID (e.g., "UNRATE").
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let series_id = format_series_id(&symbol);

        // Get latest observation (limit=1, sort descending)
        let mut params = HashMap::new();
        params.insert("series_id".to_string(), series_id);
        params.insert("limit".to_string(), "1".to_string());
        params.insert("sort_order".to_string(), "desc".to_string());

        let response = self.get(FredEndpoint::SeriesObservations, params).await?;
        let observations = FredParser::parse_observations(&response)?;

        let latest = observations
            .first()
            .and_then(|obs| obs.value)
            .ok_or_else(|| ExchangeError::NotFound("No observations available".to_string()))?;

        Ok(latest)
    }

    /// Get ticker (not applicable for FRED)
    ///
    /// FRED is economic data, not trading data - no bid/ask/volume concepts.
    async fn get_ticker(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is an economic data provider - ticker data not available".to_string(),
        ))
    }

    /// Get orderbook (not applicable for FRED)
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is an economic data provider - orderbook not available".to_string(),
        ))
    }

    /// Get klines/candles
    ///
    /// Adapts FRED observations to kline format.
    /// Each observation becomes a kline where open=close=high=low=value.
    ///
    /// Symbol.base should contain the FRED series ID.
    /// The `interval` parameter is ignored (FRED data has its own frequency).
    async fn get_klines(
        &self,
        symbol: Symbol,
        _interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let series_id = format_series_id(&symbol);

        let observations = self
            .get_series_observations(&series_id, None, None, limit.map(|l| l as u32))
            .await?;

        FredParser::observations_to_klines(observations)
    }

    /// Ping (check connection)
    async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping - try to get categories (lightweight endpoint)
        let params = HashMap::new();
        let _ = self.get(FredEndpoint::Category, params).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UnsupportedOperation - FRED is data only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for FredConnector {
    async fn market_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - trading not supported".to_string(),
        ))
    }

    async fn limit_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - trading not supported".to_string(),
        ))
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - trading not supported".to_string(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - trading not supported".to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - trading not supported".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UnsupportedOperation - FRED is data only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for FredConnector {
    async fn get_balance(
        &self,
        _asset: Option<Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - account operations not supported".to_string(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - account operations not supported".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UnsupportedOperation - FRED is data only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for FredConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - position tracking not supported".to_string(),
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is an economic data provider - funding rates not available".to_string(),
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "FRED is a data provider - leverage not applicable".to_string(),
        ))
    }
}
