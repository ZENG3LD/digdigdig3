//! CoinGecko response parsers

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct CoinGeckoParser;

impl CoinGeckoParser {
    // ═══════════════════════════════════════════════════════════════════════
    // SIMPLE PRICE PARSER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse simple price response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "bitcoin": {
    ///     "usd": 43250.5,
    ///     "eur": 39500.2
    ///   }
    /// }
    /// ```
    pub fn parse_simple_price(response: &Value) -> ExchangeResult<HashMap<String, HashMap<String, f64>>> {
        let obj = response.as_object()
            .ok_or_else(|| ExchangeError::Parse("Expected object for simple price".to_string()))?;

        let mut result = HashMap::new();
        for (coin_id, currencies) in obj.iter() {
            if let Some(curr_obj) = currencies.as_object() {
                let mut prices = HashMap::new();
                for (currency, price) in curr_obj.iter() {
                    if let Some(p) = price.as_f64() {
                        prices.insert(currency.clone(), p);
                    }
                }
                result.insert(coin_id.clone(), prices);
            }
        }

        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COINS PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse coins list
    pub fn parse_coins_list(response: &Value) -> ExchangeResult<Vec<SimpleCoin>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for coins list".to_string()))?;

        array.iter()
            .map(|item| {
                Ok(SimpleCoin {
                    id: Self::require_str(item, "id")?.to_string(),
                    symbol: Self::require_str(item, "symbol")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse coin detail
    pub fn parse_coin_detail(response: &Value) -> ExchangeResult<CoinDetail> {
        Ok(CoinDetail {
            id: Self::require_str(response, "id")?.to_string(),
            symbol: Self::require_str(response, "symbol")?.to_string(),
            name: Self::require_str(response, "name")?.to_string(),
            description: Self::parse_description(response.get("description")),
            market_data: Self::parse_market_data(response.get("market_data"))?,
            categories: Self::parse_categories(response.get("categories")),
        })
    }

    fn parse_description(desc: Option<&Value>) -> HashMap<String, String> {
        desc.and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_market_data(data: Option<&Value>) -> ExchangeResult<MarketData> {
        let data = data.ok_or_else(|| ExchangeError::Parse("Missing market_data".to_string()))?;

        Ok(MarketData {
            current_price: Self::parse_currency_map(data.get("current_price")),
            market_cap: Self::parse_currency_map(data.get("market_cap")),
            total_volume: Self::parse_currency_map(data.get("total_volume")),
            ath: Self::parse_currency_map(data.get("ath")),
        })
    }

    fn parse_currency_map(value: Option<&Value>) -> HashMap<String, f64> {
        value.and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f)))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_categories(cats: Option<&Value>) -> Vec<String> {
        cats.and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse market chart data
    pub fn parse_market_chart(response: &Value) -> ExchangeResult<CoinMarketChart> {
        Ok(CoinMarketChart {
            prices: Self::parse_chart_array(response.get("prices")),
            market_caps: Self::parse_chart_array(response.get("market_caps")),
            total_volumes: Self::parse_chart_array(response.get("total_volumes")),
        })
    }

    fn parse_chart_array(value: Option<&Value>) -> Vec<[f64; 2]> {
        value.and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.as_array().and_then(|a| {
                            if a.len() >= 2 {
                                let t = a[0].as_f64()?;
                                let v = a[1].as_f64()?;
                                Some([t, v])
                            } else {
                                None
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse coins markets
    pub fn parse_coins_markets(response: &Value) -> ExchangeResult<Vec<CoinPrice>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for coins markets".to_string()))?;

        array.iter()
            .map(|item| {
                Ok(CoinPrice {
                    id: Self::require_str(item, "id")?.to_string(),
                    symbol: Self::require_str(item, "symbol")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                    current_price: Self::get_f64(item, "current_price").unwrap_or(0.0),
                    market_cap: Self::get_f64(item, "market_cap").unwrap_or(0.0),
                    total_volume: Self::get_f64(item, "total_volume").unwrap_or(0.0),
                    price_change_24h: Self::get_f64(item, "price_change_24h").unwrap_or(0.0),
                    price_change_percentage_24h: Self::get_f64(item, "price_change_percentage_24h").unwrap_or(0.0),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SEARCH PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse trending coins
    pub fn parse_trending(response: &Value) -> ExchangeResult<Vec<TrendingCoin>> {
        let coins = response
            .get("coins")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'coins' array".to_string()))?;

        let results = coins.iter()
            .filter_map(|item| {
                let coin_item = item.get("item")?;
                Some(TrendingCoin {
                    id: Self::get_str(coin_item, "id")?.to_string(),
                    coin_id: Self::get_i64(coin_item, "coin_id").unwrap_or(0),
                    name: Self::get_str(coin_item, "name")?.to_string(),
                    symbol: Self::get_str(coin_item, "symbol")?.to_string(),
                    market_cap_rank: Self::get_i64(coin_item, "market_cap_rank"),
                    thumb: Self::get_str(coin_item, "thumb").map(|s| s.to_string()),
                    price_btc: Self::get_f64(coin_item, "price_btc").unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();
        Ok(results)
    }

    /// Parse search results
    pub fn parse_search(response: &Value) -> ExchangeResult<Vec<SimpleCoin>> {
        let coins = response
            .get("coins")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'coins' array".to_string()))?;

        coins.iter()
            .map(|item| {
                Ok(SimpleCoin {
                    id: Self::require_str(item, "id")?.to_string(),
                    symbol: Self::require_str(item, "symbol")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GLOBAL PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse global market data
    pub fn parse_global(response: &Value) -> ExchangeResult<GlobalData> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        Ok(GlobalData {
            total_market_cap: Self::parse_currency_map(data.get("total_market_cap")),
            total_volume: Self::parse_currency_map(data.get("total_volume")),
            market_cap_percentage: Self::parse_currency_map(data.get("market_cap_percentage")),
            market_cap_change_percentage_24h_usd: Self::get_f64(data, "market_cap_change_percentage_24h_usd").unwrap_or(0.0),
            active_cryptocurrencies: Self::get_u64(data, "active_cryptocurrencies").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXCHANGE PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse exchanges list
    pub fn parse_exchanges(response: &Value) -> ExchangeResult<Vec<CoinGeckoExchange>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for exchanges".to_string()))?;

        array.iter()
            .map(|item| {
                Ok(CoinGeckoExchange {
                    id: Self::require_str(item, "id")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                    year_established: Self::get_u32(item, "year_established"),
                    country: Self::get_str(item, "country").map(|s| s.to_string()),
                    trust_score: Self::get_u32(item, "trust_score"),
                    trade_volume_24h_btc: Self::get_f64(item, "trade_volume_24h_btc").unwrap_or(0.0),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error.as_str().unwrap_or("Unknown error").to_string();
            return Err(ExchangeError::Api { code: 400, message });
        }

        if let Some(status) = response.get("status") {
            if let Some(error_code) = status.get("error_code") {
                let code = error_code.as_i64().unwrap_or(0) as i32;
                let message = status
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                return Err(ExchangeError::Api { code, message });
            }
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COINGECKO-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Simple coin info (id, symbol, name)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCoin {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

/// Coin price data from markets endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinPrice {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: f64,
    pub market_cap: f64,
    pub total_volume: f64,
    pub price_change_24h: f64,
    pub price_change_percentage_24h: f64,
}

/// Detailed coin information
#[derive(Debug, Clone)]
pub struct CoinDetail {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub description: HashMap<String, String>,
    pub market_data: MarketData,
    pub categories: Vec<String>,
}

/// Market data for a coin
#[derive(Debug, Clone)]
pub struct MarketData {
    pub current_price: HashMap<String, f64>,
    pub market_cap: HashMap<String, f64>,
    pub total_volume: HashMap<String, f64>,
    pub ath: HashMap<String, f64>,
}

/// Historical market chart data
#[derive(Debug, Clone)]
pub struct CoinMarketChart {
    pub prices: Vec<[f64; 2]>,       // [timestamp, price]
    pub market_caps: Vec<[f64; 2]>,  // [timestamp, market_cap]
    pub total_volumes: Vec<[f64; 2]>, // [timestamp, volume]
}

/// Trending coin data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCoin {
    pub id: String,
    pub coin_id: i64,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: Option<i64>,
    pub thumb: Option<String>,
    pub price_btc: f64,
}

/// Global cryptocurrency market data
#[derive(Debug, Clone)]
pub struct GlobalData {
    pub total_market_cap: HashMap<String, f64>,
    pub total_volume: HashMap<String, f64>,
    pub market_cap_percentage: HashMap<String, f64>,
    pub market_cap_change_percentage_24h_usd: f64,
    pub active_cryptocurrencies: u64,
}

/// Exchange information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinGeckoExchange {
    pub id: String,
    pub name: String,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub trust_score: Option<u32>,
    pub trade_volume_24h_btc: f64,
}
