//! NewsAPI response parsers
//!
//! Parse JSON responses to domain types based on NewsAPI response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct NewsApiParser;

/// News article from NewsAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub source: NewsSource,
    pub author: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub url_to_image: Option<String>,
    pub published_at: String,
    pub content: Option<String>,
}

/// News source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    pub id: Option<String>,
    pub name: String,
}

/// News source metadata (from /sources endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSourceMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub category: String,
    pub language: String,
    pub country: String,
}

impl NewsApiParser {
    /// Check for API errors in response
    ///
    /// NewsAPI error format:
    /// ```json
    /// {
    ///   "status": "error",
    ///   "code": "apiKeyInvalid",
    ///   "message": "Your API key is invalid or incorrect."
    /// }
    /// ```
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(status) = response.get("status").and_then(|v| v.as_str()) {
            if status == "error" {
                let code = response
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");

                return Err(ExchangeError::Api {
                    code: 0,
                    message: format!("{}: {}", code, message),
                });
            }
        }
        Ok(())
    }

    /// Parse articles array from response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "ok",
    ///   "totalResults": 38,
    ///   "articles": [
    ///     {
    ///       "source": {"id": "bbc-news", "name": "BBC News"},
    ///       "author": "BBC News",
    ///       "title": "Article title",
    ///       "description": "Article description",
    ///       "url": "https://...",
    ///       "urlToImage": "https://...",
    ///       "publishedAt": "2024-01-15T10:00:00Z",
    ///       "content": "Article content..."
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_articles(response: &Value) -> ExchangeResult<Vec<NewsArticle>> {
        Self::check_error(response)?;

        let articles = response
            .get("articles")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'articles' array".to_string()))?;

        articles
            .iter()
            .map(|article| {
                let source = article
                    .get("source")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'source'".to_string()))?;

                let source_parsed = NewsSource {
                    id: source.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    name: Self::require_str(source, "name")?.to_string(),
                };

                Ok(NewsArticle {
                    source: source_parsed,
                    author: Self::get_str(article, "author").map(|s| s.to_string()),
                    title: Self::require_str(article, "title")?.to_string(),
                    description: Self::get_str(article, "description").map(|s| s.to_string()),
                    url: Self::require_str(article, "url")?.to_string(),
                    url_to_image: Self::get_str(article, "urlToImage").map(|s| s.to_string()),
                    published_at: Self::require_str(article, "publishedAt")?.to_string(),
                    content: Self::get_str(article, "content").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse sources array from /sources endpoint
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "ok",
    ///   "sources": [
    ///     {
    ///       "id": "bbc-news",
    ///       "name": "BBC News",
    ///       "description": "Use BBC News for...",
    ///       "url": "http://www.bbc.co.uk/news",
    ///       "category": "general",
    ///       "language": "en",
    ///       "country": "gb"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_sources(response: &Value) -> ExchangeResult<Vec<NewsSourceMetadata>> {
        Self::check_error(response)?;

        let sources = response
            .get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'sources' array".to_string()))?;

        sources
            .iter()
            .map(|source| {
                Ok(NewsSourceMetadata {
                    id: Self::require_str(source, "id")?.to_string(),
                    name: Self::require_str(source, "name")?.to_string(),
                    description: Self::require_str(source, "description")?.to_string(),
                    url: Self::require_str(source, "url")?.to_string(),
                    category: Self::require_str(source, "category")?.to_string(),
                    language: Self::require_str(source, "language")?.to_string(),
                    country: Self::require_str(source, "country")?.to_string(),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn _require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", field)))
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }
}
