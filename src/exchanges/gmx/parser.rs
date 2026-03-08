//! # GMX Response Parser
//!
//! Parsing JSON responses from GMX API.
//!
//! ## Price Precision
//! GMX uses 30 decimals for all USD prices.
//! Example: "2500000000000000000000000000000000" = $2,500.00
//! Conversion: value / 10^30 = USD price

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker,
};

/// Parser for GMX API responses
pub struct GmxParser;

impl GmxParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get token decimals (hardcoded for common tokens)
    ///
    /// GMX stores USD prices with formula: price_usd = raw_value / 10^(30 - token_decimals)
    ///
    /// TODO: Fetch from /tokens endpoint and cache
    fn get_token_decimals(symbol: &str) -> u32 {
        match symbol.to_uppercase().as_str() {
            "BTC" | "WBTC" | "WBTC.b" => 8,
            "ETH" | "WETH" => 18,
            "USDC" | "USDT" | "DAI" | "USDC.e" => 6,
            "APT" => 8,
            "BOME" | "PYTH" => 6,
            "FLOKI" => 9,
            "MEW" => 5,
            "TAO" | "BONK" => 9,
            "WLD" | "LINK" | "UNI" | "ARB" | "AAVE" | "AVAX" | "FTM" | "CRV" => 18,
            "APE" | "MEME" | "tBTC" | "GMX" => 18,
            "DOGE" | "SOL" => 18, // Wrapped versions
            "SUI" | "STX" | "LTC" => 18,
            _ => 18, // Default to 18 (most ERC20 tokens)
        }
    }

    /// Parse f64 from GMX price string
    ///
    /// GMX uses 30 decimals for USD prices with token-specific precision:
    /// Formula: price_usd = raw_value / 10^(30 - token_decimals)
    ///
    /// Examples:
    /// - ETH (18 decimals): "2946608494813104" / 10^12 = $2,946.61
    /// - BTC (8 decimals): "891457441636920300000000000" / 10^22 = $89,145.74
    fn parse_gmx_price(value: &Value, token_symbol: &str) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|val| {
                let token_decimals = Self::get_token_decimals(token_symbol);
                let divisor_exponent = 30 - token_decimals;
                let divisor = 10_f64.powi(divisor_exponent as i32);
                val / divisor
            })
    }

    /// Parse standard f64 (non-price fields)
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Get field as f64
    fn _get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Get GMX price field as f64 (requires token symbol for precision)
    fn get_price(data: &Value, key: &str, token_symbol: &str) -> Option<f64> {
        data.get(key).and_then(|v| Self::parse_gmx_price(v, token_symbol))
    }

    /// Require f64 field
    fn _require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::_get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Get string field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Require string field
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Get i64 field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| {
            v.as_i64().or_else(|| {
                v.as_str().and_then(|s| s.parse().ok())
            })
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ping response
    pub fn parse_ping(response: &Value) -> ExchangeResult<bool> {
        let status = response.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(status == "ok")
    }

    /// Parse single ticker price from tickers endpoint
    ///
    /// Response format (ARRAY):
    /// ```json
    /// [
    ///   {
    ///     "tokenSymbol": "ETH",
    ///     "minPrice": "2947435954854362",
    ///     "maxPrice": "2947435954854362",
    ///     "timestamp": 1769289283
    ///   }
    /// ]
    /// ```
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        // Extract base symbol (ETH from ETH/USD)
        let base = symbol.split('/').next().unwrap_or(symbol).to_uppercase();

        // Response is an array, not an object
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        // Find the ticker by tokenSymbol
        let ticker_data = arr.iter()
            .find(|item| {
                item.get("tokenSymbol")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_uppercase() == base)
                    .unwrap_or(false)
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Symbol '{}' not found in tickers", base)))?;

        // Get token symbol for precision calculation
        let token_symbol = Self::get_str(ticker_data, "tokenSymbol")
            .ok_or_else(|| ExchangeError::Parse("Missing tokenSymbol".to_string()))?;

        let min_price = Self::get_price(ticker_data, "minPrice", token_symbol)
            .ok_or_else(|| ExchangeError::Parse("Missing minPrice".to_string()))?;
        let max_price = Self::get_price(ticker_data, "maxPrice", token_symbol)
            .ok_or_else(|| ExchangeError::Parse("Missing maxPrice".to_string()))?;

        // Use average of min/max as last price
        let last_price = (min_price + max_price) / 2.0;

        // Timestamp in seconds, convert to milliseconds
        let timestamp = Self::get_i64(ticker_data, "timestamp")
            .unwrap_or(0) * 1000;

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: Some(min_price),
            ask_price: Some(max_price),
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    /// Parse all tickers
    ///
    /// Returns array of tickers
    pub fn parse_all_tickers(response: &Value) -> ExchangeResult<Vec<Ticker>> {
        let mut tickers = Vec::new();

        // Response is an array
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        for ticker_data in arr {
            // Get token symbol
            let token_symbol = match Self::get_str(ticker_data, "tokenSymbol") {
                Some(s) => s,
                None => continue, // Skip entries without symbol
            };

            let min_price = match Self::get_price(ticker_data, "minPrice", token_symbol) {
                Some(p) => p,
                None => continue, // Skip invalid entries
            };

            let max_price = match Self::get_price(ticker_data, "maxPrice", token_symbol) {
                Some(p) => p,
                None => continue,
            };

            let last_price = (min_price + max_price) / 2.0;
            let timestamp = Self::get_i64(ticker_data, "timestamp").unwrap_or(0) * 1000;

            // Format as "SYMBOL/USD" for consistency
            let formatted_symbol = format!("{}/USD", token_symbol.to_uppercase());

            tickers.push(Ticker {
                symbol: formatted_symbol,
                last_price,
                bid_price: Some(min_price),
                ask_price: Some(max_price),
                high_24h: None,
                low_24h: None,
                volume_24h: None,
                quote_volume_24h: None,
                price_change_24h: None,
                price_change_percent_24h: None,
                timestamp,
            });
        }

        Ok(tickers)
    }

    /// Parse candlesticks (OHLC)
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "candles": [
    ///     [1769288400, 89242.13, 89248.9, 89128.71, 89184.39],
    ///     [1769284800, 89190.51, 89264.9, 89189.11, 89242.0]
    ///   ],
    ///   "period": "1h"
    /// }
    /// ```
    ///
    /// Format: [timestamp, open, high, low, close]
    /// Ordering: Descending (newest first)
    /// Prices are already in USD (not raw format)
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        // Extract "candles" array from response object
        let candles_value = response.get("candles")
            .ok_or_else(|| ExchangeError::Parse("Missing 'candles' field".to_string()))?;

        let arr = candles_value.as_array()
            .ok_or_else(|| ExchangeError::Parse("'candles' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 5 {
                continue;
            }

            // GMX format: [timestamp, open, high, low, close]
            let open_time = candle[0].as_i64()
                .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?
                * 1000; // seconds to ms

            // Prices are already in USD format (not raw)
            let open = Self::parse_f64(&candle[1]).unwrap_or(0.0);
            let high = Self::parse_f64(&candle[2]).unwrap_or(0.0);
            let low = Self::parse_f64(&candle[3]).unwrap_or(0.0);
            let close = Self::parse_f64(&candle[4]).unwrap_or(0.0);

            klines.push(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume: 0.0, // GMX doesn't provide volume in candles API
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        // GMX returns newest first, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    /// Parse orderbook
    ///
    /// Note: GMX doesn't have traditional orderbooks (it's a DEX with oracle pricing).
    /// This would need to be constructed from pool liquidity data if needed.
    pub fn parse_orderbook(_response: &Value) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::NotSupported(
            "GMX uses oracle pricing, not orderbooks".to_string()
        ))
    }

    /// Parse symbols/markets list
    ///
    /// The real GMX `/markets` endpoint returns:
    /// ```json
    /// {
    ///   "markets": [
    ///     {
    ///       "name": "ETH/USD [ETH-USDC]",
    ///       "marketToken": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
    ///       "indexToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    ///       "indexTokenSymbol": "ETH",
    ///       "marketSymbol": "ETH/USD"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        // The real endpoint wraps the array in {"markets": [...]}.
        // Fall back to treating the response itself as an array for forward-compat.
        let arr = if let Some(inner) = response.get("markets").and_then(|v| v.as_array()) {
            inner
        } else if let Some(bare) = response.as_array() {
            bare
        } else {
            return Err(ExchangeError::Parse(
                "Markets response: expected object with 'markets' array or bare array".to_string(),
            ));
        };

        let mut symbols = Vec::with_capacity(arr.len());

        for market in arr {
            // Prefer "marketSymbol", then "name", then build from "indexTokenSymbol"
            if let Some(symbol) = Self::get_str(market, "marketSymbol") {
                symbols.push(symbol.to_string());
            } else if let Some(name) = Self::get_str(market, "name") {
                // "name" field: "ETH/USD [ETH-USDC]" — strip the pool suffix
                let clean = name.find('[')
                    .map(|pos| name[..pos].trim())
                    .unwrap_or(name);
                symbols.push(clean.to_string());
            } else if let Some(index_symbol) = Self::get_str(market, "indexTokenSymbol") {
                // Last resort: construct from index token
                symbols.push(format!("{}/USD", index_symbol));
            }
        }

        Ok(symbols)
    }

    // Note: WebSocket-like functionality is handled in websocket.rs via polling
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_gmx_price() {
        // ETH (18 decimals): price_usd = raw / 10^(30-18) = raw / 10^12
        let value = json!("2500000000000000");
        let price = GmxParser::parse_gmx_price(&value, "ETH").unwrap();
        assert!((price - 2500.0).abs() < 0.01);

        // BTC (8 decimals): price_usd = raw / 10^(30-8) = raw / 10^22
        let value = json!("89156000000000000000000000");
        let price = GmxParser::parse_gmx_price(&value, "BTC").unwrap();
        assert!((price - 89156.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_ping() {
        let response = json!({"status": "ok"});
        assert!(GmxParser::parse_ping(&response).unwrap());

        let response = json!({"status": "error"});
        assert!(!GmxParser::parse_ping(&response).unwrap());
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!([
            {
                "tokenSymbol": "ETH",
                "minPrice": "2500000000000000",
                "maxPrice": "2501000000000000",
                "timestamp": 1674567890
            }
        ]);

        let ticker = GmxParser::parse_ticker(&response, "ETH/USD").unwrap();
        assert_eq!(ticker.symbol, "ETH/USD");
        assert!((ticker.last_price - 2500.5).abs() < 1.0);
        assert!(ticker.bid_price.is_some());
        assert!(ticker.ask_price.is_some());
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "candles": [
                [1674567890, 2503.45, 2508.92, 2501.23, 2505.67],
                [1674564290, 2498.12, 2504.56, 2495.78, 2503.45]
            ],
            "period": "1h"
        });

        let klines = GmxParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 2);

        // Reversed to oldest first
        let first = &klines[0];
        assert_eq!(first.open_time, 1674564290000);
        assert!((first.open - 2498.12).abs() < 0.01);
        assert!((first.close - 2503.45).abs() < 0.01);
    }

    #[test]
    fn test_parse_symbols() {
        // Real GMX /markets endpoint wraps array in {"markets": [...]}
        let response = json!({
            "markets": [
                {
                    "name": "ETH/USD [ETH-USDC]",
                    "marketToken": "0xabc",
                    "indexTokenSymbol": "ETH"
                },
                {
                    "name": "BTC/USD [WBTC-USDC]",
                    "marketToken": "0xdef",
                    "indexTokenSymbol": "BTC"
                }
            ]
        });

        let symbols = GmxParser::parse_symbols(&response).unwrap();
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains(&"ETH/USD".to_string()));
        assert!(symbols.contains(&"BTC/USD".to_string()));
    }

    #[test]
    fn test_parse_symbols_bare_array() {
        // Bare-array format (fallback compat)
        let response = json!([
            { "marketSymbol": "ETH/USD", "indexTokenSymbol": "ETH" },
            { "marketSymbol": "BTC/USD", "indexTokenSymbol": "BTC" }
        ]);

        let symbols = GmxParser::parse_symbols(&response).unwrap();
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains(&"ETH/USD".to_string()));
        assert!(symbols.contains(&"BTC/USD".to_string()));
    }
}
