//! # Interactive Brokers Response Parsers
//!
//! Parse JSON responses from IB Client Portal Web API to domain types.
//!
//! IB uses field IDs (numeric keys) for market data instead of named fields.
//! This module maps those field IDs to our domain types.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult, Kline, OrderBook, Ticker,
};

/// IB response parser
pub struct IBParser;

impl IBParser {
    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price from market data snapshot
    ///
    /// Field ID 31 = Last price
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        // IB returns array of snapshots for multiple contracts
        let snapshots = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of snapshots".to_string()))?;

        if snapshots.is_empty() {
            return Err(ExchangeError::Parse("Empty snapshot array".to_string()));
        }

        let snapshot = &snapshots[0];

        // Field 31 is last price
        Self::get_f64(snapshot, "31")
            .ok_or_else(|| ExchangeError::Parse("Missing field 31 (last price)".to_string()))
    }

    /// Parse ticker from market data snapshot
    ///
    /// IB uses numeric field IDs:
    /// - 31: Last price
    /// - 55: Symbol
    /// - 70: High (session)
    /// - 71: Low (session)
    /// - 84: Bid price
    /// - 85: Ask size
    /// - 86: Ask price
    /// - 87: Volume
    /// - 88: Bid size
    /// - 7219: Prior close
    /// - _updated: Timestamp (milliseconds)
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        // IB returns array of snapshots
        let snapshots = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of snapshots".to_string()))?;

        if snapshots.is_empty() {
            return Err(ExchangeError::Parse("Empty snapshot array".to_string()));
        }

        let snapshot = &snapshots[0];

        let last_price = Self::require_f64(snapshot, "31")?;
        let timestamp = Self::get_i64(snapshot, "_updated").unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before UNIX epoch")
                .as_millis() as i64
        });

        // Calculate 24h change from prior close
        let prior_close = Self::get_f64(snapshot, "7219");
        let (price_change_24h, price_change_percent_24h) = if let Some(prior) = prior_close {
            let change = last_price - prior;
            let change_pct = if prior != 0.0 {
                (change / prior) * 100.0
            } else {
                0.0
            };
            (Some(change), Some(change_pct))
        } else {
            (None, None)
        };

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: Self::get_f64(snapshot, "84"),
            ask_price: Self::get_f64(snapshot, "86"),
            high_24h: Self::get_f64(snapshot, "70"),
            low_24h: Self::get_f64(snapshot, "71"),
            volume_24h: Self::get_f64(snapshot, "87"),
            quote_volume_24h: None, // Not provided by IB
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        })
    }

    /// Parse klines (historical data) from IB response
    ///
    /// IB format:
    /// ```json
    /// {
    ///   "data": [
    ///     { "t": 1706268600000, "o": 185.00, "c": 185.25, "h": 185.50, "l": 184.90, "v": 125000 }
    ///   ]
    /// }
    /// ```
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'data' field".to_string()))?;

        data.iter()
            .map(|bar| {
                Ok(Kline {
                    open_time: Self::require_i64(bar, "t")?,
                    open: Self::require_f64(bar, "o")?,
                    high: Self::require_f64(bar, "h")?,
                    low: Self::require_f64(bar, "l")?,
                    close: Self::require_f64(bar, "c")?,
                    volume: Self::require_f64(bar, "v")?,
                    quote_volume: None,
                    close_time: None,
                    trades: None,
                })
            })
            .collect()
    }

    /// Parse orderbook (depth of market)
    ///
    /// Note: IB may not provide orderbook via snapshot endpoint.
    /// This is a placeholder for if/when depth data is available.
    #[allow(dead_code)]
    pub fn parse_orderbook(_response: &Value) -> ExchangeResult<OrderBook> {
        // IB doesn't typically provide full orderbook in snapshot
        // This would need to be implemented if IB provides depth data
        Err(ExchangeError::UnsupportedOperation(
            "IB does not provide orderbook via snapshot endpoint".to_string(),
        ))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONTRACT SEARCH PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse contract search results
    ///
    /// Returns list of (conid, symbol, company_name) tuples
    pub fn parse_contract_search(
        response: &Value,
    ) -> ExchangeResult<Vec<(i64, String, String)>> {
        let contracts = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of contracts".to_string()))?;

        contracts
            .iter()
            .map(|contract| {
                let conid = Self::require_i64(contract, "conid")?;
                let symbol = Self::get_str(contract, "symbol")
                    .unwrap_or_default()
                    .to_string();
                let company_name = Self::get_str(contract, "companyName")
                    .unwrap_or_default()
                    .to_string();
                Ok((conid, symbol, company_name))
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCOUNT & POSITION PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse account list
    #[allow(dead_code)]
    pub fn parse_accounts(response: &Value) -> ExchangeResult<Vec<String>> {
        let accounts = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of accounts".to_string()))?;

        Ok(accounts
            .iter()
            .filter_map(|acc| {
                acc.get("accountId")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            })
            .collect())
    }

    /// Parse positions from portfolio endpoint
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<IBPosition>> {
        let positions = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        positions
            .iter()
            .map(|pos| {
                Ok(IBPosition {
                    conid: Self::require_i64(pos, "conid")?,
                    symbol: Self::get_str(pos, "contractDesc")
                        .unwrap_or_default()
                        .to_string(),
                    position: Self::require_f64(pos, "position")?,
                    avg_price: Self::require_f64(pos, "avgPrice")?,
                    market_price: Self::require_f64(pos, "mktPrice")?,
                    market_value: Self::require_f64(pos, "mktValue")?,
                    unrealized_pnl: Self::get_f64(pos, "unrealizedPnl").unwrap_or(0.0),
                    realized_pnl: Self::get_f64(pos, "realizedPnl").unwrap_or(0.0),
                    currency: Self::get_str(pos, "currency")
                        .unwrap_or("USD")
                        .to_string(),
                })
            })
            .collect()
    }

    /// Parse account summary
    pub fn parse_account_summary(response: &Value) -> ExchangeResult<IBAccountSummary> {
        // Account summary has nested structure with amount/currency fields
        let net_liq = response
            .get("netliquidation")
            .and_then(|v| v.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let total_cash = response
            .get("totalcashvalue")
            .and_then(|v| v.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let buying_power = response
            .get("buyingpower")
            .and_then(|v| v.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let equity = response
            .get("equity")
            .and_then(|v| v.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let unrealized_pnl = response
            .get("unrealizedpnl")
            .and_then(|v| v.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let realized_pnl = response
            .get("realizedpnl")
            .and_then(|v| v.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        Ok(IBAccountSummary {
            net_liquidation: net_liq,
            total_cash_value: total_cash,
            buying_power,
            equity,
            unrealized_pnl,
            realized_pnl,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ORDER PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse order response
    #[allow(dead_code)]
    pub fn parse_order_response(response: &Value) -> ExchangeResult<IBOrderResponse> {
        // Check if this is a confirmation request
        if response.get("id").is_some() && response.get("message").is_some() {
            let reply_id = Self::get_str(response, "id")
                .ok_or_else(|| ExchangeError::Parse("Missing reply ID".to_string()))?
                .to_string();

            return Ok(IBOrderResponse {
                requires_confirmation: true,
                reply_id: Some(reply_id),
                order_id: None,
                status: None,
            });
        }

        // Otherwise parse as final response
        let results = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        if results.is_empty() {
            return Err(ExchangeError::Parse("Empty order response".to_string()));
        }

        let result = &results[0];

        let order_id = Self::get_str(result, "order_id")
            .map(str::to_string);

        let status = Self::get_str(result, "order_status")
            .map(str::to_string);

        Ok(IBOrderResponse {
            requires_confirmation: false,
            reply_id: None,
            order_id,
            status,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| {
                v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .ok_or_else(|| {
                ExchangeError::Parse(format!("Missing/invalid field '{}'", field))
            })
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| {
                v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .ok_or_else(|| {
                ExchangeError::Parse(format!("Missing/invalid field '{}'", field))
            })
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IB-SPECIFIC DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

/// IB position data
#[derive(Debug, Clone)]
pub struct IBPosition {
    pub conid: i64,
    pub symbol: String,
    pub position: f64,
    pub avg_price: f64,
    pub market_price: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub currency: String,
}

/// IB account summary data
#[derive(Debug, Clone)]
pub struct IBAccountSummary {
    pub net_liquidation: f64,
    pub total_cash_value: f64,
    pub buying_power: f64,
    pub equity: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

/// IB order response
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IBOrderResponse {
    pub requires_confirmation: bool,
    pub reply_id: Option<String>,
    pub order_id: Option<String>,
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!([
            {
                "conid": 265598,
                "31": 185.50
            }
        ]);

        let price = IBParser::parse_price(&response).unwrap();
        assert_eq!(price, 185.50);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!([
            {
                "conid": 265598,
                "31": 185.50,
                "84": 185.48,
                "86": 185.52,
                "70": 186.50,
                "71": 184.20,
                "87": 55234000.0,
                "7219": 180.00,
                "_updated": 1706282450123i64
            }
        ]);

        let ticker = IBParser::parse_ticker(&response, "AAPL").unwrap();
        assert_eq!(ticker.symbol, "AAPL");
        assert_eq!(ticker.last_price, 185.50);
        assert_eq!(ticker.bid_price, Some(185.48));
        assert_eq!(ticker.ask_price, Some(185.52));
        assert_eq!(ticker.high_24h, Some(186.50));
        assert_eq!(ticker.low_24h, Some(184.20));
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "data": [
                { "t": 1706268600000i64, "o": 185.00, "c": 185.25, "h": 185.50, "l": 184.90, "v": 125000.0 },
                { "t": 1706268900000i64, "o": 185.25, "c": 185.10, "h": 185.40, "l": 185.00, "v": 98000.0 }
            ]
        });

        let klines = IBParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 2);
        assert_eq!(klines[0].open, 185.00);
        assert_eq!(klines[0].close, 185.25);
        assert_eq!(klines[1].volume, 98000.0);
    }

    #[test]
    fn test_parse_contract_search() {
        let response = json!([
            {
                "conid": 265598,
                "symbol": "AAPL",
                "companyName": "Apple Inc"
            },
            {
                "conid": 8314,
                "symbol": "SPY",
                "companyName": "SPDR S&P 500 ETF"
            }
        ]);

        let contracts = IBParser::parse_contract_search(&response).unwrap();
        assert_eq!(contracts.len(), 2);
        assert_eq!(contracts[0].0, 265598);
        assert_eq!(contracts[0].1, "AAPL");
        assert_eq!(contracts[0].2, "Apple Inc");
    }
}
