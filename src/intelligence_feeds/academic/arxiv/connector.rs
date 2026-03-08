//! arXiv connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{ArxivParser, ArxivSearchResult};

/// arXiv (Academic Papers) connector
///
/// Provides access to 2+ million research papers from arXiv.org.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::arxiv::ArxivConnector;
///
/// let connector = ArxivConnector::new();
///
/// // Search papers
/// let results = connector.search("machine learning", 0, 10).await?;
///
/// // Search by title
/// let papers = connector.search_by_title("neural networks", 10).await?;
///
/// // Get quantitative finance papers
/// let qfin_papers = connector.get_quantitative_finance(20).await?;
/// ```
pub struct ArxivConnector {
    client: Client,
    auth: ArxivAuth,
    endpoints: ArxivEndpoints,
    _testnet: bool,
}

impl ArxivConnector {
    /// Create new arXiv connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: ArxivAuth::new(),
            endpoints: ArxivEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to arXiv API
    async fn get(&self, mut params: HashMap<String, String>) -> ExchangeResult<String> {
        // No authentication needed for arXiv
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, ArxivEndpoint::Query.path());

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

        let xml = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(xml)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Search papers with custom query
    ///
    /// # Arguments
    /// - `query` - Search query (e.g., "all:machine learning", "ti:neural", "au:Bengio")
    /// - `start` - Start index for pagination
    /// - `max_results` - Maximum number of results to return (default: 10, max: 30000)
    ///
    /// # Query syntax
    /// - `all:term` - All fields
    /// - `ti:term` - Title
    /// - `au:author` - Author
    /// - `abs:term` - Abstract
    /// - `cat:category` - Category (e.g., cs.AI, q-fin.TR)
    /// - Boolean: `AND`, `OR`, `ANDNOT`
    ///
    /// # Returns
    /// Search result with papers
    pub async fn search(
        &self,
        query: &str,
        start: u64,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let mut params = HashMap::new();
        params.insert("search_query".to_string(), query.to_string());
        params.insert("start".to_string(), start.to_string());
        params.insert("max_results".to_string(), max_results.to_string());

        let xml = self.get(params).await?;
        ArxivParser::parse_search_result(&xml)
    }

    /// Search papers by title
    ///
    /// # Arguments
    /// - `title` - Title search term
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn search_by_title(
        &self,
        title: &str,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = format!("ti:{}", title);
        self.search(&query, 0, max_results).await
    }

    /// Search papers by author
    ///
    /// # Arguments
    /// - `author` - Author name
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn search_by_author(
        &self,
        author: &str,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = format!("au:{}", author);
        self.search(&query, 0, max_results).await
    }

    /// Search papers by category
    ///
    /// # Arguments
    /// - `category` - Category code (e.g., "cs.AI", "q-fin.TR", "econ.GN")
    /// - `max_results` - Maximum number of results (default: 10)
    ///
    /// # Common categories
    /// - `cs.AI` - Artificial Intelligence
    /// - `cs.LG` - Machine Learning
    /// - `stat.ML` - Statistics - Machine Learning
    /// - `q-fin.TR` - Trading and Market Microstructure
    /// - `q-fin.ST` - Statistical Finance
    /// - `q-fin.PM` - Portfolio Management
    /// - `q-fin.RM` - Risk Management
    /// - `econ.GN` - General Economics
    pub async fn search_by_category(
        &self,
        category: &str,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = format!("cat:{}", category);
        self.search(&query, 0, max_results).await
    }

    /// Get specific paper by arXiv ID
    ///
    /// # Arguments
    /// - `arxiv_id` - arXiv identifier (e.g., "2301.12345" or "2301.12345v1")
    pub async fn get_paper(&self, arxiv_id: &str) -> ExchangeResult<ArxivSearchResult> {
        let mut params = HashMap::new();
        params.insert("id_list".to_string(), arxiv_id.to_string());
        params.insert("max_results".to_string(), "1".to_string());

        let xml = self.get(params).await?;
        ArxivParser::parse_search_result(&xml)
    }

    /// Get quantitative finance papers
    ///
    /// Searches across all q-fin.* categories
    ///
    /// # Arguments
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn get_quantitative_finance(
        &self,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = "cat:q-fin.*";
        self.search(query, 0, max_results).await
    }

    /// Get machine learning papers
    ///
    /// Searches cs.LG, cs.AI, and stat.ML categories
    ///
    /// # Arguments
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn get_machine_learning(
        &self,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = "cat:cs.LG OR cat:cs.AI OR cat:stat.ML";
        self.search(query, 0, max_results).await
    }

    /// Get economics papers
    ///
    /// Searches all econ.* categories
    ///
    /// # Arguments
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn get_economics(&self, max_results: u64) -> ExchangeResult<ArxivSearchResult> {
        let query = "cat:econ.*";
        self.search(query, 0, max_results).await
    }

    /// Get algorithmic trading research papers
    ///
    /// Searches for "algorithmic trading" OR "market microstructure"
    ///
    /// # Arguments
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn get_trading_research(
        &self,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = "all:\"algorithmic trading\" OR all:\"market microstructure\"";
        self.search(query, 0, max_results).await
    }

    /// Get cryptocurrency research papers
    ///
    /// Searches for "cryptocurrency" OR "blockchain" OR "DeFi"
    ///
    /// # Arguments
    /// - `max_results` - Maximum number of results (default: 10)
    pub async fn get_crypto_research(
        &self,
        max_results: u64,
    ) -> ExchangeResult<ArxivSearchResult> {
        let query = "all:cryptocurrency OR all:blockchain OR all:DeFi";
        self.search(query, 0, max_results).await
    }
}

impl Default for ArxivConnector {
    fn default() -> Self {
        Self::new()
    }
}
