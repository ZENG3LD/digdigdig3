//! Academic research and papers

pub mod arxiv;
pub mod semantic_scholar;

pub use arxiv::{ArxivConnector, ArxivAuth};
pub use semantic_scholar::{SemanticScholarConnector, SemanticScholarAuth};
