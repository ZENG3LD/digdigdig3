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

// ═══════════════════════════════════════════════════════════════════════════
// C7 ADDITIONS — Wikimedia Analytics REST API (different base URL)
// ═══════════════════════════════════════════════════════════════════════════

/// Wikimedia Analytics API base URL
pub const WIKIMEDIA_ANALYTICS_BASE: &str = "https://wikimedia.org/api/rest_v1/metrics";

/// Build URL for unique devices endpoint
pub fn build_unique_devices_url(
    base: &str,
    project: &str,
    access_site: &str,
    granularity: &str,
    start: &str,
    end: &str,
) -> String {
    format!(
        "{}/unique-devices/{}/{}/{}/{}/{}",
        base, project, access_site, granularity, start, end
    )
}

/// Build URL for edits endpoint
pub fn build_edits_url(
    base: &str,
    project: &str,
    editor_type: &str,
    page_type: &str,
    granularity: &str,
    start: &str,
    end: &str,
) -> String {
    format!(
        "{}/edits/aggregate/{}/{}/{}/{}/{}/{}",
        base, project, editor_type, page_type, granularity, start, end
    )
}

/// Build URL for editors endpoint
pub fn build_editors_url(
    base: &str,
    project: &str,
    editor_type: &str,
    page_type: &str,
    activity_level: &str,
    granularity: &str,
    start: &str,
    end: &str,
) -> String {
    format!(
        "{}/editors/aggregate/{}/{}/{}/{}/{}/{}/{}",
        base, project, editor_type, page_type, activity_level, granularity, start, end
    )
}

/// Build URL for registered users endpoint
pub fn build_registered_users_url(
    base: &str,
    project: &str,
    granularity: &str,
    start: &str,
    end: &str,
) -> String {
    format!(
        "{}/registered-users/new/{}/{}/{}/{}",
        base, project, granularity, start, end
    )
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
