//! C2IntelFeeds API endpoints

/// Base URLs for C2IntelFeeds (GitHub raw files)
pub struct C2IntelFeedsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for C2IntelFeedsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds",
            ws_base: None, // C2IntelFeeds does not support WebSocket
        }
    }
}

/// C2IntelFeeds endpoint enum (feed files)
#[derive(Debug, Clone)]
pub enum C2IntelFeedsEndpoint {
    /// All-time IP C2 indicators
    IpFeedAll,
    /// Last 30 days IP C2 indicators
    IpFeed30Day,
    /// Last 7 days IP C2 indicators (if available)
    IpFeed7Day,
    /// Last 90 days IP C2 indicators
    IpFeed90Day,
    /// All-time domain C2 indicators
    DomainFeed,
    /// Last 30 days domain C2 indicators
    DomainFeed30Day,
    /// Last 90 days domain C2 indicators
    DomainFeed90Day,
}

impl C2IntelFeedsEndpoint {
    /// Get endpoint path (CSV file path)
    pub fn path(&self) -> &'static str {
        match self {
            Self::IpFeedAll => "/IPC2s.csv",
            Self::IpFeed30Day => "/IPC2s-30day.csv",
            Self::IpFeed7Day => "/IPC2s-7day.csv",
            Self::IpFeed90Day => "/IPC2s-90day.csv",
            Self::DomainFeed => "/domainC2s.csv",
            Self::DomainFeed30Day => "/domainC2s-30day.csv",
            Self::DomainFeed90Day => "/domainC2s-90day.csv",
        }
    }
}
