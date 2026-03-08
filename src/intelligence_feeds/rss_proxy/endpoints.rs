//! RSS Feed Proxy endpoints

/// RSS Feed Proxy endpoints
pub struct RssProxyEndpoints;

impl RssProxyEndpoints {
    /// No base URL needed - we fetch feeds directly from their sources
    pub fn new() -> Self {
        Self
    }
}

impl Default for RssProxyEndpoints {
    fn default() -> Self {
        Self::new()
    }
}

/// RSS Feed endpoint enum
#[derive(Debug, Clone)]
pub enum RssProxyEndpoint {
    /// Direct feed URL
    Feed { url: String },
}

impl RssProxyEndpoint {
    /// Get the URL for this endpoint
    pub fn url(&self) -> &str {
        match self {
            Self::Feed { url } => url,
        }
    }
}

// =============================================================================
// POPULAR FEED URLS
// =============================================================================

/// Top news sources (general news)
pub mod news_sources {
    /// BBC World News
    pub const BBC_WORLD: &str = "http://feeds.bbci.co.uk/news/world/rss.xml";

    /// BBC Technology
    pub const BBC_TECH: &str = "http://feeds.bbci.co.uk/news/technology/rss.xml";

    /// BBC Business
    pub const BBC_BUSINESS: &str = "http://feeds.bbci.co.uk/news/business/rss.xml";

    /// Reuters World News
    pub const REUTERS_WORLD: &str = "https://www.reutersagency.com/feed/";

    /// Reuters Business
    pub const REUTERS_BUSINESS: &str = "https://www.reuters.com/business/feed/";

    /// Reuters Technology
    pub const REUTERS_TECH: &str = "https://www.reuters.com/technology/feed/";

    /// NPR News
    pub const NPR_NEWS: &str = "https://feeds.npr.org/1001/rss.xml";

    /// NPR Business
    pub const NPR_BUSINESS: &str = "https://feeds.npr.org/1006/rss.xml";

    /// NPR Technology
    pub const NPR_TECH: &str = "https://feeds.npr.org/1019/rss.xml";

    /// The Guardian World
    pub const GUARDIAN_WORLD: &str = "https://www.theguardian.com/world/rss";

    /// The Guardian Business
    pub const GUARDIAN_BUSINESS: &str = "https://www.theguardian.com/business/rss";

    /// The Guardian Technology
    pub const GUARDIAN_TECH: &str = "https://www.theguardian.com/technology/rss";

    /// Al Jazeera
    pub const ALJAZEERA: &str = "https://www.aljazeera.com/xml/rss/all.xml";

    /// CNN Top Stories
    pub const CNN_TOP: &str = "http://rss.cnn.com/rss/cnn_topstories.rss";

    /// CNN World
    pub const CNN_WORLD: &str = "http://rss.cnn.com/rss/cnn_world.rss";

    /// CNN Business
    pub const CNN_BUSINESS: &str = "http://rss.cnn.com/rss/money_latest.rss";
}

/// Technology news sources
pub mod tech_sources {
    /// TechCrunch
    pub const TECHCRUNCH: &str = "https://techcrunch.com/feed/";

    /// Ars Technica
    pub const ARS_TECHNICA: &str = "https://feeds.arstechnica.com/arstechnica/index";

    /// The Verge
    pub const THE_VERGE: &str = "https://www.theverge.com/rss/index.xml";

    /// Wired
    pub const WIRED: &str = "https://www.wired.com/feed/rss";

    /// Hacker News
    pub const HACKER_NEWS: &str = "https://news.ycombinator.com/rss";

    /// MIT Technology Review
    pub const MIT_TECH_REVIEW: &str = "https://www.technologyreview.com/feed/";

    /// ZDNet
    pub const ZDNET: &str = "https://www.zdnet.com/news/rss.xml";
}

/// Policy, think tanks, and intelligence sources
pub mod policy_sources {
    /// CSIS (Center for Strategic and International Studies)
    pub const CSIS: &str = "https://www.csis.org/analysis/feed";

    /// Brookings Institution
    pub const BROOKINGS: &str = "https://www.brookings.edu/feed/";

    /// Carnegie Endowment
    pub const CARNEGIE: &str = "https://carnegieendowment.org/feed";

    /// Council on Foreign Relations
    pub const CFR: &str = "https://www.cfr.org/feeds/blog.xml";

    /// RAND Corporation
    pub const RAND: &str = "https://www.rand.org/news.xml";

    /// War on the Rocks
    pub const WAR_ON_ROCKS: &str = "https://warontherocks.com/feed/";
}

/// Financial news sources
pub mod finance_sources {
    /// Financial Times
    pub const FT: &str = "https://www.ft.com/?format=rss";

    /// Bloomberg Markets
    pub const BLOOMBERG: &str = "https://feeds.bloomberg.com/markets/news.rss";

    /// MarketWatch
    pub const MARKETWATCH: &str = "http://feeds.marketwatch.com/marketwatch/topstories/";

    /// Seeking Alpha
    pub const SEEKING_ALPHA: &str = "https://seekingalpha.com/feed.xml";
}

/// Cybersecurity sources
pub mod cyber_sources {
    /// Krebs on Security
    pub const KREBS: &str = "https://krebsonsecurity.com/feed/";

    /// Schneier on Security
    pub const SCHNEIER: &str = "https://www.schneier.com/blog/atom.xml";

    /// Dark Reading
    pub const DARK_READING: &str = "https://www.darkreading.com/rss_simple.asp";

    /// The Hacker News
    pub const HACKER_NEWS_CYBER: &str = "https://thehackernews.com/feeds/posts/default";
}

/// All top news sources (general category)
pub fn all_news() -> Vec<&'static str> {
    vec![
        news_sources::BBC_WORLD,
        news_sources::REUTERS_WORLD,
        news_sources::NPR_NEWS,
        news_sources::GUARDIAN_WORLD,
        news_sources::ALJAZEERA,
        news_sources::CNN_TOP,
    ]
}

/// All tech sources
pub fn all_tech() -> Vec<&'static str> {
    vec![
        tech_sources::TECHCRUNCH,
        tech_sources::ARS_TECHNICA,
        tech_sources::THE_VERGE,
        tech_sources::WIRED,
        tech_sources::MIT_TECH_REVIEW,
    ]
}

/// All policy/think tank sources
pub fn all_policy() -> Vec<&'static str> {
    vec![
        policy_sources::CSIS,
        policy_sources::BROOKINGS,
        policy_sources::CARNEGIE,
        policy_sources::CFR,
        policy_sources::RAND,
        policy_sources::WAR_ON_ROCKS,
    ]
}

/// All finance sources
pub fn all_finance() -> Vec<&'static str> {
    vec![
        finance_sources::FT,
        finance_sources::BLOOMBERG,
        finance_sources::MARKETWATCH,
        finance_sources::SEEKING_ALPHA,
    ]
}

/// All cyber sources
pub fn all_cyber() -> Vec<&'static str> {
    vec![
        cyber_sources::KREBS,
        cyber_sources::SCHNEIER,
        cyber_sources::DARK_READING,
        cyber_sources::HACKER_NEWS_CYBER,
    ]
}
