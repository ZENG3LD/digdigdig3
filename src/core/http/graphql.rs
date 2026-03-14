//! # GraphQL Client
//!
//! Thin wrapper over `HttpClient` for GraphQL queries.
//! GraphQL uses HTTP POST with `{"query": "...", "variables": {...}}` body.
//! No new dependencies needed — reuses the existing HTTP transport.

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::core::types::{ExchangeResult};
use super::client::HttpClient;

/// GraphQL client — thin wrapper over `HttpClient`.
///
/// GraphQL queries are plain HTTP POST requests with a JSON body containing
/// `query` and optional `variables`. This struct handles that encoding and
/// delegates to `HttpClient` for transport, retry, and error mapping.
///
/// # Example
///
/// ```rust,no_run
/// # use digdigdig3::core::http::{HttpClient, GraphQlClient};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let http = HttpClient::new(10_000)?;
/// let client = GraphQlClient::new(http, "https://api.example.com/graphql");
///
/// let result = client
///     .query("{ markets { symbol price } }", None)
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct GraphQlClient {
    http: HttpClient,
    endpoint: String,
}

impl GraphQlClient {
    /// Create a new GraphQL client wrapping an `HttpClient`.
    ///
    /// `endpoint` is the full URL to the GraphQL endpoint
    /// (e.g. `"https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3"`).
    pub fn new(http: HttpClient, endpoint: &str) -> Self {
        Self {
            http,
            endpoint: endpoint.to_string(),
        }
    }

    /// Execute a GraphQL query or mutation.
    ///
    /// Sends a POST request with `{"query": query, "variables": variables}`.
    /// Returns the raw JSON response (callers inspect `data` / `errors` fields
    /// themselves, since GraphQL error shapes vary by provider).
    pub async fn query(&self, query: &str, variables: Option<Value>) -> ExchangeResult<Value> {
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });
        let empty_headers = HashMap::new();
        self.http.post(&self.endpoint, &body, &empty_headers).await
    }

    /// Execute a GraphQL query with additional HTTP headers.
    ///
    /// Use this when the endpoint requires authentication headers
    /// (e.g. `Authorization: Bearer <token>` or `X-Api-Key: <key>`).
    pub async fn query_with_headers(
        &self,
        query: &str,
        variables: Option<Value>,
        headers: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });
        self.http.post(&self.endpoint, &body, &headers).await
    }

    /// Return a reference to the underlying `HttpClient`.
    ///
    /// Useful when a connector needs to mix direct REST calls alongside
    /// GraphQL queries on the same transport.
    pub fn http(&self) -> &HttpClient {
        &self.http
    }
}
