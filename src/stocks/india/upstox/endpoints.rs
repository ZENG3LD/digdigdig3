//! # Upstox Endpoints
//!
//! URL'ы и endpoint enum для Upstox API.


// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Upstox API
#[derive(Debug, Clone)]
pub struct UpstoxUrls {
    pub rest_base: &'static str,
    pub rest_hft: &'static str,
    pub rest_v3: &'static str,
    pub _ws_market_data: &'static str,
    pub _ws_portfolio: &'static str,
}

impl UpstoxUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest_base: "https://api.upstox.com/v2",
        rest_hft: "https://api-hft.upstox.com/v2",
        rest_v3: "https://api.upstox.com/v3",
        _ws_market_data: "wss://api.upstox.com/v2/feed/market-data-feed/protobuf",
        _ws_portfolio: "wss://api.upstox.com/v2/feed/portfolio-stream-feed",
    };

    /// Get REST base URL (v2 or HFT)
    pub fn rest_url(&self, use_hft: bool) -> &str {
        if use_hft {
            self.rest_hft
        } else {
            self.rest_base
        }
    }

    /// Get V3 REST URL
    pub fn rest_v3_url(&self) -> &str {
        self.rest_v3
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Upstox API endpoints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum UpstoxEndpoint {
    // === AUTHENTICATION ===
    LoginDialog,
    LoginToken,

    // === MARKET DATA ===
    MarketQuoteLtp,
    MarketQuoteQuotes,
    MarketQuoteOhlc,
    /// GET /v2/market-quote/ltp — multi-instrument LTP quotes (comma-separated instrument_key)
    MarketQuotesMulti,
    HistoricalCandleV2,
    HistoricalCandleV3,
    /// GET /v3/historical-candle/{instrument_key}/{unit}/{interval}/{to_date}/{from_date}
    HistoricalDataV3,
    IntradayCandleV2,
    IntradayCandleV3,
    OptionChain,
    OptionContract,

    // === TRADING ===
    OrderPlaceV2,
    OrderPlaceV3,
    OrderModify,
    OrderCancel,
    OrderDetails,
    OrderBook,
    OrderTrades,
    TradeHistory,
    MultiOrderPlace,
    MultiOrderCancel,

    // === GTT ORDERS ===
    GttPlace,
    GttModify,
    GttCancel,
    GttOrders,
    GttOrderDetails,

    // === PORTFOLIO ===
    PositionsShortTerm,
    HoldingsLongTerm,
    MtfPositions,
    ConvertPosition,
    ExitAllPositions,

    // === ACCOUNT ===
    FundsAndMargin,
    MarginRequirement,
    TradeCharges,
    TradePnl,
    Brokerage,
    UserProfile,

    // === WEBSOCKET ===
    WsMarketDataAuthorize,
    WsPortfolioAuthorize,
}

impl UpstoxEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Authentication
            Self::LoginDialog => "/login/authorization/dialog".to_string(),
            Self::LoginToken => "/login/authorization/token".to_string(),

            // Market Data
            Self::MarketQuoteLtp => "/market-quote/ltp".to_string(),
            Self::MarketQuoteQuotes => "/market-quote/quotes".to_string(),
            Self::MarketQuoteOhlc => "/market-quote/ohlc".to_string(),
            Self::MarketQuotesMulti => "/market-quote/ltp".to_string(),
            Self::HistoricalCandleV2 => "/historical-candle".to_string(),
            Self::HistoricalCandleV3 => "/historical-candle".to_string(),
            Self::HistoricalDataV3 => "/historical-candle".to_string(),
            Self::IntradayCandleV2 => "/historical-candle/intraday".to_string(),
            Self::IntradayCandleV3 => "/historical-candle/intraday".to_string(),
            Self::OptionChain => "/option/chain".to_string(),
            Self::OptionContract => "/option/contract".to_string(),

            // Trading
            Self::OrderPlaceV2 => "/order/place".to_string(),
            Self::OrderPlaceV3 => "/order/place".to_string(),
            Self::OrderModify => "/order/modify".to_string(),
            Self::OrderCancel => "/order/cancel".to_string(),
            Self::OrderDetails => "/order/details".to_string(),
            Self::OrderBook => "/order/details".to_string(),
            Self::OrderTrades => "/order/trades".to_string(),
            Self::TradeHistory => "/order/history".to_string(),
            Self::MultiOrderPlace => "/order/multi/place".to_string(),
            Self::MultiOrderCancel => "/order/multi/cancel".to_string(),

            // GTT Orders
            Self::GttPlace => "/order/gtt/place".to_string(),
            Self::GttModify => "/order/gtt/modify".to_string(),
            Self::GttCancel => "/order/gtt/cancel".to_string(),
            Self::GttOrders => "/gtt/orders".to_string(),
            Self::GttOrderDetails => "/gtt/order".to_string(),

            // Portfolio
            Self::PositionsShortTerm => "/portfolio/short-term-positions".to_string(),
            Self::HoldingsLongTerm => "/portfolio/long-term-holdings".to_string(),
            Self::MtfPositions => "/portfolio/mtf-positions".to_string(),
            Self::ConvertPosition => "/portfolio/convert-position".to_string(),
            Self::ExitAllPositions => "/portfolio/positions".to_string(),

            // Account
            Self::FundsAndMargin => "/user/get-funds-and-margin".to_string(),
            Self::MarginRequirement => "/charges/margin".to_string(),
            Self::TradeCharges => "/trade/profit-loss/charges".to_string(),
            Self::TradePnl => "/trade/profit-loss/data".to_string(),
            Self::Brokerage => "/charges/brokerage".to_string(),
            Self::UserProfile => "/user/profile".to_string(),

            // WebSocket
            Self::WsMarketDataAuthorize => "/feed/market-data-feed/authorize".to_string(),
            Self::WsPortfolioAuthorize => "/feed/portfolio-stream-feed/authorize".to_string(),
        }
    }

    /// Check if endpoint should use V3 API
    pub fn is_v3(&self) -> bool {
        matches!(
            self,
            Self::HistoricalCandleV3
                | Self::IntradayCandleV3
                | Self::HistoricalDataV3
                | Self::OrderPlaceV3
                | Self::GttPlace
                | Self::GttModify
                | Self::GttCancel
                | Self::MtfPositions
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Upstox API
///
/// Upstox uses format: {SEGMENT}|{IDENTIFIER}
/// Examples:
/// - NSE_EQ|INE669E01016 (Equity - ISIN-based)
/// - NSE_FO|54321 (F&O - exchange token)
/// - BSE_EQ|INE002A01018 (BSE equity)
/// - MCX_FO|12345 (MCX futures)
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // For Indian stocks, symbol.base is the instrument identifier
    // symbol.quote represents the segment (NSE_EQ, BSE_EQ, etc.)

    if symbol.quote.is_empty() {
        // Default to NSE_EQ if no segment specified
        format!("NSE_EQ|{}", symbol.base.to_uppercase())
    } else if symbol.quote.contains('|') {
        // Already in Upstox format
        symbol.quote.clone()
    } else {
        // Construct from parts
        format!("{}|{}", symbol.quote.to_uppercase(), symbol.base.to_uppercase())
    }
}

/// Parse symbol from Upstox format back to domain Symbol
pub fn _parse_symbol(api_symbol: &str) -> crate::core::types::Symbol {
    if let Some((segment, identifier)) = api_symbol.split_once('|') {
        crate::core::types::Symbol {
            base: identifier.to_string(),
            quote: segment.to_string(),
            raw: Some(api_symbol.to_string()),
        }
    } else {
        // Fallback: treat as identifier only
        crate::core::types::Symbol {
            base: api_symbol.to_string(),
            quote: "NSE_EQ".to_string(),
            raw: Some(api_symbol.to_string()),
        }
    }
}

/// Map kline interval to Upstox format
///
/// Returns (unit, interval) tuple
/// Examples:
/// - "1m" -> ("minutes", "1")
/// - "5m" -> ("minutes", "5")
/// - "1h" -> ("hours", "1")
/// - "1d" -> ("days", "1")
pub fn map_kline_interval(interval: &str) -> crate::core::ExchangeResult<(&'static str, String)> {
    let interval_lower = interval.to_lowercase();

    if interval_lower.ends_with('m') {
        let num = interval_lower.trim_end_matches('m');
        Ok(("minutes", num.to_string()))
    } else if interval_lower.ends_with('h') {
        let num = interval_lower.trim_end_matches('h');
        Ok(("hours", num.to_string()))
    } else if interval_lower.ends_with('d') {
        Ok(("days", "1".to_string()))
    } else if interval_lower.ends_with('w') {
        Ok(("weeks", "1".to_string()))
    } else if interval_lower.ends_with("mo") {
        Ok(("months", "1".to_string()))
    } else {
        Err(crate::core::ExchangeError::InvalidRequest(
            format!("Invalid interval: {}", interval)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        let symbol = crate::core::types::Symbol {
            base: "INE669E01016".to_string(),
            quote: "NSE_EQ".to_string(),
        };
        assert_eq!(format_symbol(&symbol), "NSE_EQ|INE669E01016");

        let symbol2 = crate::core::types::Symbol {
            base: "54321".to_string(),
            quote: "NSE_FO".to_string(),
        };
        assert_eq!(format_symbol(&symbol2), "NSE_FO|54321");

        let symbol3 = crate::core::types::Symbol {
            base: "INE002A01018".to_string(),
            quote: "".to_string(),
        };
        assert_eq!(format_symbol(&symbol3), "NSE_EQ|INE002A01018");
    }

    #[test]
    fn test_parse_symbol() {
        let parsed = _parse_symbol("NSE_EQ|INE669E01016");
        assert_eq!(parsed.base, "INE669E01016");
        assert_eq!(parsed.quote, "NSE_EQ");

        let parsed2 = _parse_symbol("BSE_FO|12345");
        assert_eq!(parsed2.base, "12345");
        assert_eq!(parsed2.quote, "BSE_FO");

        let parsed3 = _parse_symbol("RELIANCE");
        assert_eq!(parsed3.base, "RELIANCE");
        assert_eq!(parsed3.quote, "NSE_EQ");
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m").unwrap(), ("minutes", "1".to_string()));
        assert_eq!(map_kline_interval("5m").unwrap(), ("minutes", "5".to_string()));
        assert_eq!(map_kline_interval("1h").unwrap(), ("hours", "1".to_string()));
        assert_eq!(map_kline_interval("1d").unwrap(), ("days", "1".to_string()));
        assert_eq!(map_kline_interval("1w").unwrap(), ("weeks", "1".to_string()));
        assert!(map_kline_interval("invalid").is_err());
    }
}
