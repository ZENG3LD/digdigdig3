//! NewsAPI.org connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{NewsApiParser, NewsArticle, NewsSourceMetadata};

/// NewsAPI.org connector
///
/// Provides access to news articles from 150,000+ sources worldwide.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::newsapi::NewsApiConnector;
///
/// let connector = NewsApiConnector::from_env();
///
/// // Get top business headlines
/// let articles = connector.get_business_news(Some("us")).await?;
///
/// // Get crypto news
/// let crypto = connector.get_crypto_news().await?;
///
/// // Search for specific topics
/// let results = connector.get_everything(Some("bitcoin"), None, None, None, None, None, None, None, None).await?;
/// ```
pub struct NewsApiConnector {
    client: Client,
    auth: NewsApiAuth,
    endpoints: NewsApiEndpoints,
}

impl NewsApiConnector {
    /// Create new NewsAPI connector with authentication
    pub fn new(auth: NewsApiAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: NewsApiEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `NEWSAPI_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(NewsApiAuth::from_env())
    }

    /// Internal: Make GET request to NewsAPI
    async fn get(
        &self,
        endpoint: NewsApiEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Build headers with authentication
        let mut headers = reqwest::header::HeaderMap::new();
        self.auth.sign_headers(&mut headers);

        let response = self
            .client
            .get(&url)
            .headers(headers)
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

        // Check for NewsAPI errors
        NewsApiParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEWSAPI-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get top headlines from a specific country/category
    ///
    /// # Arguments
    /// - `country` - 2-letter ISO 3166-1 country code (e.g., "us", "gb", "de")
    /// - `category` - News category (business, entertainment, general, health, science, sports, technology)
    /// - `query` - Keywords or phrases to search for in article titles and descriptions
    /// - `page_size` - Number of results to return per page (max 100)
    /// - `page` - Page number to retrieve
    ///
    /// # Returns
    /// Vector of news articles
    pub async fn get_top_headlines(
        &self,
        country: Option<&str>,
        category: Option<NewsCategory>,
        query: Option<&str>,
        page_size: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<NewsArticle>> {
        let mut params = HashMap::new();

        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }
        if let Some(cat) = category {
            params.insert("category".to_string(), cat.as_str().to_string());
        }
        if let Some(q) = query {
            params.insert("q".to_string(), q.to_string());
        }
        if let Some(ps) = page_size {
            params.insert("pageSize".to_string(), ps.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(NewsApiEndpoint::TopHeadlines, params).await?;
        NewsApiParser::parse_articles(&response)
    }

    /// Search everything - all articles matching search criteria
    ///
    /// # Arguments
    /// - `query` - Keywords or phrases to search for (required if sources/domains not specified)
    /// - `sources` - Comma-separated list of news source IDs (e.g., "bbc-news,cnn")
    /// - `domains` - Comma-separated list of domains (e.g., "bbc.co.uk,cnn.com")
    /// - `from` - Start date (ISO 8601 format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)
    /// - `to` - End date (ISO 8601 format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)
    /// - `language` - 2-letter ISO 639-1 language code
    /// - `sort_by` - Sort order (relevancy, popularity, publishedAt)
    /// - `page_size` - Number of results to return per page (max 100)
    /// - `page` - Page number to retrieve
    ///
    /// # Returns
    /// Vector of news articles
    #[allow(clippy::too_many_arguments)]
    pub async fn get_everything(
        &self,
        query: Option<&str>,
        sources: Option<&str>,
        domains: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
        language: Option<NewsLanguage>,
        sort_by: Option<NewsSortBy>,
        page_size: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<NewsArticle>> {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("q".to_string(), q.to_string());
        }
        if let Some(s) = sources {
            params.insert("sources".to_string(), s.to_string());
        }
        if let Some(d) = domains {
            params.insert("domains".to_string(), d.to_string());
        }
        if let Some(f) = from {
            params.insert("from".to_string(), f.to_string());
        }
        if let Some(t) = to {
            params.insert("to".to_string(), t.to_string());
        }
        if let Some(l) = language {
            params.insert("language".to_string(), l.as_str().to_string());
        }
        if let Some(sb) = sort_by {
            params.insert("sortBy".to_string(), sb.as_str().to_string());
        }
        if let Some(ps) = page_size {
            params.insert("pageSize".to_string(), ps.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(NewsApiEndpoint::Everything, params).await?;
        NewsApiParser::parse_articles(&response)
    }

    /// Get list of available news sources
    ///
    /// # Arguments
    /// - `category` - Filter by category
    /// - `language` - Filter by language
    /// - `country` - Filter by country (2-letter ISO code)
    ///
    /// # Returns
    /// Vector of news source metadata
    pub async fn get_sources(
        &self,
        category: Option<NewsCategory>,
        language: Option<NewsLanguage>,
        country: Option<&str>,
    ) -> ExchangeResult<Vec<NewsSourceMetadata>> {
        let mut params = HashMap::new();

        if let Some(cat) = category {
            params.insert("category".to_string(), cat.as_str().to_string());
        }
        if let Some(l) = language {
            params.insert("language".to_string(), l.as_str().to_string());
        }
        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }

        let response = self.get(NewsApiEndpoint::Sources, params).await?;
        NewsApiParser::parse_sources(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get business news for a specific country
    ///
    /// Shortcut for `get_top_headlines()` with category=business
    pub async fn get_business_news(&self, country: Option<&str>) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_top_headlines(country, Some(NewsCategory::Business), None, None, None)
            .await
    }

    /// Get technology news for a specific country
    ///
    /// Shortcut for `get_top_headlines()` with category=technology
    pub async fn get_tech_news(&self, country: Option<&str>) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_top_headlines(country, Some(NewsCategory::Technology), None, None, None)
            .await
    }

    /// Get science news for a specific country
    ///
    /// Shortcut for `get_top_headlines()` with category=science
    pub async fn get_science_news(&self, country: Option<&str>) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_top_headlines(country, Some(NewsCategory::Science), None, None, None)
            .await
    }

    /// Get financial news
    ///
    /// Searches everything with financial keywords
    pub async fn get_financial_news(&self, query: Option<&str>) -> ExchangeResult<Vec<NewsArticle>> {
        let search_query = query.unwrap_or("finance OR stock OR bonds OR investment");
        self.get_everything(
            Some(search_query),
            None,
            None,
            None,
            None,
            Some(NewsLanguage::English),
            Some(NewsSortBy::PublishedAt),
            None,
            None,
        )
        .await
    }

    /// Get cryptocurrency news
    ///
    /// Searches for crypto-related articles
    pub async fn get_crypto_news(&self) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_everything(
            Some("cryptocurrency OR bitcoin OR ethereum OR crypto"),
            None,
            None,
            None,
            None,
            Some(NewsLanguage::English),
            Some(NewsSortBy::PublishedAt),
            None,
            None,
        )
        .await
    }

    /// Get stock market news
    ///
    /// Searches for major stock market indices
    pub async fn get_market_news(&self) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_everything(
            Some("\"stock market\" OR \"S&P 500\" OR NASDAQ OR \"Dow Jones\""),
            None,
            None,
            None,
            None,
            Some(NewsLanguage::English),
            Some(NewsSortBy::PublishedAt),
            None,
            None,
        )
        .await
    }

    /// Get economic news
    ///
    /// Searches for macroeconomic indicators and central bank news
    pub async fn get_economic_news(&self) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_everything(
            Some("GDP OR inflation OR unemployment OR \"Federal Reserve\" OR \"interest rate\""),
            None,
            None,
            None,
            None,
            Some(NewsLanguage::English),
            Some(NewsSortBy::PublishedAt),
            None,
            None,
        )
        .await
    }

    /// Get geopolitical news
    ///
    /// Searches for geopolitical events that may affect markets
    pub async fn get_geopolitical_news(&self) -> ExchangeResult<Vec<NewsArticle>> {
        self.get_everything(
            Some("sanctions OR \"trade war\" OR geopolitical OR \"international relations\""),
            None,
            None,
            None,
            None,
            Some(NewsLanguage::English),
            Some(NewsSortBy::PublishedAt),
            None,
            None,
        )
        .await
    }
}
