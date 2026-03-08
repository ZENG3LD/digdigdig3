//! # Uniswap Response Parser
//!
//! Parse responses from:
//! - Trading API (JSON)
//! - The Graph Subgraph (GraphQL)
//! - Ethereum RPC (JSON-RPC)

use serde_json::Value;
use crate::core::{
    ExchangeError, ExchangeResult,
    Price, Ticker, OrderBook, Kline, Balance,
};

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

pub struct UniswapParser;

impl UniswapParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING API RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse quote response
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "quote": {
    ///     "aggregatedOutputs": [{
    ///       "amount": "1000000000",
    ///       "token": "0x..."
    ///     }],
    ///     "slippageTolerance": 0.5
    ///   }
    /// }
    /// ```
    pub fn parse_quote(response: &Value) -> ExchangeResult<Price> {
        let quote = response
            .get("quote")
            .ok_or_else(|| ExchangeError::Parse("Missing quote field".to_string()))?;

        let outputs = quote
            .get("aggregatedOutputs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing aggregatedOutputs".to_string()))?;

        let output = outputs
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty aggregatedOutputs".to_string()))?;

        let amount_str = output
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing amount".to_string()))?;

        let amount: f64 = amount_str
            .parse()
            .map_err(|e| ExchangeError::Parse(format!("Invalid amount: {}", e)))?;

        // Amount is in smallest unit (wei), convert to decimal
        // For now, assume USDC (6 decimals) - should be improved with token metadata
        let price = amount / 1_000_000.0;

        Ok(price)
    }

    /// Parse swap transaction response
    pub fn parse_swap_transaction(response: &Value) -> ExchangeResult<String> {
        let tx = response
            .get("transaction")
            .ok_or_else(|| ExchangeError::Parse("Missing transaction field".to_string()))?;

        let data = tx
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing transaction data".to_string()))?;

        Ok(data.to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUBGRAPH RESPONSES (GraphQL)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract data from GraphQL response
    fn extract_graphql_data(response: &Value) -> ExchangeResult<&Value> {
        response
            .get("data")
            .ok_or_else(|| {
                // Check for errors
                if let Some(errors) = response.get("errors") {
                    if let Some(err) = errors.as_array().and_then(|arr| arr.first()) {
                        if let Some(msg) = err.get("message").and_then(|v| v.as_str()) {
                            return ExchangeError::Api {
                                code: -1,
                                message: msg.to_string(),
                            };
                        }
                    }
                }
                ExchangeError::Parse("Missing data field in GraphQL response".to_string())
            })
    }

    /// Parse pool data from subgraph
    ///
    /// Example pool response:
    /// ```json
    /// {
    ///   "data": {
    ///     "pool": {
    ///       "token0": { "symbol": "USDC", "decimals": "6" },
    ///       "token1": { "symbol": "WETH", "decimals": "18" },
    ///       "sqrtPrice": "1234567890...",
    ///       "liquidity": "987654321...",
    ///       "feeTier": "500"
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_pool_price(response: &Value) -> ExchangeResult<Price> {
        let data = Self::extract_graphql_data(response)?;

        let pool = data
            .get("pool")
            .ok_or_else(|| ExchangeError::Parse("Missing pool field".to_string()))?;

        let sqrt_price_str = pool
            .get("sqrtPrice")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing sqrtPrice".to_string()))?;

        // Convert sqrtPriceX96 to decimal price
        let price = Self::sqrt_price_x96_to_price(sqrt_price_str)?;

        Ok(price)
    }

    /// Parse ticker from pool data
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let data = Self::extract_graphql_data(response)?;

        let pool = data
            .get("pool")
            .ok_or_else(|| ExchangeError::Parse("Missing pool field".to_string()))?;

        let sqrt_price_str = pool
            .get("sqrtPrice")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing sqrtPrice".to_string()))?;

        let price = Self::sqrt_price_x96_to_price(sqrt_price_str)?;

        let volume_str = pool
            .get("volumeUSD")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let volume: f64 = volume_str.parse().unwrap_or(0.0);

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: Some(volume),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: 0,
        })
    }

    /// Parse swaps from subgraph (for recent trades)
    ///
    /// Example swaps response:
    /// ```json
    /// {
    ///   "data": {
    ///     "swaps": [{
    ///       "timestamp": "1735680000",
    ///       "amount0": "-1000000000",
    ///       "amount1": "500000000000000000",
    ///       "amountUSD": "1000.50"
    ///     }]
    ///   }
    /// }
    /// ```
    pub fn parse_klines_from_swaps(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_graphql_data(response)?;

        let swaps = data
            .get("swaps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing swaps array".to_string()))?;

        let mut klines = Vec::new();

        for swap in swaps {
            let timestamp_str = swap
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("0");

            let timestamp: i64 = timestamp_str.parse().unwrap_or(0);

            let amount_usd_str = swap
                .get("amountUSD")
                .and_then(|v| v.as_str())
                .unwrap_or("0");

            let price: f64 = amount_usd_str.parse().unwrap_or(0.0);

            // Simplified: use swap price as OHLC
            klines.push(Kline {
                open_time: timestamp * 1000,
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 0.0,
                quote_volume: Some(0.0),
                close_time: Some(timestamp * 1000),
                trades: Some(1),
            });
        }

        Ok(klines)
    }

    /// Parse orderbook from pool liquidity
    ///
    /// Note: Uniswap V3 doesn't have a traditional orderbook.
    /// We simulate it from pool liquidity data.
    pub fn parse_orderbook_from_pool(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_graphql_data(response)?;

        let pool = data
            .get("pool")
            .ok_or_else(|| ExchangeError::Parse("Missing pool field".to_string()))?;

        let sqrt_price_str = pool
            .get("sqrtPrice")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing sqrtPrice".to_string()))?;

        let price = Self::sqrt_price_x96_to_price(sqrt_price_str)?;

        let liquidity_str = pool
            .get("liquidity")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let liquidity: f64 = liquidity_str.parse().unwrap_or(0.0);

        // Simulate orderbook with pool liquidity
        // In reality, this should calculate liquidity at different tick ranges
        let avg_depth = liquidity / 1_000_000.0; // Simplified

        Ok(OrderBook {
            bids: vec![(price * 0.999, avg_depth)], // 0.1% below
            asks: vec![(price * 1.001, avg_depth)], // 0.1% above
            timestamp: 0,
            sequence: None,
        })
    }

    /// Parse trading pairs from subgraph
    pub fn parse_trading_pairs(response: &Value) -> ExchangeResult<Vec<(String, String)>> {
        let data = Self::extract_graphql_data(response)?;

        let pools = data
            .get("pools")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing pools array".to_string()))?;

        let mut pairs = Vec::new();

        for pool in pools {
            let token0 = pool
                .get("token0")
                .and_then(|t| t.get("symbol"))
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            let token1 = pool
                .get("token1")
                .and_then(|t| t.get("symbol"))
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            pairs.push((token0.to_string(), token1.to_string()));
        }

        Ok(pairs)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ETHEREUM RPC RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse token balance from eth_call response
    pub fn parse_balance(response: &Value, symbol: &str) -> ExchangeResult<Balance> {
        let result = response
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing result field".to_string()))?;

        // Remove 0x prefix and parse hex
        let hex_str = result.trim_start_matches("0x");
        let balance_wei = u128::from_str_radix(hex_str, 16)
            .map_err(|e| ExchangeError::Parse(format!("Invalid hex balance: {}", e)))?;

        // Convert wei to decimal (assume 18 decimals for now)
        let balance = balance_wei as f64 / 1e18;

        Ok(Balance {
            asset: symbol.to_string(),
            free: balance,
            locked: 0.0,
            total: balance,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // UTILITIES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Convert sqrtPriceX96 to decimal price
    ///
    /// Formula: price = (sqrtPriceX96 / 2^96)^2
    ///
    /// Note: This returns the raw price ratio (token1/token0 in smallest units).
    /// To get human-readable price, must adjust for decimals.
    fn sqrt_price_x96_to_price(sqrt_price_str: &str) -> ExchangeResult<Price> {
        let sqrt_price: f64 = sqrt_price_str
            .parse()
            .map_err(|e| ExchangeError::Parse(format!("Invalid sqrtPrice: {}", e)))?;

        // Q64.96 format: divide by 2^96
        let q96 = 2_f64.powi(96);
        let sqrt_price_decimal = sqrt_price / q96;

        // Square to get actual price
        let price = sqrt_price_decimal * sqrt_price_decimal;

        Ok(price)
    }

    /// Convert sqrtPriceX96 to human-readable price accounting for decimals
    ///
    /// Formula from Uniswap V3:
    /// 1. raw_price = (sqrtPriceX96 / 2^96)^2  (this is token1/token0 ratio in smallest units)
    /// 2. To get token1 price (in token0): (1/raw_price) * 10^(token1_decimals - token0_decimals)
    /// 3. To get token0 price (in token1): raw_price * 10^(token0_decimals - token1_decimals)
    ///
    /// Example: USDC/WETH pool (token0=USDC 6 decimals, token1=WETH 18 decimals)
    /// - raw_price = token1/token0 in smallest units = wei_WETH / micro_USDC
    /// - WETH price in USDC = (1/raw_price) * 10^(18-6) = $2409
    /// - USDC price in WETH = raw_price * 10^(6-18) = 0.000415 ETH per USDC
    pub fn sqrt_price_x96_to_human_price(
        sqrt_price_x96: u128,
        token0_decimals: u8,
        token1_decimals: u8,
        want_token1_price: bool,
    ) -> ExchangeResult<Price> {
        // Calculate raw price (token1/token0 in smallest units)
        let q96 = 2_f64.powi(96);
        let sqrt_price = sqrt_price_x96 as f64 / q96;
        let raw_price = sqrt_price * sqrt_price;

        if raw_price == 0.0 {
            return Err(ExchangeError::Parse("Zero raw price".to_string()));
        }

        // Calculate human-readable price
        let price = if want_token1_price {
            // Want token1 price (in token0 terms)
            // Example: WETH price in USDC
            // price = (1/raw_price) * 10^(token1_decimals - token0_decimals)
            let decimal_adjustment = 10_f64.powi(token1_decimals as i32 - token0_decimals as i32);
            (1.0 / raw_price) * decimal_adjustment
        } else {
            // Want token0 price (in token1 terms)
            // Example: USDC price in WETH
            // price = raw_price * 10^(token0_decimals - token1_decimals)
            let decimal_adjustment = 10_f64.powi(token0_decimals as i32 - token1_decimals as i32);
            raw_price * decimal_adjustment
        };

        Ok(price)
    }

    /// Check for error in response
    pub fn check_response(response: &Value) -> ExchangeResult<()> {
        // Check for HTTP-level errors
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");

            let code = error
                .get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(-1) as i32;

            return Err(ExchangeError::Api {
                code,
                message: message.to_string(),
            });
        }

        // Check for GraphQL errors
        if let Some(errors) = response.get("errors") {
            if let Some(err) = errors.as_array().and_then(|arr| arr.first()) {
                let message = err
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GraphQL error");

                return Err(ExchangeError::Api {
                    code: -1,
                    message: message.to_string(),
                });
            }
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_quote() {
        let response = json!({
            "quote": {
                "aggregatedOutputs": [{
                    "amount": "1000000000",
                    "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
                }]
            }
        });

        let price = UniswapParser::parse_quote(&response).unwrap();
        assert_eq!(price, 1000.0);
    }

    #[test]
    fn test_sqrt_price_conversion() {
        // Example: USDC/WETH pool with price around 2000 USDC per ETH
        // sqrtPriceX96 ≈ 1234567890123456789012345678
        let sqrt_price_str = "1234567890123456789012345678";
        let price = UniswapParser::sqrt_price_x96_to_price(sqrt_price_str).unwrap();
        assert!(price > 0.0);
    }

    #[test]
    fn test_parse_balance() {
        let response = json!({
            "result": "0x0de0b6b3a7640000" // 1 ETH in wei
        });

        let balance = UniswapParser::parse_balance(&response, "WETH").unwrap();
        assert_eq!(balance.asset, "WETH");
        assert!((balance.free - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_check_response_error() {
        let response = json!({
            "error": {
                "code": 400,
                "message": "Invalid request"
            }
        });

        let result = UniswapParser::check_response(&response);
        assert!(result.is_err());
    }
}
