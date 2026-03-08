//! Finnhub response parsers
//!
//! Parse JSON responses to domain types based on Finnhub API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::*;

pub struct FinnhubParser;

impl FinnhubParser {
    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for API errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let error_msg = error.as_str().unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: 0,
                message: error_msg.to_string(),
            });
        }
        Ok(())
    }

    /// Get required string field
    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    /// Get optional string field
    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    /// Get required f64 field
    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    /// Get optional f64 field
    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    /// Get required i64 field
    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    /// Get optional i64 field
    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STOCK PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse quote data
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "c": 150.25,
    ///   "d": -1.23,
    ///   "dp": -0.82,
    ///   "h": 152.0,
    ///   "l": 149.5,
    ///   "o": 151.0,
    ///   "pc": 151.48,
    ///   "t": 1706000000
    /// }
    /// ```
    pub fn parse_quote(response: &Value) -> ExchangeResult<Quote> {
        Ok(Quote {
            current_price: Self::require_f64(response, "c")?,
            change: Self::get_f64(response, "d"),
            change_percent: Self::get_f64(response, "dp"),
            high: Self::get_f64(response, "h"),
            low: Self::get_f64(response, "l"),
            open: Self::get_f64(response, "o"),
            previous_close: Self::get_f64(response, "pc"),
            timestamp: Self::require_i64(response, "t")?,
        })
    }

    /// Parse candle data
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "c": [150.1, 151.2, 149.8],
    ///   "h": [152.0, 153.0, 151.0],
    ///   "l": [149.0, 150.0, 148.5],
    ///   "o": [151.0, 150.5, 151.5],
    ///   "s": "ok",
    ///   "t": [1706000000, 1706003600, 1706007200],
    ///   "v": [1000000, 1200000, 950000]
    /// }
    /// ```
    pub fn parse_candles(response: &Value) -> ExchangeResult<Vec<Candle>> {
        let status = Self::require_str(response, "s")?;
        if status != "ok" {
            return Err(ExchangeError::Parse(format!("Invalid status: {}", status)));
        }

        let closes = response.get("c").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'c' array".to_string()))?;
        let highs = response.get("h").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'h' array".to_string()))?;
        let lows = response.get("l").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'l' array".to_string()))?;
        let opens = response.get("o").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'o' array".to_string()))?;
        let timestamps = response.get("t").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 't' array".to_string()))?;
        let volumes = response.get("v").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'v' array".to_string()))?;

        let len = closes.len();
        let mut candles = Vec::with_capacity(len);

        for i in 0..len {
            candles.push(Candle {
                open: opens[i].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid open".to_string()))?,
                high: highs[i].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid high".to_string()))?,
                low: lows[i].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid low".to_string()))?,
                close: closes[i].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid close".to_string()))?,
                volume: volumes[i].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid volume".to_string()))?,
                timestamp: timestamps[i].as_i64().ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?,
            });
        }

        Ok(candles)
    }

    /// Parse symbol search results
    pub fn parse_search_results(response: &Value) -> ExchangeResult<Vec<SearchResult>> {
        let results = response.get("result").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                Ok(SearchResult {
                    symbol: Self::require_str(item, "symbol")?.to_string(),
                    description: Self::require_str(item, "description")?.to_string(),
                    display_symbol: Self::require_str(item, "displaySymbol")?.to_string(),
                    instrument_type: Self::get_str(item, "type").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse company profile
    pub fn parse_company_profile(response: &Value) -> ExchangeResult<CompanyProfile> {
        Ok(CompanyProfile {
            country: Self::get_str(response, "country").map(|s| s.to_string()),
            currency: Self::get_str(response, "currency").map(|s| s.to_string()),
            exchange: Self::get_str(response, "exchange").map(|s| s.to_string()),
            ipo: Self::get_str(response, "ipo").map(|s| s.to_string()),
            market_cap: Self::get_f64(response, "marketCapitalization"),
            name: Self::require_str(response, "name")?.to_string(),
            ticker: Self::require_str(response, "ticker")?.to_string(),
            weburl: Self::get_str(response, "weburl").map(|s| s.to_string()),
            logo: Self::get_str(response, "logo").map(|s| s.to_string()),
            phone: Self::get_str(response, "phone").map(|s| s.to_string()),
            share_outstanding: Self::get_f64(response, "shareOutstanding"),
            industry: Self::get_str(response, "finnhubIndustry").map(|s| s.to_string()),
        })
    }

    /// Parse company peers
    pub fn parse_peers(response: &Value) -> ExchangeResult<Vec<String>> {
        let peers = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        peers
            .iter()
            .map(|v| v.as_str()
                .ok_or_else(|| ExchangeError::Parse("Invalid peer symbol".to_string()))
                .map(|s| s.to_string())
            )
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse market news
    pub fn parse_news(response: &Value) -> ExchangeResult<Vec<NewsArticle>> {
        let articles = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        articles
            .iter()
            .map(|article| {
                Ok(NewsArticle {
                    category: Self::require_str(article, "category")?.to_string(),
                    datetime: Self::require_i64(article, "datetime")?,
                    headline: Self::require_str(article, "headline")?.to_string(),
                    id: Self::require_i64(article, "id")?,
                    image: Self::get_str(article, "image").map(|s| s.to_string()),
                    related: Self::get_str(article, "related").map(|s| s.to_string()),
                    source: Self::require_str(article, "source")?.to_string(),
                    summary: Self::require_str(article, "summary")?.to_string(),
                    url: Self::require_str(article, "url")?.to_string(),
                })
            })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Real-time quote data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub current_price: f64,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub open: Option<f64>,
    pub previous_close: Option<f64>,
    pub timestamp: i64,
}

/// Candle/OHLC data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: i64,
}

/// Symbol search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub symbol: String,
    pub description: String,
    pub display_symbol: String,
    pub instrument_type: Option<String>,
}

/// Company profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub country: Option<String>,
    pub currency: Option<String>,
    pub exchange: Option<String>,
    pub ipo: Option<String>,
    pub market_cap: Option<f64>,
    pub name: String,
    pub ticker: String,
    pub weburl: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub share_outstanding: Option<f64>,
    pub industry: Option<String>,
}

/// News article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub category: String,
    pub datetime: i64,
    pub headline: String,
    pub id: i64,
    pub image: Option<String>,
    pub related: Option<String>,
    pub source: String,
    pub summary: String,
    pub url: String,
}

/// Market status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatus {
    pub exchange: String,
    pub holiday: Option<String>,
    pub is_open: bool,
    pub session: String,
    pub timezone: String,
}

/// Earnings data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earnings {
    pub date: String,
    pub eps_actual: Option<f64>,
    pub eps_estimate: Option<f64>,
    pub hour: String,
    pub quarter: i64,
    pub revenue_actual: Option<f64>,
    pub revenue_estimate: Option<f64>,
    pub symbol: String,
    pub year: i64,
}

/// IPO data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipo {
    pub date: String,
    pub exchange: String,
    pub name: String,
    pub number_of_shares: Option<i64>,
    pub price: Option<String>,
    pub status: String,
    pub symbol: String,
    pub total_shares_value: Option<i64>,
}

/// Social sentiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSentiment {
    pub symbol: String,
    pub data: Vec<SentimentData>,
}

/// Sentiment data point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SentimentData {
    pub atTime: String,
    pub mention: i64,
    pub positive_mention: i64,
    pub negative_mention: i64,
    pub positive_score: f64,
    pub negative_score: f64,
    pub score: f64,
}

/// Insider transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTransaction {
    pub symbol: String,
    pub name: String,
    pub share: i64,
    pub change: i64,
    pub filing_date: String,
    pub transaction_date: String,
    pub transaction_price: Option<f64>,
    pub transaction_code: String,
}

/// Recommendation trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationTrend {
    pub buy: i64,
    pub hold: i64,
    pub period: String,
    pub sell: i64,
    pub strong_buy: i64,
    pub strong_sell: i64,
    pub symbol: String,
}
