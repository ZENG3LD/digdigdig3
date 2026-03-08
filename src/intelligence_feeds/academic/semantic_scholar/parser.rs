//! Semantic Scholar response parsers
//!
//! Parse JSON responses to domain types based on Semantic Scholar API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct SemanticScholarParser;

impl SemanticScholarParser {
    // ═══════════════════════════════════════════════════════════════════════
    // SEMANTIC SCHOLAR-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse paper search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total": 1234,
    ///   "offset": 0,
    ///   "next": 10,
    ///   "data": [
    ///     {
    ///       "paperId": "649def34f8be52c8b66281af98ae884c09aef38b",
    ///       "title": "Machine Learning for Trading",
    ///       "abstract": "...",
    ///       "year": 2020,
    ///       "citationCount": 42,
    ///       "referenceCount": 35,
    ///       "influentialCitationCount": 5,
    ///       "venue": "ICML",
    ///       "url": "https://...",
    ///       "authors": [...],
    ///       "fieldsOfStudy": ["Computer Science", "Economics"]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_search_result(response: &Value) -> ExchangeResult<ScholarSearchResult> {
        let total = response
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let offset = response
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let papers = data
            .iter()
            .filter_map(|p| Self::parse_paper(p).ok())
            .collect();

        Ok(ScholarSearchResult {
            total,
            offset,
            data: papers,
        })
    }

    /// Parse a single paper
    pub fn parse_paper(paper: &Value) -> ExchangeResult<ScholarPaper> {
        let paper_id = Self::require_str(paper, "paperId")?.to_string();
        let title = Self::require_str(paper, "title")?.to_string();

        let abstract_text = Self::get_str(paper, "abstract").map(|s| s.to_string());
        let year = Self::get_u32(paper, "year");
        let citation_count = Self::get_u32(paper, "citationCount").unwrap_or(0);
        let reference_count = Self::get_u32(paper, "referenceCount").unwrap_or(0);
        let influential_citation_count = Self::get_u32(paper, "influentialCitationCount").unwrap_or(0);
        let venue = Self::get_str(paper, "venue").map(|s| s.to_string());
        let url = Self::get_str(paper, "url")
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("https://www.semanticscholar.org/paper/{}", paper_id));
        let publication_date = Self::get_str(paper, "publicationDate").map(|s| s.to_string());

        // Parse authors
        let authors = paper
            .get("authors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| Self::parse_author(a).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Parse fields of study
        let fields_of_study = paper
            .get("fieldsOfStudy")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ScholarPaper {
            paper_id,
            title,
            abstract_text,
            year,
            citation_count,
            reference_count,
            influential_citation_count,
            venue,
            url,
            authors,
            fields_of_study,
            publication_date,
        })
    }

    /// Parse a single author
    pub fn parse_author(author: &Value) -> ExchangeResult<ScholarAuthor> {
        let author_id = Self::require_str(author, "authorId")?.to_string();
        let name = Self::require_str(author, "name")?.to_string();

        let h_index = Self::get_u32(author, "hIndex");
        let citation_count = Self::get_u32(author, "citationCount");
        let paper_count = Self::get_u32(author, "paperCount");

        Ok(ScholarAuthor {
            author_id,
            name,
            h_index,
            citation_count,
            paper_count,
        })
    }

    /// Parse citation
    pub fn parse_citation(citation: &Value) -> ExchangeResult<ScholarCitation> {
        let citing_paper = citation
            .get("citingPaper")
            .ok_or_else(|| ExchangeError::Parse("Missing 'citingPaper'".to_string()))
            .and_then(Self::parse_paper)?;

        let is_influential = Self::get_bool(citation, "isInfluential").unwrap_or(false);

        Ok(ScholarCitation {
            citing_paper,
            is_influential,
        })
    }

    /// Parse citations list
    pub fn parse_citations(response: &Value) -> ExchangeResult<Vec<ScholarCitation>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data.iter()
            .filter_map(|c| Self::parse_citation(c).ok())
            .collect::<Vec<_>>())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 400,
                message,
            });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|n| n as u32)
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SEMANTIC SCHOLAR-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Academic paper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarPaper {
    #[serde(rename = "paperId")]
    pub paper_id: String,
    pub title: String,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub year: Option<u32>,
    #[serde(rename = "citationCount")]
    pub citation_count: u32,
    #[serde(rename = "referenceCount")]
    pub reference_count: u32,
    #[serde(rename = "influentialCitationCount")]
    pub influential_citation_count: u32,
    pub venue: Option<String>,
    pub url: String,
    pub authors: Vec<ScholarAuthor>,
    #[serde(rename = "fieldsOfStudy")]
    pub fields_of_study: Vec<String>,
    #[serde(rename = "publicationDate")]
    pub publication_date: Option<String>,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarAuthor {
    #[serde(rename = "authorId")]
    pub author_id: String,
    pub name: String,
    #[serde(rename = "hIndex")]
    pub h_index: Option<u32>,
    #[serde(rename = "citationCount")]
    pub citation_count: Option<u32>,
    #[serde(rename = "paperCount")]
    pub paper_count: Option<u32>,
}

/// Search result with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarSearchResult {
    pub total: u64,
    pub offset: u64,
    pub data: Vec<ScholarPaper>,
}

/// Citation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarCitation {
    #[serde(rename = "citingPaper")]
    pub citing_paper: ScholarPaper,
    #[serde(rename = "isInfluential")]
    pub is_influential: bool,
}
