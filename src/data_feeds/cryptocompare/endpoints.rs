//! CryptoCompare API endpoints

use crate::core::types::Symbol;

/// Base URLs for CryptoCompare API
pub struct CryptoCompareEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CryptoCompareEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://min-api.cryptocompare.com",
            ws_base: Some("wss://streamer.cryptocompare.com/v2"),
        }
    }
}

/// API endpoint enum
#[derive(Debug, Clone, Copy)]
pub enum CryptoCompareEndpoint {
    // === CURRENT PRICE DATA ===
    /// Single symbol current price: /data/price
    Price,
    /// Multiple symbols price matrix: /data/pricemulti
    PriceMulti,
    /// Full price data (OHLCV, volume, market cap): /data/pricemultifull
    PriceMultiFull,
    /// Daily average price: /data/dayAvg
    DayAvg,

    // === HISTORICAL DATA ===
    /// Historical price at timestamp: /data/pricehistorical
    PriceHistorical,
    /// Daily OHLCV bars: /data/histoday
    HistoDay,
    /// Hourly OHLCV bars: /data/histohour
    HistoHour,
    /// Minute OHLCV bars: /data/histominute
    HistoMinute,
    /// Daily OHLCV bars v2: /data/v2/histoday
    HistoDayV2,
    /// Hourly OHLCV bars v2: /data/v2/histohour
    HistoHourV2,
    /// Minute OHLCV bars v2: /data/v2/histominute
    HistoMinuteV2,

    // === EXCHANGE & TRADING PAIRS ===
    /// Top exchanges by volume: /data/top/exchanges
    TopExchanges,
    /// Top exchanges full data: /data/top/exchanges/full
    TopExchangesFull,
    /// Top pairs by volume: /data/top/pairs
    TopPairs,
    /// Top coins by volume: /data/top/volumes
    TopVolumes,
    /// Top coins by market cap: /data/top/mktcapfull
    TopMktCapFull,
    /// Top coins by total volume: /data/top/totalvolfull
    TopTotalVolFull,

    // === METADATA & REFERENCE DATA ===
    /// All coins list: /data/all/coinlist
    CoinList,
    /// All exchanges and pairs: /data/all/exchanges
    ExchangeList,
    /// Blockchain list: /data/blockchain/list
    BlockchainList,
    /// Blockchain daily stats: /data/blockchain/histo/day
    BlockchainHistoDay,
    /// Latest blockchain data: /data/blockchain/latest
    BlockchainLatest,

    // === NEWS & SOCIAL ===
    /// Latest news articles: /data/v2/news/
    News,
    /// News feeds list: /data/news/feeds
    NewsFeeds,
    /// News categories: /data/news/categories
    NewsCategories,
    /// Latest social stats: /data/social/coin/latest
    SocialLatest,
    /// Historical social stats (daily): /data/social/coin/histo/day
    SocialHistoDay,
    /// Historical social stats (hourly): /data/social/coin/histo/hour
    SocialHistoHour,

    // === ACCOUNT & API MANAGEMENT ===
    /// Rate limit status: /stats/rate/limit
    RateLimit,
    /// Hourly rate limit: /stats/rate/hour/limit
    RateLimitHour,
}

impl CryptoCompareEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Current Price Data
            Self::Price => "/data/price",
            Self::PriceMulti => "/data/pricemulti",
            Self::PriceMultiFull => "/data/pricemultifull",
            Self::DayAvg => "/data/dayAvg",

            // Historical Data
            Self::PriceHistorical => "/data/pricehistorical",
            Self::HistoDay => "/data/histoday",
            Self::HistoHour => "/data/histohour",
            Self::HistoMinute => "/data/histominute",
            Self::HistoDayV2 => "/data/v2/histoday",
            Self::HistoHourV2 => "/data/v2/histohour",
            Self::HistoMinuteV2 => "/data/v2/histominute",

            // Exchange & Trading Pairs
            Self::TopExchanges => "/data/top/exchanges",
            Self::TopExchangesFull => "/data/top/exchanges/full",
            Self::TopPairs => "/data/top/pairs",
            Self::TopVolumes => "/data/top/volumes",
            Self::TopMktCapFull => "/data/top/mktcapfull",
            Self::TopTotalVolFull => "/data/top/totalvolfull",

            // Metadata & Reference Data
            Self::CoinList => "/data/all/coinlist",
            Self::ExchangeList => "/data/all/exchanges",
            Self::BlockchainList => "/data/blockchain/list",
            Self::BlockchainHistoDay => "/data/blockchain/histo/day",
            Self::BlockchainLatest => "/data/blockchain/latest",

            // News & Social
            Self::News => "/data/v2/news/",
            Self::NewsFeeds => "/data/news/feeds",
            Self::NewsCategories => "/data/news/categories",
            Self::SocialLatest => "/data/social/coin/latest",
            Self::SocialHistoDay => "/data/social/coin/histo/day",
            Self::SocialHistoHour => "/data/social/coin/histo/hour",

            // Account & API Management
            Self::RateLimit => "/stats/rate/limit",
            Self::RateLimitHour => "/stats/rate/hour/limit",
        }
    }

    /// Whether endpoint requires API key
    pub fn requires_api_key(&self) -> bool {
        match self {
            // Endpoints that REQUIRE API key
            Self::News
            | Self::SocialLatest
            | Self::SocialHistoDay
            | Self::SocialHistoHour
            | Self::RateLimit
            | Self::RateLimitHour => true,

            // All other endpoints are optional (but recommended for rate limits)
            _ => false,
        }
    }
}

/// Format symbol for CryptoCompare API
///
/// CryptoCompare uses separate `fsym` (from symbol) and `tsym` (to symbol) parameters.
/// - Base currency: BTC, ETH, etc.
/// - Quote currency: USD, USDT, EUR, etc.
///
/// # Examples
/// - BTC/USDT -> fsym=BTC, tsym=USDT
/// - ETH/BTC -> fsym=ETH, tsym=BTC
///
/// # Note
/// CryptoCompare expects uppercase symbols without delimiters.
pub fn format_symbol(symbol: &Symbol) -> (String, String) {
    (
        symbol.base.to_uppercase(),
        symbol.quote.to_uppercase(),
    )
}

/// Map interval string to CryptoCompare aggregation parameter
///
/// CryptoCompare supports aggregation of bars. For example:
/// - aggregate=1 -> 1-minute bars
/// - aggregate=3 -> 3-minute bars (groups 3x 1-minute bars)
/// - aggregate=7 -> 7-day bars (groups 7x daily bars)
///
/// This function returns a sensible default aggregation for common intervals.
pub fn map_interval_aggregate(interval: &str) -> (CryptoCompareEndpoint, u32) {
    match interval {
        // Minute intervals (use histominute)
        "1m" => (CryptoCompareEndpoint::HistoMinute, 1),
        "3m" => (CryptoCompareEndpoint::HistoMinute, 3),
        "5m" => (CryptoCompareEndpoint::HistoMinute, 5),
        "15m" => (CryptoCompareEndpoint::HistoMinute, 15),
        "30m" => (CryptoCompareEndpoint::HistoMinute, 30),

        // Hourly intervals (use histohour)
        "1h" => (CryptoCompareEndpoint::HistoHour, 1),
        "2h" => (CryptoCompareEndpoint::HistoHour, 2),
        "4h" => (CryptoCompareEndpoint::HistoHour, 4),
        "6h" => (CryptoCompareEndpoint::HistoHour, 6),
        "8h" => (CryptoCompareEndpoint::HistoHour, 8),
        "12h" => (CryptoCompareEndpoint::HistoHour, 12),

        // Daily intervals (use histoday)
        "1d" => (CryptoCompareEndpoint::HistoDay, 1),
        "1w" => (CryptoCompareEndpoint::HistoDay, 7),
        "1M" => (CryptoCompareEndpoint::HistoDay, 30),

        // Default: 1 hour
        _ => (CryptoCompareEndpoint::HistoHour, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        let symbol = Symbol {
            base: "btc".to_string(),
            quote: "usdt".to_string(),
        };

        let (fsym, tsym) = format_symbol(&symbol);
        assert_eq!(fsym, "BTC");
        assert_eq!(tsym, "USDT");
    }

    #[test]
    fn test_map_interval_aggregate() {
        let (endpoint, aggregate) = map_interval_aggregate("5m");
        assert_eq!(aggregate, 5);
        assert!(matches!(endpoint, CryptoCompareEndpoint::HistoMinute));

        let (endpoint, aggregate) = map_interval_aggregate("1h");
        assert_eq!(aggregate, 1);
        assert!(matches!(endpoint, CryptoCompareEndpoint::HistoHour));

        let (endpoint, aggregate) = map_interval_aggregate("1d");
        assert_eq!(aggregate, 1);
        assert!(matches!(endpoint, CryptoCompareEndpoint::HistoDay));
    }
}
