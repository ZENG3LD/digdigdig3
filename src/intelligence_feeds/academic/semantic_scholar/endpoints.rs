//! Semantic Scholar API endpoints

/// Base URLs for Semantic Scholar API
pub struct SemanticScholarEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SemanticScholarEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.semanticscholar.org/graph/v1",
            ws_base: None, // Semantic Scholar does not support WebSocket
        }
    }
}

/// Semantic Scholar API endpoint enum
#[derive(Debug, Clone)]
pub enum SemanticScholarEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // PAPER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search for papers by query
    PaperSearch,
    /// Get paper details by ID
    PaperDetails,
    /// Get citations for a paper
    PaperCitations,
    /// Get references for a paper
    PaperReferences,
    /// Batch paper lookup
    PaperBatch,

    // ═══════════════════════════════════════════════════════════════════════
    // AUTHOR ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search for authors by query
    AuthorSearch,
    /// Get author details by ID
    AuthorDetails,
    /// Get author's papers
    AuthorPapers,
}

impl SemanticScholarEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Paper endpoints
            Self::PaperSearch => "/paper/search",
            Self::PaperDetails => "/paper", // Will append /{paperId}
            Self::PaperCitations => "/paper", // Will append /{paperId}/citations
            Self::PaperReferences => "/paper", // Will append /{paperId}/references
            Self::PaperBatch => "/paper/batch",

            // Author endpoints
            Self::AuthorSearch => "/author/search",
            Self::AuthorDetails => "/author", // Will append /{authorId}
            Self::AuthorPapers => "/author", // Will append /{authorId}/papers
        }
    }

    /// Build full path with ID (for endpoints that need it)
    pub fn path_with_id(&self, id: &str) -> String {
        match self {
            Self::PaperDetails => format!("/paper/{}", id),
            Self::PaperCitations => format!("/paper/{}/citations", id),
            Self::PaperReferences => format!("/paper/{}/references", id),
            Self::AuthorDetails => format!("/author/{}", id),
            Self::AuthorPapers => format!("/author/{}/papers", id),
            _ => self.path().to_string(),
        }
    }
}
