//! Hacker News Firebase API endpoints

/// Base URLs for Hacker News API
pub struct HackerNewsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for HackerNewsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://hacker-news.firebaseio.com/v0",
            ws_base: None, // Hacker News does not require WebSocket for basic feed
        }
    }
}

/// Hacker News API endpoint enum
#[derive(Debug, Clone)]
pub enum HackerNewsEndpoint {
    /// Top stories (up to 500)
    TopStories,
    /// New stories (up to 500)
    NewStories,
    /// Best stories (up to 500)
    BestStories,
    /// Ask HN stories (up to 200)
    AskStories,
    /// Show HN stories (up to 200)
    ShowStories,
    /// Job stories (up to 200)
    JobStories,
    /// Get specific item by ID
    Item { id: u64 },
    /// Get user by username
    User { id: String },
    /// Get max item ID
    MaxItem,
    /// Get recent updates (items and profiles)
    Updates,
}

impl HackerNewsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::TopStories => "/topstories.json".to_string(),
            Self::NewStories => "/newstories.json".to_string(),
            Self::BestStories => "/beststories.json".to_string(),
            Self::AskStories => "/askstories.json".to_string(),
            Self::ShowStories => "/showstories.json".to_string(),
            Self::JobStories => "/jobstories.json".to_string(),
            Self::Item { id } => format!("/item/{}.json", id),
            Self::User { id } => format!("/user/{}.json", id),
            Self::MaxItem => "/maxitem.json".to_string(),
            Self::Updates => "/updates.json".to_string(),
        }
    }
}
