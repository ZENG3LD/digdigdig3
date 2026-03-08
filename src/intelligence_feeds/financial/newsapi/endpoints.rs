//! NewsAPI.org endpoints

/// Base URLs for NewsAPI
pub struct NewsApiEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for NewsApiEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://newsapi.org/v2",
            ws_base: None, // NewsAPI does not support WebSocket
        }
    }
}

/// NewsAPI endpoint enum
#[derive(Debug, Clone)]
pub enum NewsApiEndpoint {
    /// Get top headlines from a country/category
    TopHeadlines,
    /// Search everything
    Everything,
    /// Get news sources
    Sources,
}

impl NewsApiEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::TopHeadlines => "/top-headlines",
            Self::Everything => "/everything",
            Self::Sources => "/top-headlines/sources",
        }
    }
}

/// NewsAPI categories
#[derive(Debug, Clone, Copy)]
pub enum NewsCategory {
    Business,
    Entertainment,
    General,
    Health,
    Science,
    Sports,
    Technology,
}

impl NewsCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Business => "business",
            Self::Entertainment => "entertainment",
            Self::General => "general",
            Self::Health => "health",
            Self::Science => "science",
            Self::Sports => "sports",
            Self::Technology => "technology",
        }
    }
}

/// NewsAPI languages
#[derive(Debug, Clone, Copy)]
pub enum NewsLanguage {
    Arabic,
    German,
    English,
    Spanish,
    French,
    Hebrew,
    Italian,
    Dutch,
    Norwegian,
    Portuguese,
    Russian,
    Swedish,
    Urdu,
    Chinese,
}

impl NewsLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Arabic => "ar",
            Self::German => "de",
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::Hebrew => "he",
            Self::Italian => "it",
            Self::Dutch => "nl",
            Self::Norwegian => "no",
            Self::Portuguese => "pt",
            Self::Russian => "ru",
            Self::Swedish => "sv",
            Self::Urdu => "ud",
            Self::Chinese => "zh",
        }
    }
}

/// NewsAPI sort options for /everything endpoint
#[derive(Debug, Clone, Copy)]
pub enum NewsSortBy {
    /// Articles more closely related to search query come first
    Relevancy,
    /// Articles from popular sources and publishers come first
    Popularity,
    /// Newest articles come first
    PublishedAt,
}

impl NewsSortBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Relevancy => "relevancy",
            Self::Popularity => "popularity",
            Self::PublishedAt => "publishedAt",
        }
    }
}
