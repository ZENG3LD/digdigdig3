//! GDELT (Global Database of Events, Language, and Tone) API connector
//!
//! Provides access to:
//! - **DOC API**: News article search and timeline analysis
//! - **GEO API**: Geographic event mapping (GeoJSON)
//! - **TV API**: Television monitoring and analysis
//! - **CONTEXT API**: Contextual information retrieval
//!
//! # Authentication
//! GDELT API is completely public and requires no authentication.
//!
//! # Example Usage
//! ```ignore
//! use connectors_v5::data_feeds::gdelt::{GdeltConnector, DocMode, SortOrder};
//!
//! let connector = GdeltConnector::new();
//!
//! // Search for recent articles about a topic
//! let articles = connector.search_articles(
//!     "artificial intelligence",
//!     DocMode::ArtList,
//!     Some("2024-01-01"),
//!     Some("2024-01-31"),
//!     Some(100),
//!     Some(SortOrder::DateDesc),
//! ).await?;
//!
//! // Get sentiment timeline
//! let sentiment = connector.get_sentiment_timeline(
//!     "Federal Reserve",
//!     Some("2024-01-01"),
//!     None,
//! ).await?;
//!
//! // Get conflict events for a country
//! let conflicts = connector.get_conflict_events("Ukraine", None, None).await?;
//! ```

pub mod endpoints;
pub mod auth;
pub mod parser;
pub mod connector;

pub use endpoints::{
    GdeltEndpoints, GdeltEndpoint, DocMode, GeoMode, TvMode, SortOrder, format_gdelt_datetime,
};
pub use auth::GdeltAuth;
pub use parser::{
    GdeltParser, GdeltArticle, TimelinePoint, GdeltGeoResponse, GeoFeature, GeoGeometry,
    TvClip, ContextResponse,
};
pub use connector::GdeltConnector;
