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
    FundingRate, Position, PositionSide, MarginType, Order, OrderSide, OrderStatus, OrderType, TimeInForce,
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

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING RATE (from /markets/info)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding rate for a given symbol from the `/markets/info` response.
    ///
    /// The `/markets/info` endpoint returns an object keyed by market token address:
    /// ```json
    /// {
    ///   "0x70d9...": {
    ///     "marketSymbol": "ETH/USD [ETH-USDC]",
    ///     "indexTokenSymbol": "ETH",
    ///     "fundingFactorPerSecond": "380517503805175",
    ///     "longsPayShorts": true,
    ///     "borrowingFactorPerSecondForLongs": "190258751902587",
    ///     "borrowingFactorPerSecondForShorts": "190258751902587"
    ///   }
    /// }
    /// ```
    ///
    /// `is_long` selects which side's effective rate to return:
    /// - `true`  → `fundingFactorPerSecond` (positive when `longsPayShorts`, negative otherwise)
    /// - `false` → negated rate
    ///
    /// Rates are raw per-second factors with 30-decimal precision.
    /// Converted to per-hour by multiplying by 3600.
    pub fn parse_funding_rate(
        response: &Value,
        symbol: &str,
        is_long: bool,
    ) -> ExchangeResult<FundingRate> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Extract base symbol (e.g. "ETH" from "ETH/USD")
        let base = symbol.split('/').next().unwrap_or(symbol).to_uppercase();

        // Response can be either:
        // 1. A top-level object {"markets": [...]} (unlikely for /markets/info but defensive)
        // 2. An object keyed by market token address
        let markets_obj = if let Some(inner) = response.get("markets") {
            inner
        } else {
            response
        };

        // Find a market entry whose indexTokenSymbol (or marketSymbol prefix) matches
        let market_data = if let Some(obj) = markets_obj.as_object() {
            obj.values().find(|v| {
                // Match by indexTokenSymbol field
                v.get("indexTokenSymbol")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_uppercase() == base)
                    .unwrap_or(false)
            }).ok_or_else(|| ExchangeError::Parse(format!(
                "Symbol '{}' not found in /markets/info response", base
            )))?
        } else if let Some(arr) = markets_obj.as_array() {
            // Fallback: array of market objects
            arr.iter().find(|v| {
                v.get("indexTokenSymbol")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_uppercase() == base)
                    .unwrap_or(false)
            }).ok_or_else(|| ExchangeError::Parse(format!(
                "Symbol '{}' not found in /markets/info response", base
            )))?
        } else {
            return Err(ExchangeError::Parse(
                "/markets/info: expected object or array".to_string()
            ));
        };

        // Parse funding factor per second (30-decimal precision string)
        // Rate = factor / 10^30 per second → multiply by 3600 for per-hour
        let raw_factor: f64 = market_data
            .get("fundingFactorPerSecond")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Convert to hourly rate: raw / 10^30 * 3600
        let factor_per_hour = raw_factor / 1e30 * 3600.0;

        // longsPayShorts determines sign
        let longs_pay = market_data
            .get("longsPayShorts")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // If longsPayShorts=true: longs pay (positive rate for longs, negative for shorts)
        // If longsPayShorts=false: shorts pay (negative rate for longs, positive for shorts)
        let rate = if is_long {
            if longs_pay { factor_per_hour } else { -factor_per_hour }
        } else {
            if longs_pay { -factor_per_hour } else { factor_per_hour }
        };

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Ok(FundingRate {
            symbol: symbol.to_string(),
            rate,
            next_funding_time: None, // GMX uses continuous funding, no discrete settlement
            timestamp: now_ms,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUBSQUID — POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse open positions from a Subsquid GraphQL response.
    ///
    /// Expected Subsquid response shape:
    /// ```json
    /// {
    ///   "data": {
    ///     "positions": [
    ///       {
    ///         "id": "...",
    ///         "market": "0x...",
    ///         "collateralToken": "0x...",
    ///         "isLong": true,
    ///         "sizeInUsd": "5000000000000000000000000000000000",
    ///         "sizeInTokens": "2000000000000000000",
    ///         "collateralAmount": "500000000000000000",
    ///         "entryPrice": "2500000000000000000000000000000000",
    ///         "unrealizedPnl": "100000000000000000000000000000000",
    ///         "createdAt": 1674567890,
    ///         "indexTokenSymbol": "ETH"
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// USD values use 30-decimal precision; `sizeInTokens` / `collateralAmount` use
    /// token-native decimals.
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let arr = response
            .get("data")
            .and_then(|d| d.get("positions"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse(
                "Subsquid positions: expected data.positions array".to_string()
            ))?;

        let mut positions = Vec::with_capacity(arr.len());

        for item in arr {
            // Determine symbol: prefer "indexTokenSymbol", fall back to market address
            let raw_symbol = item.get("indexTokenSymbol")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| {
                    item.get("market")
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN")
                });
            let symbol = format!("{}/USD", raw_symbol.to_uppercase());

            let is_long = item.get("isLong")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let side = if is_long { PositionSide::Long } else { PositionSide::Short };

            // sizeInUsd: 30-decimal USD → f64
            let size_usd: f64 = item.get("sizeInUsd")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e30)
                .unwrap_or(0.0);

            // sizeInTokens: token-native decimals (18 for most) → use as quantity
            let size_tokens: f64 = item.get("sizeInTokens")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e18)
                .unwrap_or(size_usd);

            // entryPrice: 30-decimal USD
            let entry_price: f64 = item.get("entryPrice")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e30)
                .unwrap_or(0.0);

            // unrealizedPnl: 30-decimal USD (may be negative string)
            let unrealized_pnl: f64 = item.get("unrealizedPnl")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e30)
                .or_else(|| {
                    // Some Subsquid versions return as numeric
                    item.get("unrealizedPnl").and_then(|v| v.as_f64())
                })
                .unwrap_or(0.0);

            // realizedPnl: optional
            let realized_pnl: Option<f64> = item.get("realizedPnl")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e30);

            // collateralAmount: token decimals (18 default)
            // Not directly mapped to a Position field; use for margin approximation
            let collateral: f64 = item.get("collateralAmount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e18)
                .unwrap_or(0.0);

            // Leverage: size_usd / collateral_usd (approximation since collateral is in tokens)
            // We leave it as 1 when we cannot compute it reliably
            let leverage: u32 = if collateral > 0.0 && entry_price > 0.0 {
                let collateral_usd = collateral * entry_price;
                if collateral_usd > 0.0 {
                    (size_usd / collateral_usd).round() as u32
                } else {
                    1
                }
            } else {
                1
            };

            positions.push(Position {
                symbol,
                side,
                quantity: size_tokens,
                entry_price,
                mark_price: None,
                unrealized_pnl,
                realized_pnl,
                liquidation_price: None,
                leverage,
                margin_type: MarginType::Cross, // GMX always cross (shared pool)
                margin: Some(collateral),
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUBSQUID — ORDERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse orders from a Subsquid GraphQL response.
    ///
    /// Used for both `get_open_orders` (status=active) and
    /// `get_order_history` (status=executed/cancelled).
    ///
    /// Expected shape per order in `data.orders`:
    /// ```json
    /// {
    ///   "id": "0x...",
    ///   "orderType": "MarketIncrease",
    ///   "market": "0x...",
    ///   "indexTokenSymbol": "ETH",
    ///   "sizeDeltaUsd": "1000000000000000000000000000000000",
    ///   "triggerPrice": "2500000000000000000000000000000000",
    ///   "isLong": true,
    ///   "status": "active",
    ///   "timestamp": 1674567890
    /// }
    /// ```
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let arr = response
            .get("data")
            .and_then(|d| d.get("orders"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse(
                "Subsquid orders: expected data.orders array".to_string()
            ))?;

        let mut orders = Vec::with_capacity(arr.len());

        for item in arr {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let raw_symbol = item.get("indexTokenSymbol")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            let symbol = format!("{}/USD", raw_symbol.to_uppercase());

            let is_long = item.get("isLong")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let side = if is_long { OrderSide::Buy } else { OrderSide::Sell };

            // Map Subsquid orderType string to our enum
            let order_type_str = item.get("orderType")
                .and_then(|v| v.as_str())
                .unwrap_or("MarketIncrease");
            let (order_type, stop_price) = Self::map_order_type(order_type_str, item);

            // sizeDeltaUsd: 30-decimal USD
            let size_usd: f64 = item.get("sizeDeltaUsd")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e30)
                .unwrap_or(0.0);

            // triggerPrice: 30-decimal USD → price field for limit orders
            let trigger_price: Option<f64> = item.get("triggerPrice")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v / 1e30)
                .filter(|&p| p > 0.0);

            let status_str = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("active");
            let status = Self::map_order_status(status_str);

            let timestamp_sec = item.get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let created_at = timestamp_sec * 1000; // to ms

            orders.push(Order {
                id,
                client_order_id: None,
                symbol,
                side,
                order_type,
                status,
                price: trigger_price,
                stop_price,
                quantity: size_usd, // GMX sizes are in USD notional
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at,
                updated_at: None,
                time_in_force: TimeInForce::Gtc,
            });
        }

        Ok(orders)
    }

    /// Parse a single order from Subsquid `data.order` (singular).
    pub fn parse_single_order(response: &Value) -> ExchangeResult<Order> {
        // Some Subsquid queries return `data.order` (singular) for by-id lookup
        let item = response
            .get("data")
            .and_then(|d| d.get("order"))
            .ok_or_else(|| ExchangeError::Parse(
                "Subsquid order: expected data.order object".to_string()
            ))?;

        // Wrap in a fake array response for code reuse
        let fake = serde_json::json!({
            "data": { "orders": [item] }
        });
        let mut orders = Self::parse_orders(&fake)?;
        orders.pop().ok_or_else(|| ExchangeError::Parse(
            "Subsquid order: empty result".to_string()
        ))
    }

    /// Map Subsquid order type string to our `OrderType` enum.
    ///
    /// Also returns an optional `stop_price` for stop-type orders.
    fn map_order_type(order_type_str: &str, item: &Value) -> (OrderType, Option<f64>) {
        let trigger_price: Option<f64> = item.get("triggerPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|v| v / 1e30)
            .filter(|&p| p > 0.0);

        match order_type_str {
            "LimitIncrease" | "LimitDecrease" | "LimitSwap" => {
                let price = trigger_price.unwrap_or(0.0);
                (OrderType::Limit { price }, None)
            }
            "StopLossDecrease" => {
                let stop = trigger_price.unwrap_or(0.0);
                (OrderType::StopMarket { stop_price: stop }, Some(stop))
            }
            _ => {
                // MarketIncrease, MarketDecrease, MarketSwap, Liquidation, etc.
                (OrderType::Market, None)
            }
        }
    }

    /// Map Subsquid order status string to our `OrderStatus` enum.
    fn map_order_status(status: &str) -> OrderStatus {
        match status.to_lowercase().as_str() {
            "active" | "open" => OrderStatus::Open,
            "executed" | "filled" => OrderStatus::Filled,
            "cancelled" | "canceled" => OrderStatus::Canceled,
            "frozen" | "frozen-order" => OrderStatus::Open, // frozen = paused, still live
            _ => OrderStatus::Open,
        }
    }
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
