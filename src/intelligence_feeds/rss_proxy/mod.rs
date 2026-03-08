//! # RSS Feed Proxy (News Aggregator) Connector
//!
//! Category: data_feeds
//! Type: RSS/Atom Feed Aggregator
//!
//! ## Features
//! - REST API: N/A (direct XML fetching)
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (public RSS feeds)
//!
//! ## Data Types
//! - RSS 2.0: Yes
//! - Atom 1.0: Yes
//! - News articles: Yes
//! - Blog posts: Yes
//! - Metadata: Yes (titles, links, descriptions, dates, authors, categories)
//!
//! ## Key Capabilities
//! - Fetch any RSS/Atom feed by URL
//! - Parse RSS 2.0 and Atom 1.0 formats
//! - Pre-configured access to 100+ popular news sources
//! - Batch fetching (concurrent)
//! - Item aggregation across multiple feeds
//!
//! ## Data Sources
//! - **News**: BBC, Reuters, NPR, Guardian, Al Jazeera, CNN
//! - **Technology**: TechCrunch, Ars Technica, The Verge, Wired, MIT Tech Review
//! - **Policy**: CSIS, Brookings, Carnegie Endowment, CFR, RAND
//! - **Finance**: Financial Times, Bloomberg, MarketWatch, Seeking Alpha
//! - **Cybersecurity**: Krebs on Security, Schneier, Dark Reading, The Hacker News
//!
//! ## Rate Limits
//! - No enforced limits (public feeds)
//! - Recommended: 1 request per feed per minute
//! - Respect robots.txt and feed update frequency
//!
//! ## Data Coverage
//! - 100+ approved news domains
//! - Real-time updates (as feeds are published)
//! - Historical depth: Varies by source (typically last 30-100 items)
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Follow source website terms of service
//! - Attribution requested
//! - Use polite User-Agent
//!
//! ## Implementation Notes
//! - Auto-detects RSS vs Atom format
//! - Handles HTML entities in content
//! - Concurrent fetching for batch operations
//! - Graceful error handling (skips failed feeds)
//! - User-Agent identifies NEMO Terminal

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{RssProxyEndpoint, RssProxyEndpoints};
pub use auth::RssProxyAuth;
pub use parser::{
    RssProxyParser, RssFeed, RssFeedItem,
};
pub use connector::RssProxyConnector;

// Re-export popular feed URL constants for convenience
pub use endpoints::{
    news_sources, tech_sources, policy_sources, finance_sources, cyber_sources,
    all_news, all_tech, all_policy, all_finance, all_cyber,
};
