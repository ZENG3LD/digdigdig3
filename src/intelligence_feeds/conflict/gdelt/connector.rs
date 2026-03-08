//! GDELT connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    GdeltParser, GdeltArticle, TimelinePoint, GdeltGeoResponse, TvClip, ContextResponse,
};

/// GDELT (Global Database of Events, Language, and Tone) connector
///
/// Provides access to global news coverage, events analysis, and media monitoring.
///
/// # APIs
/// - **DOC API**: News/events search and timelines
/// - **GEO API**: Geographic event mapping
/// - **TV API**: Television monitoring and analysis
/// - **CONTEXT API**: Contextual information
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::gdelt::GdeltConnector;
///
/// let connector = GdeltConnector::new();
///
/// // Search for articles about sanctions
/// let articles = connector.search_articles(
///     "sanctions",
///     DocMode::ArtList,
///     None,
///     None,
///     Some(100),
///     None,
/// ).await?;
///
/// // Get conflict events for a country
/// let conflicts = connector.get_conflict_events("Ukraine", None, None).await?;
///
/// // Get sentiment timeline
/// let timeline = connector.get_sentiment_timeline("Biden", None, None).await?;
/// ```
pub struct GdeltConnector {
    client: Client,
    auth: GdeltAuth,
    endpoints: GdeltEndpoints,
}

impl GdeltConnector {
    /// Create new GDELT connector
    ///
    /// GDELT API is public and requires no authentication
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: GdeltAuth::new(),
            endpoints: GdeltEndpoints::default(),
        }
    }

    /// Create connector from environment (no auth needed, but kept for consistency)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to GDELT API
    async fn get(
        &self,
        endpoint: GdeltEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for GDELT)
        self.auth.sign_query(&mut params);

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

        // Check for API errors
        GdeltParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DOC API - News/Events Search
    // ═══════════════════════════════════════════════════════════════════════

    /// Search articles using DOC API
    ///
    /// # Arguments
    /// - `query` - Search query (keywords, boolean operators, filters)
    /// - `mode` - Output mode (artlist, timelinevol, timelinetone, etc.)
    /// - `start_date` - Optional start date (YYYY-MM-DD or YYYYMMDDHHMMSS)
    /// - `end_date` - Optional end date (YYYY-MM-DD or YYYYMMDDHHMMSS)
    /// - `max_records` - Optional max records (default 250, max varies by mode)
    /// - `sort` - Optional sort order
    ///
    /// # Returns
    /// Vector of articles (when mode is ArtList or ArtGallery)
    pub async fn search_articles(
        &self,
        query: &str,
        mode: DocMode,
        start_date: Option<&str>,
        end_date: Option<&str>,
        max_records: Option<u32>,
        sort: Option<SortOrder>,
    ) -> ExchangeResult<Vec<GdeltArticle>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("mode".to_string(), mode.as_str().to_string());
        params.insert("format".to_string(), "json".to_string());

        if let Some(start) = start_date {
            params.insert("startdatetime".to_string(), format_gdelt_datetime(start));
        }
        if let Some(end) = end_date {
            params.insert("enddatetime".to_string(), format_gdelt_datetime(end));
        }
        if let Some(max) = max_records {
            params.insert("maxrecords".to_string(), max.to_string());
        }
        if let Some(s) = sort {
            params.insert("sort".to_string(), s.as_str().to_string());
        }

        let response = self.get(GdeltEndpoint::DocApi, params).await?;
        GdeltParser::parse_articles(&response)
    }

    /// Get article timeline (volume or tone over time)
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `mode` - Timeline mode (TimelineVol or TimelineTone)
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of timeline data points
    pub async fn get_article_timeline(
        &self,
        query: &str,
        mode: DocMode,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<TimelinePoint>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("mode".to_string(), mode.as_str().to_string());
        params.insert("format".to_string(), "json".to_string());

        if let Some(start) = start_date {
            params.insert("startdatetime".to_string(), format_gdelt_datetime(start));
        }
        if let Some(end) = end_date {
            params.insert("enddatetime".to_string(), format_gdelt_datetime(end));
        }

        let response = self.get(GdeltEndpoint::DocApi, params).await?;
        GdeltParser::parse_timeline(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GEO API - Geographic Events
    // ═══════════════════════════════════════════════════════════════════════

    /// Search geographic events
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `mode` - Geographic visualization mode
    ///
    /// # Returns
    /// GeoJSON response with geographic features
    pub async fn search_geo(
        &self,
        query: &str,
        mode: GeoMode,
    ) -> ExchangeResult<GdeltGeoResponse> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("mode".to_string(), mode.as_str().to_string());
        params.insert("format".to_string(), "GeoJSON".to_string());

        let response = self.get(GdeltEndpoint::GeoApi, params).await?;
        GdeltParser::parse_geo(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TV API - Television Monitoring
    // ═══════════════════════════════════════════════════════════════════════

    /// Search TV content
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `mode` - TV output mode
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of TV clips or timeline data
    pub async fn search_tv(
        &self,
        query: &str,
        mode: TvMode,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<TvClip>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("mode".to_string(), mode.as_str().to_string());
        params.insert("format".to_string(), "json".to_string());

        if let Some(start) = start_date {
            params.insert("startdatetime".to_string(), format_gdelt_datetime(start));
        }
        if let Some(end) = end_date {
            params.insert("enddatetime".to_string(), format_gdelt_datetime(end));
        }

        let response = self.get(GdeltEndpoint::TvApi, params).await?;
        GdeltParser::parse_tv_clips(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONTEXT API
    // ═══════════════════════════════════════════════════════════════════════

    /// Get context for query
    ///
    /// # Arguments
    /// - `query` - Context query
    ///
    /// # Returns
    /// Context response (structure varies)
    pub async fn get_context(&self, query: &str) -> ExchangeResult<ContextResponse> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("format".to_string(), "json".to_string());

        let response = self.get(GdeltEndpoint::ContextApi, params).await?;
        GdeltParser::parse_context(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE GETTERS - Domain-Specific Queries
    // ═══════════════════════════════════════════════════════════════════════

    /// Get conflict/war events for a country
    ///
    /// # Arguments
    /// - `country` - Country name or code
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of articles about conflicts
    pub async fn get_conflict_events(
        &self,
        country: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<GdeltArticle>> {
        let query = format!("(conflict OR war OR military) sourcecountry:{}", country);
        self.search_articles(&query, DocMode::ArtList, start_date, end_date, Some(250), None)
            .await
    }

    /// Get economic news for a topic
    ///
    /// # Arguments
    /// - `topic` - Economic topic (e.g., "GDP", "inflation", "employment")
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of articles about economic topics
    pub async fn get_economic_news(
        &self,
        topic: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<GdeltArticle>> {
        let query = format!("(economy OR GDP OR inflation OR recession) {}", topic);
        self.search_articles(&query, DocMode::ArtList, start_date, end_date, Some(250), None)
            .await
    }

    /// Get sanctions-related news for a country
    ///
    /// # Arguments
    /// - `country` - Country name
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of articles about sanctions
    pub async fn get_sanctions_news(
        &self,
        country: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<GdeltArticle>> {
        let query = format!("sanctions {}", country);
        self.search_articles(&query, DocMode::ArtList, start_date, end_date, Some(250), None)
            .await
    }

    /// Get election-related news for a country
    ///
    /// # Arguments
    /// - `country` - Country name
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of articles about elections
    pub async fn get_election_news(
        &self,
        country: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<GdeltArticle>> {
        let query = format!("(election OR vote) {}", country);
        self.search_articles(&query, DocMode::ArtList, start_date, end_date, Some(250), None)
            .await
    }

    /// Get sentiment timeline for a query
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of timeline points with tone scores
    pub async fn get_sentiment_timeline(
        &self,
        query: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<TimelinePoint>> {
        self.get_article_timeline(query, DocMode::TimelineTone, start_date, end_date)
            .await
    }

    /// Get volume timeline for a query
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `start_date` - Optional start date
    /// - `end_date` - Optional end date
    ///
    /// # Returns
    /// Vector of timeline points with article counts
    pub async fn get_volume_timeline(
        &self,
        query: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> ExchangeResult<Vec<TimelinePoint>> {
        self.get_article_timeline(query, DocMode::TimelineVol, start_date, end_date)
            .await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // REAL-TIME MONITORING - Timespan-based queries
    // ═══════════════════════════════════════════════════════════════════════

    /// Get articles from the last 15 minutes (GKG real-time feed equivalent)
    ///
    /// # Arguments
    /// - `query` - Optional search query (defaults to "*" for all articles)
    ///
    /// # Returns
    /// Vector of articles published in the last 15 minutes
    ///
    /// # Example
    /// ```ignore
    /// // Get all articles from last 15 minutes
    /// let recent = connector.get_latest_15min(None).await?;
    ///
    /// // Get specific topic from last 15 minutes
    /// let tech = connector.get_latest_15min(Some("technology")).await?;
    /// ```
    pub async fn get_latest_15min(&self, query: Option<&str>) -> ExchangeResult<Vec<GdeltArticle>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.unwrap_or("*").to_string());
        params.insert("mode".to_string(), "artlist".to_string());
        params.insert("format".to_string(), "json".to_string());
        params.insert("timespan".to_string(), "15min".to_string());
        params.insert("maxrecords".to_string(), "250".to_string());

        let response = self.get(GdeltEndpoint::DocApi, params).await?;
        GdeltParser::parse_articles(&response)
    }

    /// Get geographic events from a recent time window for mapping
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `timespan` - Optional timespan (e.g., "1hour", "15min", "3days")
    ///
    /// # Returns
    /// GeoJSON response with geographic features
    ///
    /// # Example
    /// ```ignore
    /// // Get conflict events from last hour
    /// let conflicts = connector.get_geo_recent("conflict", Some("1hour")).await?;
    /// ```
    pub async fn get_geo_recent(&self, query: &str, timespan: Option<&str>) -> ExchangeResult<GdeltGeoResponse> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("mode".to_string(), "PointData".to_string());
        params.insert("format".to_string(), "GeoJSON".to_string());
        if let Some(ts) = timespan {
            params.insert("timespan".to_string(), ts.to_string());
        }

        let response = self.get(GdeltEndpoint::GeoApi, params).await?;
        GdeltParser::parse_geo(&response)
    }

    /// Get real-time conflict events (last 15 min) for map overlay
    ///
    /// # Returns
    /// Vector of conflict-related articles from the last 15 minutes
    ///
    /// # Example
    /// ```ignore
    /// let conflicts = connector.get_realtime_conflicts().await?;
    /// for article in conflicts {
    ///     println!("{}: {}", article.title, article.url);
    /// }
    /// ```
    pub async fn get_realtime_conflicts(&self) -> ExchangeResult<Vec<GdeltArticle>> {
        self.get_latest_15min(Some("(conflict OR war OR attack OR bombing OR missile)")).await
    }

    /// Get real-time disaster events (last 15 min)
    ///
    /// # Returns
    /// Vector of disaster-related articles from the last 15 minutes
    pub async fn get_realtime_disasters(&self) -> ExchangeResult<Vec<GdeltArticle>> {
        self.get_latest_15min(Some("(earthquake OR flood OR hurricane OR wildfire OR tsunami)")).await
    }

    /// Get real-time terrorism events (last 15 min)
    ///
    /// # Returns
    /// Vector of terrorism-related articles from the last 15 minutes
    pub async fn get_realtime_terrorism(&self) -> ExchangeResult<Vec<GdeltArticle>> {
        self.get_latest_15min(Some("(terrorism OR terrorist OR terror attack OR bombing)")).await
    }
}

impl Default for GdeltConnector {
    fn default() -> Self {
        Self::new()
    }
}
