//! Hacker News connector implementation

use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{HackerNewsParser, HnStory, HnUser};

/// Hacker News (Social News Feed) connector
///
/// Provides access to Hacker News stories, comments, jobs, and user data
/// via the Firebase-powered public API.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::hacker_news::HackerNewsConnector;
///
/// let connector = HackerNewsConnector::new();
///
/// // Get top 30 stories
/// let stories = connector.get_top_stories(Some(30)).await?;
///
/// // Get specific story
/// let story = connector.get_story(8863).await?;
///
/// // Get user profile
/// let user = connector.get_user("pg").await?;
/// ```
pub struct HackerNewsConnector {
    client: Client,
    auth: HackerNewsAuth,
    endpoints: HackerNewsEndpoints,
    testnet: bool,
}

impl HackerNewsConnector {
    /// Create new Hacker News connector
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            auth: HackerNewsAuth::new(),
            endpoints: HackerNewsEndpoints::default(),
            testnet: false,
        }
    }

    /// Internal: Make GET request to Hacker News API
    async fn get(&self, endpoint: HackerNewsEndpoint) -> ExchangeResult<Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json = response
            .json::<Value>()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    /// Internal: Fetch multiple stories concurrently with limit
    ///
    /// Implements concurrent fetching with max 10 concurrent requests to prevent fan-out
    async fn fetch_stories_concurrent(&self, ids: Vec<u64>) -> ExchangeResult<Vec<HnStory>> {
        const MAX_CONCURRENT: usize = 10;

        let mut stories = Vec::new();

        // Process in chunks of MAX_CONCURRENT
        for chunk in ids.chunks(MAX_CONCURRENT) {
            let mut handles = Vec::new();

            for &id in chunk {
                let connector = self.clone_light();
                let handle = tokio::spawn(async move {
                    connector.get_story(id).await
                });
                handles.push(handle);
            }

            // Wait for all in this chunk
            for handle in handles {
                match handle.await {
                    Ok(Ok(story)) => stories.push(story),
                    Ok(Err(_)) => {
                        // Skip stories that failed to fetch (deleted, not found, etc.)
                        continue;
                    }
                    Err(_) => {
                        // Task panicked, skip
                        continue;
                    }
                }
            }
        }

        Ok(stories)
    }

    /// Create a lightweight clone for concurrent operations
    fn clone_light(&self) -> Self {
        Self {
            client: self.client.clone(),
            auth: self.auth.clone(),
            endpoints: HackerNewsEndpoints::default(),
            testnet: self.testnet,
        }
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get top stories
    ///
    /// # Arguments
    /// - `limit` - Maximum number of stories to fetch (default: 30, max: 60)
    ///
    /// # Returns
    /// Vector of stories with full metadata
    ///
    /// # Note
    /// This method:
    /// 1. Fetches story IDs list (up to 500 available)
    /// 2. Takes first N IDs based on limit
    /// 3. Fetches each story concurrently (max 10 at a time)
    pub async fn get_top_stories(&self, limit: Option<usize>) -> ExchangeResult<Vec<HnStory>> {
        let limit = limit.unwrap_or(30).min(60);

        let data = self.get(HackerNewsEndpoint::TopStories).await?;
        let ids = HackerNewsParser::parse_story_ids(&data)?;

        let ids_to_fetch: Vec<u64> = ids.into_iter().take(limit).collect();

        self.fetch_stories_concurrent(ids_to_fetch).await
    }

    /// Get new stories (chronological)
    ///
    /// # Arguments
    /// - `limit` - Maximum number of stories to fetch (default: 30, max: 60)
    pub async fn get_new_stories(&self, limit: Option<usize>) -> ExchangeResult<Vec<HnStory>> {
        let limit = limit.unwrap_or(30).min(60);

        let data = self.get(HackerNewsEndpoint::NewStories).await?;
        let ids = HackerNewsParser::parse_story_ids(&data)?;

        let ids_to_fetch: Vec<u64> = ids.into_iter().take(limit).collect();

        self.fetch_stories_concurrent(ids_to_fetch).await
    }

    /// Get best stories (quality-ranked)
    ///
    /// # Arguments
    /// - `limit` - Maximum number of stories to fetch (default: 30, max: 60)
    pub async fn get_best_stories(&self, limit: Option<usize>) -> ExchangeResult<Vec<HnStory>> {
        let limit = limit.unwrap_or(30).min(60);

        let data = self.get(HackerNewsEndpoint::BestStories).await?;
        let ids = HackerNewsParser::parse_story_ids(&data)?;

        let ids_to_fetch: Vec<u64> = ids.into_iter().take(limit).collect();

        self.fetch_stories_concurrent(ids_to_fetch).await
    }

    /// Get Ask HN stories
    ///
    /// # Arguments
    /// - `limit` - Maximum number of stories to fetch (default: 30, max: 60)
    pub async fn get_ask_stories(&self, limit: Option<usize>) -> ExchangeResult<Vec<HnStory>> {
        let limit = limit.unwrap_or(30).min(60);

        let data = self.get(HackerNewsEndpoint::AskStories).await?;
        let ids = HackerNewsParser::parse_story_ids(&data)?;

        let ids_to_fetch: Vec<u64> = ids.into_iter().take(limit).collect();

        self.fetch_stories_concurrent(ids_to_fetch).await
    }

    /// Get Show HN stories
    ///
    /// # Arguments
    /// - `limit` - Maximum number of stories to fetch (default: 30, max: 60)
    pub async fn get_show_stories(&self, limit: Option<usize>) -> ExchangeResult<Vec<HnStory>> {
        let limit = limit.unwrap_or(30).min(60);

        let data = self.get(HackerNewsEndpoint::ShowStories).await?;
        let ids = HackerNewsParser::parse_story_ids(&data)?;

        let ids_to_fetch: Vec<u64> = ids.into_iter().take(limit).collect();

        self.fetch_stories_concurrent(ids_to_fetch).await
    }

    /// Get job stories
    ///
    /// # Arguments
    /// - `limit` - Maximum number of stories to fetch (default: 30, max: 60)
    pub async fn get_job_stories(&self, limit: Option<usize>) -> ExchangeResult<Vec<HnStory>> {
        let limit = limit.unwrap_or(30).min(60);

        let data = self.get(HackerNewsEndpoint::JobStories).await?;
        let ids = HackerNewsParser::parse_story_ids(&data)?;

        let ids_to_fetch: Vec<u64> = ids.into_iter().take(limit).collect();

        self.fetch_stories_concurrent(ids_to_fetch).await
    }

    /// Get specific story by ID
    ///
    /// # Arguments
    /// - `id` - Story ID
    ///
    /// # Returns
    /// Story metadata
    ///
    /// # Errors
    /// Returns `ExchangeError::NotFound` if story doesn't exist or is deleted
    pub async fn get_story(&self, id: u64) -> ExchangeResult<HnStory> {
        let data = self.get(HackerNewsEndpoint::Item { id }).await?;
        HackerNewsParser::parse_story(&data)
    }

    /// Get user profile
    ///
    /// # Arguments
    /// - `id` - Username (case-sensitive)
    ///
    /// # Returns
    /// User profile with karma, creation date, and bio
    ///
    /// # Errors
    /// Returns `ExchangeError::NotFound` if user doesn't exist
    pub async fn get_user(&self, id: &str) -> ExchangeResult<HnUser> {
        let data = self.get(HackerNewsEndpoint::User { id: id.to_string() }).await?;
        HackerNewsParser::parse_user(&data)
    }

    /// Get max item ID
    ///
    /// Returns the current largest item ID in the system.
    /// Useful for discovering new items by incrementing from the last known max.
    pub async fn get_max_item_id(&self) -> ExchangeResult<u64> {
        let data = self.get(HackerNewsEndpoint::MaxItem).await?;
        HackerNewsParser::parse_max_item(&data)
    }
}

impl Default for HackerNewsConnector {
    fn default() -> Self {
        Self::new()
    }
}
