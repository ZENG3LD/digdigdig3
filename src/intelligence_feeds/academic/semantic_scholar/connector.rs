//! Semantic Scholar connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    SemanticScholarParser, ScholarPaper, ScholarAuthor, ScholarSearchResult, ScholarCitation,
};

/// Semantic Scholar (Academic Research API) connector
///
/// Provides access to 200M+ academic papers with citations, authors, and metadata.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::semantic_scholar::SemanticScholarConnector;
///
/// let connector = SemanticScholarConnector::from_env();
///
/// // Search for papers
/// let results = connector.search_papers("machine learning", Some(10), None, None, None).await?;
///
/// // Get paper details
/// let paper = connector.get_paper("649def34f8be52c8b66281af98ae884c09aef38b").await?;
///
/// // Get citations
/// let citations = connector.get_paper_citations("649def34f8be52c8b66281af98ae884c09aef38b", Some(20), None).await?;
/// ```
pub struct SemanticScholarConnector {
    client: Client,
    auth: SemanticScholarAuth,
    endpoints: SemanticScholarEndpoints,
    _testnet: bool,
}

impl SemanticScholarConnector {
    /// Create new Semantic Scholar connector with authentication
    pub fn new(auth: SemanticScholarAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: SemanticScholarEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `SEMANTIC_SCHOLAR_API_KEY` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(SemanticScholarAuth::from_env())
    }

    /// Create connector without API key (lower rate limits)
    pub fn unauthenticated() -> Self {
        Self::new(SemanticScholarAuth::unauthenticated())
    }

    /// Internal: Make GET request to Semantic Scholar API
    async fn get(
        &self,
        endpoint: SemanticScholarEndpoint,
        path_override: Option<String>,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();

        // Add API key authentication
        self.auth.sign_headers(&mut headers);

        let path = path_override.unwrap_or_else(|| endpoint.path().to_string());
        let url = format!("{}{}", self.endpoints.rest_base, path);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

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

        // Check for Semantic Scholar API errors
        SemanticScholarParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PAPER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for papers by query
    ///
    /// # Arguments
    /// - `query` - Search query string (required)
    /// - `limit` - Number of results (1-100, default 10)
    /// - `offset` - Pagination offset (default 0)
    /// - `year` - Filter by publication year (e.g., "2020" or "2018-2020")
    /// - `fields_of_study` - Filter by field (e.g., "Computer Science")
    ///
    /// # Returns
    /// Search results with papers and pagination info
    pub async fn search_papers(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        year: Option<&str>,
        fields_of_study: Option<&str>,
    ) -> ExchangeResult<ScholarSearchResult> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }
        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }
        if let Some(fields) = fields_of_study {
            params.insert("fieldsOfStudy".to_string(), fields.to_string());
        }

        // Request all available fields
        params.insert(
            "fields".to_string(),
            "paperId,title,abstract,year,citationCount,referenceCount,influentialCitationCount,venue,url,authors,fieldsOfStudy,publicationDate".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::PaperSearch, None, params).await?;
        SemanticScholarParser::parse_search_result(&response)
    }

    /// Get paper details by ID
    ///
    /// # Arguments
    /// - `paper_id` - Paper ID (Semantic Scholar ID or external IDs like DOI, ArXiv ID)
    ///
    /// # Returns
    /// Full paper details
    pub async fn get_paper(&self, paper_id: &str) -> ExchangeResult<ScholarPaper> {
        let path = SemanticScholarEndpoint::PaperDetails.path_with_id(paper_id);

        let mut params = HashMap::new();
        params.insert(
            "fields".to_string(),
            "paperId,title,abstract,year,citationCount,referenceCount,influentialCitationCount,venue,url,authors,fieldsOfStudy,publicationDate".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::PaperDetails, Some(path), params).await?;
        SemanticScholarParser::parse_paper(&response)
    }

    /// Get citations for a paper
    ///
    /// # Arguments
    /// - `paper_id` - Paper ID
    /// - `limit` - Number of citations (1-1000, default 100)
    /// - `offset` - Pagination offset (default 0)
    ///
    /// # Returns
    /// List of papers that cite this paper
    pub async fn get_paper_citations(
        &self,
        paper_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<ScholarCitation>> {
        let path = SemanticScholarEndpoint::PaperCitations.path_with_id(paper_id);

        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        params.insert(
            "fields".to_string(),
            "citingPaper,isInfluential".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::PaperCitations, Some(path), params).await?;
        SemanticScholarParser::parse_citations(&response)
    }

    /// Get references for a paper
    ///
    /// # Arguments
    /// - `paper_id` - Paper ID
    /// - `limit` - Number of references (1-1000, default 100)
    /// - `offset` - Pagination offset (default 0)
    ///
    /// # Returns
    /// List of papers that this paper cites
    pub async fn get_paper_references(
        &self,
        paper_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<ScholarCitation>> {
        let path = SemanticScholarEndpoint::PaperReferences.path_with_id(paper_id);

        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        params.insert(
            "fields".to_string(),
            "citingPaper,isInfluential".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::PaperReferences, Some(path), params).await?;
        SemanticScholarParser::parse_citations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AUTHOR ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for authors by query
    ///
    /// # Arguments
    /// - `query` - Author name search query (required)
    /// - `limit` - Number of results (1-100, default 10)
    /// - `offset` - Pagination offset (default 0)
    ///
    /// # Returns
    /// List of matching authors
    pub async fn search_authors(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<ScholarAuthor>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        params.insert(
            "fields".to_string(),
            "authorId,name,hIndex,citationCount,paperCount".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::AuthorSearch, None, params).await?;

        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data.iter()
            .filter_map(|a| SemanticScholarParser::parse_author(a).ok())
            .collect::<Vec<_>>())
    }

    /// Get author details by ID
    ///
    /// # Arguments
    /// - `author_id` - Author ID
    ///
    /// # Returns
    /// Full author details
    pub async fn get_author(&self, author_id: &str) -> ExchangeResult<ScholarAuthor> {
        let path = SemanticScholarEndpoint::AuthorDetails.path_with_id(author_id);

        let mut params = HashMap::new();
        params.insert(
            "fields".to_string(),
            "authorId,name,hIndex,citationCount,paperCount".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::AuthorDetails, Some(path), params).await?;
        SemanticScholarParser::parse_author(&response)
    }

    /// Get author's papers
    ///
    /// # Arguments
    /// - `author_id` - Author ID
    /// - `limit` - Number of papers (1-1000, default 100)
    /// - `offset` - Pagination offset (default 0)
    ///
    /// # Returns
    /// List of papers by this author
    pub async fn get_author_papers(
        &self,
        author_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<ScholarPaper>> {
        let path = SemanticScholarEndpoint::AuthorPapers.path_with_id(author_id);

        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        params.insert(
            "fields".to_string(),
            "paperId,title,abstract,year,citationCount,referenceCount,influentialCitationCount,venue,url,authors,fieldsOfStudy,publicationDate".to_string(),
        );

        let response = self.get(SemanticScholarEndpoint::AuthorPapers, Some(path), params).await?;

        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data.iter()
            .filter_map(|p| SemanticScholarParser::parse_paper(p).ok())
            .collect::<Vec<_>>())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS (Domain-specific searches)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get finance research papers
    ///
    /// Convenience method: fieldsOfStudy=Economics, query=finance
    pub async fn get_finance_research(&self, limit: Option<u32>) -> ExchangeResult<ScholarSearchResult> {
        self.search_papers("finance", limit, None, None, Some("Economics")).await
    }

    /// Get machine learning research papers
    ///
    /// Convenience method: fieldsOfStudy=Computer Science, query=machine learning
    pub async fn get_ml_research(&self, limit: Option<u32>) -> ExchangeResult<ScholarSearchResult> {
        self.search_papers("machine learning", limit, None, None, Some("Computer Science")).await
    }

    /// Get algorithmic trading papers
    ///
    /// Convenience method: query=algorithmic trading
    pub async fn get_trading_papers(&self, limit: Option<u32>) -> ExchangeResult<ScholarSearchResult> {
        self.search_papers("algorithmic trading", limit, None, None, None).await
    }
}
