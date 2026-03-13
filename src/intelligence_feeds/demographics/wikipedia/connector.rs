//! Wikipedia Pageviews connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{WikipediaParser, PageviewsEntry, TopArticle, TopCountry};

/// Wikipedia Pageviews connector
///
/// Provides access to Wikipedia article pageview statistics.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::wikipedia::WikipediaConnector;
///
/// let connector = WikipediaConnector::from_env();
///
/// // Get article views for Bitcoin
/// let views = connector.get_article_views("Bitcoin", "20240101", "20240131", "daily").await?;
///
/// // Get top articles for a specific day
/// let top = connector.get_top_articles("en.wikipedia", "2024", "01", "15").await?;
///
/// // Convenience method for stock ticker attention
/// let ticker_views = connector.get_ticker_attention("AAPL", "20240101", "20240131").await?;
/// ```
pub struct WikipediaConnector {
    client: Client,
    auth: WikipediaAuth,
    endpoints: WikipediaEndpoints,
    _testnet: bool,
}

impl WikipediaConnector {
    /// Create new Wikipedia connector with authentication
    pub fn new(auth: WikipediaAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: WikipediaEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Uses default User-Agent unless WIKIPEDIA_USER_AGENT is set
    pub fn from_env() -> Self {
        Self::new(WikipediaAuth::from_env())
    }

    /// Internal: Make GET request to Wikipedia Pageviews API
    async fn get(&self, url: String) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();
        self.auth.add_headers(&mut headers);

        let mut request = self.client.get(&url);

        for (key, value) in headers {
            request = request.header(key, value);
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

        // Check for Wikipedia API errors
        WikipediaParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CORE WIKIPEDIA PAGEVIEWS METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get pageviews for a specific article
    ///
    /// # Arguments
    /// - `article` - Article title (e.g., "Bitcoin", "Apple_Inc.")
    /// - `start` - Start date (YYYYMMDD format)
    /// - `end` - End date (YYYYMMDD format)
    /// - `granularity` - "daily" or "monthly"
    ///
    /// # Returns
    /// Vector of pageview entries with timestamps and view counts
    pub async fn get_article_views(
        &self,
        article: &str,
        start: &str,
        end: &str,
        granularity: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        let url = build_per_article_url(
            self.endpoints.rest_base,
            "en.wikipedia",
            "all-access",
            "all-agents",
            article,
            granularity,
            start,
            end,
        );

        let response = self.get(url).await?;
        WikipediaParser::parse_pageviews(&response)
    }

    /// Get daily pageviews for a specific article (convenience method)
    pub async fn get_article_views_daily(
        &self,
        article: &str,
        start: &str,
        end: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        self.get_article_views(article, start, end, "daily").await
    }

    /// Get monthly pageviews for a specific article (convenience method)
    pub async fn get_article_views_monthly(
        &self,
        article: &str,
        start: &str,
        end: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        self.get_article_views(article, start, end, "monthly").await
    }

    /// Get aggregate pageviews for entire project
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia", "de.wikipedia")
    /// - `start` - Start date (YYYYMMDD format)
    /// - `end` - End date (YYYYMMDD format)
    /// - `granularity` - "daily" or "monthly"
    ///
    /// # Returns
    /// Vector of aggregate pageview entries
    pub async fn get_aggregate_views(
        &self,
        project: &str,
        start: &str,
        end: &str,
        granularity: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        let url = build_aggregate_url(
            self.endpoints.rest_base,
            project,
            "all-access",
            "all-agents",
            granularity,
            start,
            end,
        );

        let response = self.get(url).await?;
        WikipediaParser::parse_pageviews(&response)
    }

    /// Get most viewed articles for a specific date
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia")
    /// - `year` - Year (YYYY)
    /// - `month` - Month (MM)
    /// - `day` - Day (DD)
    ///
    /// # Returns
    /// Vector of top articles with view counts and rankings
    pub async fn get_top_articles(
        &self,
        project: &str,
        year: &str,
        month: &str,
        day: &str,
    ) -> ExchangeResult<Vec<TopArticle>> {
        let url = build_top_url(
            self.endpoints.rest_base,
            project,
            "all-access",
            year,
            month,
            day,
        );

        let response = self.get(url).await?;
        WikipediaParser::parse_top_articles(&response)
    }

    /// Get pageviews by country for a specific month
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia")
    /// - `year` - Year (YYYY)
    /// - `month` - Month (MM)
    ///
    /// # Returns
    /// Vector of country entries with view counts
    pub async fn get_top_articles_by_country(
        &self,
        project: &str,
        year: &str,
        month: &str,
    ) -> ExchangeResult<Vec<TopCountry>> {
        let url = build_top_per_country_url(
            self.endpoints.rest_base,
            project,
            "all-access",
            year,
            month,
        );

        let response = self.get(url).await?;
        WikipediaParser::parse_top_by_country(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS FOR TRADING ATTENTION SIGNALS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Wikipedia attention for a stock ticker
    ///
    /// Searches for the company's Wikipedia page views as a proxy for retail attention.
    /// Common ticker symbols map to company names (e.g., AAPL -> Apple_Inc.)
    ///
    /// # Arguments
    /// - `ticker` - Stock ticker symbol (will be converted to company article)
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    pub async fn get_ticker_attention(
        &self,
        ticker: &str,
        start: &str,
        end: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        // Map common tickers to Wikipedia article names
        let article = match ticker.to_uppercase().as_str() {
            "AAPL" => "Apple_Inc.",
            "MSFT" => "Microsoft",
            "GOOGL" | "GOOG" => "Google",
            "AMZN" => "Amazon_(company)",
            "TSLA" => "Tesla,_Inc.",
            "META" | "FB" => "Meta_Platforms",
            "NVDA" => "Nvidia",
            "AMD" => "Advanced_Micro_Devices",
            "NFLX" => "Netflix",
            // Default: use ticker as-is
            _ => ticker,
        };

        self.get_article_views_daily(article, start, end).await
    }

    /// Get Wikipedia attention for a cryptocurrency
    ///
    /// # Arguments
    /// - `coin_name` - Cryptocurrency name (e.g., "Bitcoin", "Ethereum")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    pub async fn get_crypto_attention(
        &self,
        coin_name: &str,
        start: &str,
        end: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        self.get_article_views_daily(coin_name, start, end).await
    }

    /// Get Wikipedia attention for a company
    ///
    /// # Arguments
    /// - `company` - Company name with underscores (e.g., "Apple_Inc.")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    pub async fn get_company_attention(
        &self,
        company: &str,
        start: &str,
        end: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        self.get_article_views_daily(company, start, end).await
    }

    /// Get Wikipedia attention for a geopolitical topic
    ///
    /// # Arguments
    /// - `topic` - Topic name (e.g., "Ukraine", "Federal_Reserve")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    pub async fn get_geopolitical_attention(
        &self,
        topic: &str,
        start: &str,
        end: &str,
    ) -> ExchangeResult<Vec<PageviewsEntry>> {
        self.get_article_views_daily(topic, start, end).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get unique device counts for a project
    ///
    /// Counts the number of unique devices (browser sessions) accessing a
    /// Wikipedia project, which is a proxy for total unique users.
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    /// - `granularity` - "daily" or "monthly"
    ///
    /// # Returns
    /// Unique device counts as raw JSON
    pub async fn get_unique_devices(
        &self,
        project: &str,
        start: &str,
        end: &str,
        granularity: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let url = build_unique_devices_url(
            WIKIMEDIA_ANALYTICS_BASE,
            project,
            "all-sites",
            granularity,
            start,
            end,
        );
        self.get(url).await
    }

    /// Get edit counts for a project
    ///
    /// Returns aggregate edit statistics for a Wikipedia project.
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    /// - `granularity` - "daily", "monthly", or "yearly"
    ///
    /// # Returns
    /// Edit counts as raw JSON
    pub async fn get_edits(
        &self,
        project: &str,
        start: &str,
        end: &str,
        granularity: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let url = build_edits_url(
            WIKIMEDIA_ANALYTICS_BASE,
            project,
            "all-editor-types",
            "all-page-types",
            granularity,
            start,
            end,
        );
        self.get(url).await
    }

    /// Get editor counts for a project
    ///
    /// Returns the number of active editors on a Wikipedia project.
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    /// - `granularity` - "daily" or "monthly"
    ///
    /// # Returns
    /// Editor counts as raw JSON
    pub async fn get_editors(
        &self,
        project: &str,
        start: &str,
        end: &str,
        granularity: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let url = build_editors_url(
            WIKIMEDIA_ANALYTICS_BASE,
            project,
            "all-editor-types",
            "all-page-types",
            "all-activity-levels",
            granularity,
            start,
            end,
        );
        self.get(url).await
    }

    /// Get newly registered user counts for a project
    ///
    /// # Arguments
    /// - `project` - Project name (e.g., "en.wikipedia")
    /// - `start` - Start date (YYYYMMDD)
    /// - `end` - End date (YYYYMMDD)
    /// - `granularity` - "daily" or "monthly"
    ///
    /// # Returns
    /// Registered user counts as raw JSON
    pub async fn get_registered_users(
        &self,
        project: &str,
        start: &str,
        end: &str,
        granularity: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let url = build_registered_users_url(
            WIKIMEDIA_ANALYTICS_BASE,
            project,
            granularity,
            start,
            end,
        );
        self.get(url).await
    }
}
