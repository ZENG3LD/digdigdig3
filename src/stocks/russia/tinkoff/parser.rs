//! Tinkoff Invest response parsers
//!
//! Parse JSON responses to domain types.
//!
//! ## Special Tinkoff Types
//!
//! ### Quotation
//! Prices use special format: `{units: int64, nano: int32}`
//! Example: 150.25 = {units: 150, nano: 250000000}
//! Precision: 9 decimal places
//!
//! ### MoneyValue
//! Money amounts: `{currency: string, units: int64, nano: int32}`
//! Example: 1000.50 RUB = {currency: "RUB", units: 1000, nano: 500000000}
//!
//! ### Timestamps
//! ISO 8601 format in UTC: "2026-01-26T10:30:45.123456Z"

use serde_json::Value;
use crate::core::types::*;

pub struct TinkoffParser;

impl TinkoffParser {
    // ═══════════════════════════════════════════════════════════════════════
    // QUOTATION/MONEY CONVERTERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse Quotation type to f64
    ///
    /// Quotation: {units: int64, nano: int32}
    /// Result: units + nano / 1_000_000_000.0
    fn parse_quotation(quotation: &Value) -> ExchangeResult<f64> {
        let units = Self::require_i64(quotation, "units")?;
        let nano = quotation.get("nano")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Convert to f64: units + nano / 1 billion
        let value = units as f64 + (nano as f64 / 1_000_000_000.0);
        Ok(value)
    }

    /// Parse MoneyValue type to f64
    ///
    /// MoneyValue: {currency: string, units: int64, nano: int32}
    fn parse_money_value(money: &Value) -> ExchangeResult<f64> {
        Self::parse_quotation(money)
    }

    /// Parse ISO 8601 timestamp to Unix milliseconds
    fn parse_timestamp(timestamp_str: &str) -> ExchangeResult<i64> {
        // Use chrono to parse ISO 8601
        use chrono::{DateTime, Utc};
        let dt = DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|e| ExchangeError::Parse(format!("Invalid timestamp: {}", e)))?;
        Ok(dt.with_timezone(&Utc).timestamp_millis())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STANDARD MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse GetLastPrices response
    ///
    /// Response: {lastPrices: [{figi, price: {units, nano}, time, instrumentUid}]}
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let last_prices = response
            .get("lastPrices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'lastPrices' array".to_string()))?;

        if last_prices.is_empty() {
            return Err(ExchangeError::Parse("Empty lastPrices array".to_string()));
        }

        let first_price = &last_prices[0];
        let price_obj = first_price
            .get("price")
            .ok_or_else(|| ExchangeError::Parse("Missing 'price' field".to_string()))?;

        Self::parse_quotation(price_obj)
    }

    /// Parse GetOrderBook response to Ticker
    ///
    /// Order book includes lastPrice, bids, asks which we can use for ticker data
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let last_price = response
            .get("lastPrice")
            .map(Self::parse_quotation)
            .transpose()?
            .ok_or_else(|| ExchangeError::Parse("Missing lastPrice".to_string()))?;

        let empty_vec = vec![];
        let bids = response
            .get("bids")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let asks = response
            .get("asks")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let bid_price = if !bids.is_empty() {
            bids[0].get("price").and_then(|p| Self::parse_quotation(p).ok())
        } else {
            None
        };

        let ask_price = if !asks.is_empty() {
            asks[0].get("price").and_then(|p| Self::parse_quotation(p).ok())
        } else {
            None
        };

        let timestamp_str = Self::get_str(response, "lastPriceTs")
            .or_else(|| Self::get_str(response, "orderbookTs"))
            .unwrap_or("");

        let timestamp = if !timestamp_str.is_empty() {
            Self::parse_timestamp(timestamp_str)?
        } else {
            chrono::Utc::now().timestamp_millis()
        };

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    /// Parse GetCandles response
    ///
    /// Response: {candles: [{open: {units, nano}, high, low, close, volume, time, isComplete}]}
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let candles = response
            .get("candles")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'candles' array".to_string()))?;

        candles.iter().map(|candle| {
            let open = Self::parse_quotation(
                candle.get("open")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'open'".to_string()))?
            )?;
            let high = Self::parse_quotation(
                candle.get("high")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'high'".to_string()))?
            )?;
            let low = Self::parse_quotation(
                candle.get("low")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'low'".to_string()))?
            )?;
            let close = Self::parse_quotation(
                candle.get("close")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'close'".to_string()))?
            )?;
            let volume = Self::get_i64(candle, "volume")
                .map(|v| v as f64)
                .unwrap_or(0.0);

            let time_str = Self::get_str(candle, "time")
                .ok_or_else(|| ExchangeError::Parse("Missing 'time'".to_string()))?;
            let open_time = Self::parse_timestamp(time_str)?;

            Ok(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                quote_volume: None,
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    /// Parse GetOrderBook response
    ///
    /// Response: {figi, depth, bids: [{price, quantity}], asks: [{price, quantity}], ...}
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let bids = Self::parse_order_levels(response.get("bids"))?;
        let asks = Self::parse_order_levels(response.get("asks"))?;

        let timestamp_str = Self::get_str(response, "orderbookTs")
            .unwrap_or("");
        let timestamp = if !timestamp_str.is_empty() {
            Self::parse_timestamp(timestamp_str)?
        } else {
            chrono::Utc::now().timestamp_millis()
        };

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence: None,
        })
    }

    /// Parse Shares (stocks list) response
    ///
    /// Response: {instruments: [{ticker, figi, ...}]}
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let instruments = response
            .get("instruments")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'instruments' array".to_string()))?;

        Ok(instruments.iter()
            .filter_map(|inst| inst.get("ticker").and_then(|t| t.as_str()))
            .map(|s| s.to_string())
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING & ACCOUNT DATA
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse PostOrder response
    ///
    /// Response: {orderId, executionReportStatus, lotsRequested, lotsExecuted, ...}
    pub fn parse_order_result(response: &Value) -> ExchangeResult<Order> {
        let order_id = Self::get_str(response, "orderId")
            .ok_or_else(|| ExchangeError::Parse("Missing orderId".to_string()))?
            .to_string();

        let status_str = Self::get_str(response, "executionReportStatus")
            .unwrap_or("EXECUTION_REPORT_STATUS_UNSPECIFIED");

        let status = Self::parse_order_status(status_str);

        let lots_requested = Self::get_i64(response, "lotsRequested")
            .unwrap_or(0) as f64;
        let lots_executed = Self::get_i64(response, "lotsExecuted")
            .unwrap_or(0) as f64;

        let price = response.get("initialOrderPrice")
            .and_then(|p| Self::parse_money_value(p).ok());

        let executed_price = response.get("executedOrderPrice")
            .and_then(|p| Self::parse_money_value(p).ok());

        let direction_str = Self::get_str(response, "direction")
            .unwrap_or("ORDER_DIRECTION_BUY");
        let side = if direction_str.contains("SELL") {
            OrderSide::Sell
        } else {
            OrderSide::Buy
        };

        let order_type_str = Self::get_str(response, "orderType")
            .unwrap_or("ORDER_TYPE_MARKET");
        let order_type = if order_type_str.contains("LIMIT") {
            OrderType::Limit
        } else {
            OrderType::Market
        };

        let timestamp = chrono::Utc::now().timestamp_millis();

        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: String::new(), // Will be filled by caller
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity: lots_requested,
            filled_quantity: lots_executed,
            average_price: executed_price,
            commission: None,
            commission_asset: None,
            created_at: timestamp,
            updated_at: Some(timestamp),
            time_in_force: TimeInForce::GTC,
        })
    }

    /// Parse order status from Tinkoff enum string
    fn parse_order_status(status: &str) -> OrderStatus {
        match status {
            "EXECUTION_REPORT_STATUS_NEW" => OrderStatus::New,
            "EXECUTION_REPORT_STATUS_FILL" => OrderStatus::Filled,
            "EXECUTION_REPORT_STATUS_PARTIALLYFILL" => OrderStatus::PartiallyFilled,
            "EXECUTION_REPORT_STATUS_CANCELLED" => OrderStatus::Canceled,
            "EXECUTION_REPORT_STATUS_REJECTED" => OrderStatus::Rejected,
            _ => OrderStatus::Open,
        }
    }

    /// Parse GetPositions response
    ///
    /// Response: {money: [{currency, balance, blocked}], securities: [{figi, balance, blocked}]}
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let mut balances = Vec::new();

        // Parse cash balances
        if let Some(money) = response.get("money").and_then(|m| m.as_array()) {
            for item in money {
                let currency = Self::get_str(item, "currency")
                    .unwrap_or("RUB")
                    .to_string();
                let free = item.get("balance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let locked = item.get("blocked")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                balances.push(Balance {
                    asset: currency,
                    free,
                    locked,
                    total: free + locked,
                });
            }
        }

        Ok(balances)
    }

    /// Parse GetPortfolio response
    ///
    /// Response: {positions: [{figi, quantity, averagePositionPrice, currentPrice, expectedYield}]}
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let positions_arr = response
            .get("positions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'positions' array".to_string()))?;

        positions_arr.iter().map(|pos| {
            let symbol = Self::get_str(pos, "figi")
                .unwrap_or("")
                .to_string();

            let quantity = pos.get("quantity")
                .and_then(|q| Self::parse_quotation(q).ok())
                .unwrap_or(0.0);

            let entry_price = pos.get("averagePositionPrice")
                .and_then(|p| Self::parse_money_value(p).ok())
                .unwrap_or(0.0);

            let mark_price = pos.get("currentPrice")
                .and_then(|p| Self::parse_money_value(p).ok());

            let unrealized_pnl = pos.get("expectedYield")
                .and_then(|y| Self::parse_quotation(y).ok())
                .unwrap_or(0.0);

            Ok(Position {
                symbol,
                side: if quantity >= 0.0 { PositionSide::Long } else { PositionSide::Short },
                quantity: quantity.abs(),
                entry_price,
                mark_price,
                unrealized_pnl,
                realized_pnl: None,
                liquidation_price: None,
                leverage: 1, // Tinkoff stocks don't use leverage
                margin_type: MarginType::Cross,
                margin: None,
                take_profit: None,
                stop_loss: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn parse_order_levels(value: Option<&Value>) -> ExchangeResult<Vec<(f64, f64)>> {
        let array = value
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Invalid order levels".to_string()))?;

        array.iter().map(|level| {
            let price_obj = level.get("price")
                .ok_or_else(|| ExchangeError::Parse("Missing price in level".to_string()))?;
            let price = Self::parse_quotation(price_obj)?;

            let quantity = level.get("quantity")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ExchangeError::Parse("Missing quantity in level".to_string()))?
                as f64;

            Ok((price, quantity))
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_quotation() {
        let quotation = json!({
            "units": 150,
            "nano": 250000000
        });

        let result = TinkoffParser::parse_quotation(&quotation).unwrap();
        assert!((result - 150.25).abs() < 0.000001);
    }

    #[test]
    fn test_parse_quotation_zero_nano() {
        let quotation = json!({
            "units": 100,
            "nano": 0
        });

        let result = TinkoffParser::parse_quotation(&quotation).unwrap();
        assert!((result - 100.0).abs() < 0.000001);
    }

    #[test]
    fn test_parse_money_value() {
        let money = json!({
            "currency": "RUB",
            "units": 1000,
            "nano": 500000000
        });

        let result = TinkoffParser::parse_money_value(&money).unwrap();
        assert!((result - 1000.5).abs() < 0.000001);
    }
}
