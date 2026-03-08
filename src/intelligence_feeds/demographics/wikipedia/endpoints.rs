//! Wikipedia Pageviews API endpoints

/// Base URLs for Wikipedia Pageviews API
pub struct WikipediaEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for WikipediaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://wikimedia.org/api/rest_v1/metrics/pageviews",
            ws_base: None, // Wikipedia Pageviews does not support WebSocket
        }
    }
}

/// Wikipedia Pageviews API endpoint enum
#[derive(Debug, Clone)]
pub enum WikipediaEndpoint {
    /// Get pageviews for a specific article
    /// /per-article/{project}/{access}/{agent}/{article}/{granularity}/{start}/{end}
    PerArticle,

    /// Get aggregate pageviews for entire project
    /// /aggregate/{project}/{access}/{agent}/{granularity}/{start}/{end}
    Aggregate,

    /// Get most viewed articles for a date
    /// /top/{project}/{access}/{year}/{month}/{day}
    Top,

    /// Get pageviews by country
    /// /top-per-country/{project}/{access}/{year}/{month}
    TopPerCountry,
}

impl WikipediaEndpoint {
    /// Get base endpoint path (without parameters)
    pub fn base_path(&self) -> &'static str {
        match self {
            Self::PerArticle => "/per-article",
            Self::Aggregate => "/aggregate",
            Self::Top => "/top",
            Self::TopPerCountry => "/top-per-country",
        }
    }
}

/// Build full URL for per-article endpoint
#[allow(clippy::too_many_arguments)]
pub fn build_per_article_url(
    base: &str,
    project: &str,
    access: &str,
    agent: &str,
    article: &str,
    granularity: &str,
    start: &str,
    end: &str,
) -> String {
    format!(
        "{}/per-article/{}/{}/{}/{}/{}/{}/{}",
        base, project, access, agent, article, granularity, start, end
    )
}

/// Build full URL for aggregate endpoint
pub fn build_aggregate_url(
    base: &str,
    project: &str,
    access: &str,
    agent: &str,
    granularity: &str,
    start: &str,
    end: &str,
) -> String {
    format!(
        "{}/aggregate/{}/{}/{}/{}/{}/{}",
        base, project, access, agent, granularity, start, end
    )
}

/// Build full URL for top articles endpoint
pub fn build_top_url(
    base: &str,
    project: &str,
    access: &str,
    year: &str,
    month: &str,
    day: &str,
) -> String {
    format!(
        "{}/top/{}/{}/{}/{}/{}",
        base, project, access, year, month, day
    )
}

/// Build full URL for top-per-country endpoint
pub fn build_top_per_country_url(
    base: &str,
    project: &str,
    access: &str,
    year: &str,
    month: &str,
) -> String {
    format!(
        "{}/top-per-country/{}/{}/{}/{}",
        base, project, access, year, month
    )
}
