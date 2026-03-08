//! # Hacker News (Social News Feed) Connector
//!
//! Category: data_feeds
//! Type: Social News Aggregator
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No (not needed for basic feed)
//! - Authentication: None required
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Stories: Yes (standard, Ask HN, Show HN)
//! - Jobs: Yes
//! - Comments: Yes (via item endpoint)
//! - Users: Yes
//! - Real-time updates: Yes (via updates endpoint)
//!
//! ## Key Endpoints
//! - /topstories.json - Top 500 stories (HN algorithm ranked)
//! - /newstories.json - Newest 500 stories (chronological)
//! - /beststories.json - Best 500 stories (quality ranked)
//! - /askstories.json - Latest 200 Ask HN stories
//! - /showstories.json - Latest 200 Show HN stories
//! - /jobstories.json - Latest 200 job postings
//! - /item/{id}.json - Get specific item (story, comment, job)
//! - /user/{id}.json - Get user profile
//! - /maxitem.json - Current max item ID
//! - /updates.json - Recently changed items and profiles
//!
//! ## Rate Limits
//! - Official limit: None currently enforced
//! - Recommended: Limit concurrent requests to ~10 to prevent fan-out
//!
//! ## Data Coverage
//! - All Hacker News content
//! - Stories: Top, new, best, ask, show, jobs
//! - Comments: Full comment trees
//! - Users: Public profile data
//! - Historical depth: All items since HN launch (2007)
//! - Update frequency: Real-time (seconds to minutes)
//!
//! ## Usage Restrictions
//! - Free for all use
//! - No API key required
//! - No authentication needed
//! - Respect the API (don't abuse with excessive requests)
//!
//! ## Implementation Notes
//! - Story fetching is a 2-step process: (1) get ID list, (2) fetch each item
//! - Concurrent fetching implemented with max 10 requests at a time
//! - Deleted/not found items are skipped gracefully
//! - Timeout per request: 10 seconds

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{HackerNewsEndpoint, HackerNewsEndpoints};
pub use auth::HackerNewsAuth;
pub use parser::{
    HackerNewsParser, HnStory, HnUser, HnItemType, HnUpdates,
};
pub use connector::HackerNewsConnector;
