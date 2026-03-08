//! RSS Feed Proxy connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::auth::RssProxyAuth;
use super::endpoints::{self, RssProxyEndpoints};
use super::parser::{RssFeed, RssFeedItem, RssProxyParser};

/// RSS Feed Proxy connector
///
/// Fetches and parses RSS/Atom feeds from 100+ approved news domains.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::rss_proxy::RssProxyConnector;
///
/// let connector = RssProxyConnector::new();
///
/// // Fetch specific feed
/// let bbc = connector.fetch_bbc_news().await?;
///
/// // Fetch all top news sources
/// let all_news = connector.fetch_all_news().await?;
///
/// // Fetch tech feeds
/// let tech = connector.fetch_tech_feeds().await?;
///
/// // Fetch custom feed
/// let custom = connector.fetch_feed("https://example.com/feed.xml").await?;
/// ```
pub struct RssProxyConnector {
    client: Client,
    auth: RssProxyAuth,
    _endpoints: RssProxyEndpoints,
}

impl RssProxyConnector {
    /// Create new RSS Feed Proxy connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: RssProxyAuth::new(),
            _endpoints: RssProxyEndpoints,
        }
    }

    /// Internal: Fetch XML from URL
    async fn fetch_xml(&self, url: &str) -> ExchangeResult<String> {
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(url);

        // Add headers
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
                message: format!("HTTP {}: Failed to fetch feed", response.status()),
            });
        }

        let xml = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(xml)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Fetch and parse any RSS/Atom feed by URL
    ///
    /// # Arguments
    /// - `url` - Feed URL
    ///
    /// # Returns
    /// Parsed feed with items
    pub async fn fetch_feed(&self, url: &str) -> ExchangeResult<RssFeed> {
        let xml = self.fetch_xml(url).await?;
        RssProxyParser::parse_feed(&xml)
    }

    // ==========================================================================
    // INDIVIDUAL NEWS SOURCES
    // ==========================================================================

    /// Fetch BBC World News
    pub async fn fetch_bbc_news(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::BBC_WORLD).await
    }

    /// Fetch BBC Technology
    pub async fn fetch_bbc_tech(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::BBC_TECH).await
    }

    /// Fetch BBC Business
    pub async fn fetch_bbc_business(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::BBC_BUSINESS)
            .await
    }

    /// Fetch Reuters World News
    pub async fn fetch_reuters(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::REUTERS_WORLD)
            .await
    }

    /// Fetch Reuters Business
    pub async fn fetch_reuters_business(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::REUTERS_BUSINESS)
            .await
    }

    /// Fetch Reuters Technology
    pub async fn fetch_reuters_tech(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::REUTERS_TECH)
            .await
    }

    /// Fetch NPR News
    pub async fn fetch_npr_news(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::NPR_NEWS).await
    }

    /// Fetch NPR Business
    pub async fn fetch_npr_business(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::NPR_BUSINESS)
            .await
    }

    /// Fetch NPR Technology
    pub async fn fetch_npr_tech(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::NPR_TECH).await
    }

    /// Fetch The Guardian World
    pub async fn fetch_guardian(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::GUARDIAN_WORLD)
            .await
    }

    /// Fetch The Guardian Business
    pub async fn fetch_guardian_business(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::GUARDIAN_BUSINESS)
            .await
    }

    /// Fetch The Guardian Technology
    pub async fn fetch_guardian_tech(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::GUARDIAN_TECH)
            .await
    }

    /// Fetch Al Jazeera
    pub async fn fetch_aljazeera(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::ALJAZEERA).await
    }

    /// Fetch CNN Top Stories
    pub async fn fetch_cnn(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::news_sources::CNN_TOP).await
    }

    /// Fetch TechCrunch
    pub async fn fetch_techcrunch(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::tech_sources::TECHCRUNCH).await
    }

    /// Fetch Ars Technica
    pub async fn fetch_ars_technica(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::tech_sources::ARS_TECHNICA).await
    }

    /// Fetch The Verge
    pub async fn fetch_the_verge(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::tech_sources::THE_VERGE).await
    }

    /// Fetch Wired
    pub async fn fetch_wired(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::tech_sources::WIRED).await
    }

    /// Fetch CSIS (Center for Strategic and International Studies)
    pub async fn fetch_csis(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::policy_sources::CSIS).await
    }

    /// Fetch Brookings Institution
    pub async fn fetch_brookings(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::policy_sources::BROOKINGS).await
    }

    /// Fetch Carnegie Endowment
    pub async fn fetch_carnegie(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::policy_sources::CARNEGIE).await
    }

    /// Fetch Council on Foreign Relations
    pub async fn fetch_cfr(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::policy_sources::CFR).await
    }

    /// Fetch RAND Corporation
    pub async fn fetch_rand(&self) -> ExchangeResult<RssFeed> {
        self.fetch_feed(endpoints::policy_sources::RAND).await
    }

    // ==========================================================================
    // BATCH FETCHING
    // ==========================================================================

    /// Fetch all top news sources
    ///
    /// Returns feeds from BBC, Reuters, NPR, Guardian, Al Jazeera, CNN
    pub async fn fetch_all_news(&self) -> ExchangeResult<Vec<RssFeed>> {
        let urls = endpoints::all_news();
        self.fetch_multiple(&urls).await
    }

    /// Fetch all tech news sources
    ///
    /// Returns feeds from TechCrunch, Ars Technica, The Verge, Wired, MIT Tech Review
    pub async fn fetch_tech_feeds(&self) -> ExchangeResult<Vec<RssFeed>> {
        let urls = endpoints::all_tech();
        self.fetch_multiple(&urls).await
    }

    /// Fetch all policy/think tank sources
    ///
    /// Returns feeds from CSIS, Brookings, Carnegie, CFR, RAND, War on the Rocks
    pub async fn fetch_policy_feeds(&self) -> ExchangeResult<Vec<RssFeed>> {
        let urls = endpoints::all_policy();
        self.fetch_multiple(&urls).await
    }

    /// Fetch all finance sources
    ///
    /// Returns feeds from FT, Bloomberg, MarketWatch, Seeking Alpha
    pub async fn fetch_finance_feeds(&self) -> ExchangeResult<Vec<RssFeed>> {
        let urls = endpoints::all_finance();
        self.fetch_multiple(&urls).await
    }

    /// Fetch all cybersecurity sources
    ///
    /// Returns feeds from Krebs, Schneier, Dark Reading, The Hacker News
    pub async fn fetch_cyber_feeds(&self) -> ExchangeResult<Vec<RssFeed>> {
        let urls = endpoints::all_cyber();
        self.fetch_multiple(&urls).await
    }

    /// Fetch multiple feeds concurrently
    ///
    /// # Arguments
    /// - `urls` - Slice of feed URLs
    ///
    /// # Returns
    /// Vector of successfully fetched feeds (failures are skipped)
    pub async fn fetch_multiple(&self, urls: &[&str]) -> ExchangeResult<Vec<RssFeed>> {
        let mut tasks = Vec::new();

        for url in urls {
            let url_owned = url.to_string();
            let connector = self.clone();
            let task = tokio::spawn(async move { connector.fetch_feed(&url_owned).await });
            tasks.push(task);
        }

        let mut feeds = Vec::new();
        for task in tasks {
            if let Ok(Ok(feed)) = task.await {
                feeds.push(feed);
            }
            // Skip errors - continue fetching other feeds
        }

        Ok(feeds)
    }

    /// Aggregate all items from multiple feeds into a single list
    ///
    /// # Arguments
    /// - `feeds` - Vector of feeds to aggregate
    ///
    /// # Returns
    /// Vector of all items with source_name tagged
    pub fn aggregate_items(&self, feeds: &[RssFeed]) -> Vec<RssFeedItem> {
        let mut all_items = Vec::new();

        for feed in feeds {
            for item in &feed.items {
                let mut item = item.clone();
                // Tag with source feed name if not already present
                if item.source_name.is_none() {
                    item.source_name = Some(feed.title.clone());
                }
                all_items.push(item);
            }
        }

        // Sort by date (newest first)
        all_items.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

        all_items
    }
}

impl Clone for RssProxyConnector {
    fn clone(&self) -> Self {
        Self {
            client: Client::new(),
            auth: self.auth.clone(),
            _endpoints: RssProxyEndpoints,
        }
    }
}

impl Default for RssProxyConnector {
    fn default() -> Self {
        Self::new()
    }
}
