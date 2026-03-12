//! # OANDA v20 Response Parser
//!
//! JSON parsing for OANDA v20 API responses.
//!
//! Note: OANDA returns all numeric values as strings to avoid floating-point precision issues.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, AccountInfo, AccountType,
    MarginType,
};

/// OANDA response parser
pub struct OandaParser;

impl OandaParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse f64 from string (OANDA returns all numbers as strings)
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Parse f64 from field
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Parse required f64
    fn _require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Parse string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Parse required string
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Parse i64 from field
    fn _get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| {
            v.as_str().and_then(|s| s.parse().ok())
                .or_else(|| v.as_i64())
        })
    }

    /// Parse RFC3339 timestamp to milliseconds
    fn parse_timestamp(s: &str) -> Option<i64> {
        // OANDA format: "2026-01-26T12:34:56.789123456Z"
        // We need to parse this into milliseconds
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.timestamp_millis())
    }

    /// Get best bid price from pricing response
    fn get_best_bid(data: &Value) -> Option<f64> {
        data.get("bids")
            .and_then(|bids| bids.as_array())
            .and_then(|arr| arr.first())
            .and_then(|bid| bid.get("price"))
            .and_then(Self::parse_f64)
            .or_else(|| Self::get_f64(data, "closeoutBid"))
    }

    /// Get best ask price from pricing response
    fn get_best_ask(data: &Value) -> Option<f64> {
        data.get("asks")
            .and_then(|asks| asks.as_array())
            .and_then(|arr| arr.first())
            .and_then(|ask| ask.get("price"))
            .and_then(Self::parse_f64)
            .or_else(|| Self::get_f64(data, "closeoutAsk"))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse account ID from accounts list response
    pub fn parse_account_id(response: &Value) -> ExchangeResult<String> {
        let accounts = response.get("accounts")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'accounts' array".to_string()))?;

        let first = accounts.first()
            .ok_or_else(|| ExchangeError::Parse("No accounts found".to_string()))?;

        let id = Self::require_str(first, "id")?;
        Ok(id.to_string())
    }

    /// Parse account summary
    pub fn parse_account_info(response: &Value) -> ExchangeResult<AccountInfo> {
        let account = response.get("account")
            .ok_or_else(|| ExchangeError::Parse("Missing 'account' field".to_string()))?;

        // OANDA doesn't have separate spot/futures - it's all forex
        let account_type = AccountType::Spot; // Use Spot as default for forex

        // Extract balances
        let balance_usd = Self::get_f64(account, "balance").unwrap_or(0.0);
        let currency = Self::get_str(account, "currency").unwrap_or("USD");

        let balances = vec![
            Balance {
                asset: currency.to_string(),
                free: Self::get_f64(account, "marginAvailable").unwrap_or(balance_usd),
                locked: Self::get_f64(account, "marginUsed").unwrap_or(0.0),
                total: balance_usd,
            }
        ];

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0, // OANDA uses spread, not commission
            taker_commission: 0.0,
            balances,
        })
    }

    /// Parse balances from account summary
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let account = response.get("account")
            .ok_or_else(|| ExchangeError::Parse("Missing 'account' field".to_string()))?;

        let balance = Self::get_f64(account, "balance").unwrap_or(0.0);
        let margin_used = Self::get_f64(account, "marginUsed").unwrap_or(0.0);
        let margin_available = Self::get_f64(account, "marginAvailable").unwrap_or(balance);
        let currency = Self::get_str(account, "currency").unwrap_or("USD");

        Ok(vec![Balance {
            asset: currency.to_string(),
            free: margin_available,
            locked: margin_used,
            total: balance,
        }])
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from pricing response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let prices = response.get("prices")
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'prices' array".to_string()))?;

        let price_obj = prices.first()
            .ok_or_else(|| ExchangeError::Parse("Empty prices array".to_string()))?;

        // Use mid price between best bid and ask
        let bid = Self::get_best_bid(price_obj);
        let ask = Self::get_best_ask(price_obj);

        match (bid, ask) {
            (Some(b), Some(a)) => Ok((b + a) / 2.0),
            (Some(p), None) | (None, Some(p)) => Ok(p),
            _ => Err(ExchangeError::Parse("No valid price data".to_string())),
        }
    }

    /// Parse candles (klines) from instruments endpoint
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let candles = response.get("candles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'candles' array".to_string()))?;

        let mut klines = Vec::with_capacity(candles.len());

        for candle in candles {
            // Skip incomplete candles unless requested
            let complete = candle.get("complete").and_then(|c| c.as_bool()).unwrap_or(false);
            if !complete {
                continue;
            }

            let time_str = Self::require_str(candle, "time")?;
            let open_time = Self::parse_timestamp(time_str)
                .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?;

            // OANDA provides bid, ask, and mid prices. Use mid for candles
            let mid = candle.get("mid")
                .or_else(|| candle.get("bid"))
                .ok_or_else(|| ExchangeError::Parse("Missing candle data".to_string()))?;

            let volume = Self::get_f64(candle, "volume").unwrap_or(0.0);

            klines.push(Kline {
                open_time,
                open: Self::get_f64(mid, "o").unwrap_or(0.0),
                high: Self::get_f64(mid, "h").unwrap_or(0.0),
                low: Self::get_f64(mid, "l").unwrap_or(0.0),
                close: Self::get_f64(mid, "c").unwrap_or(0.0),
                volume,
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse orderbook from pricing data
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let prices = response.get("prices")
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'prices' array".to_string()))?;

        let price_obj = prices.first()
            .ok_or_else(|| ExchangeError::Parse("Empty prices array".to_string()))?;

        let time_str = Self::get_str(price_obj, "time").unwrap_or("");
        let timestamp = Self::parse_timestamp(time_str).unwrap_or(0);

        // Parse bids
        let bids = price_obj.get("bids")
            .and_then(|b| b.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|level| {
                        let price = Self::get_f64(level, "price")?;
                        let liquidity = Self::get_f64(level, "liquidity")?;
                        Some((price, liquidity))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse asks
        let asks = price_obj.get("asks")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|level| {
                        let price = Self::get_f64(level, "price")?;
                        let liquidity = Self::get_f64(level, "liquidity")?;
                        Some((price, liquidity))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(OrderBook {
            timestamp,
            bids,
            asks,
            sequence: None,
        })
    }

    /// Parse ticker data
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let prices = response.get("prices")
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'prices' array".to_string()))?;

        let price_obj = prices.first()
            .ok_or_else(|| ExchangeError::Parse("Empty prices array".to_string()))?;

        let bid = Self::get_best_bid(price_obj).unwrap_or(0.0);
        let ask = Self::get_best_ask(price_obj).unwrap_or(0.0);
        let last = (bid + ask) / 2.0;

        let time_str = Self::get_str(price_obj, "time").unwrap_or("");
        let timestamp = Self::parse_timestamp(time_str).unwrap_or(0);

        Ok(Ticker {
            symbol: Self::get_str(price_obj, "instrument").unwrap_or("").to_string(),
            last_price: last,
            bid_price: Some(bid),
            ask_price: Some(ask),
            volume_24h: None,
            quote_volume_24h: None,
            high_24h: None,
            low_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order ID from order creation response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        // OANDA returns transaction info
        let transaction = response.get("orderCreateTransaction")
            .or_else(|| response.get("orderFillTransaction"))
            .ok_or_else(|| ExchangeError::Parse("Missing order transaction".to_string()))?;

        let id = Self::require_str(transaction, "id")?;
        Ok(id.to_string())
    }

    /// Parse single order
    pub fn parse_order(response: &Value, default_symbol: &str) -> ExchangeResult<Order> {
        let order = response.get("order")
            .or_else(|| response.get("orderCreateTransaction"))
            .ok_or_else(|| ExchangeError::Parse("Missing order data".to_string()))?;

        let id = Self::require_str(order, "id")?.to_string();
        let instrument = Self::get_str(order, "instrument").unwrap_or(default_symbol);
        let order_type_str = Self::get_str(order, "type").unwrap_or("MARKET");
        let state = Self::get_str(order, "state").unwrap_or("PENDING");

        // Parse units (positive = buy, negative = sell)
        let units_str = Self::get_str(order, "units").unwrap_or("0");
        let units: f64 = units_str.parse().unwrap_or(0.0);
        let side = if units >= 0.0 { OrderSide::Buy } else { OrderSide::Sell };
        let quantity = units.abs();

        let order_type = match order_type_str {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit { price: 0.0 },
            "STOP" => OrderType::StopMarket { stop_price: 0.0 },
            "MARKET_IF_TOUCHED" => OrderType::StopLimit { stop_price: 0.0, limit_price: 0.0 },
            _ => OrderType::Market,
        };

        let status = match state {
            "PENDING" => OrderStatus::New,
            "FILLED" => OrderStatus::Filled,
            "TRIGGERED" => OrderStatus::PartiallyFilled,
            "CANCELLED" => OrderStatus::Canceled,
            _ => OrderStatus::New,
        };

        let price = Self::get_f64(order, "price");
        let create_time = Self::get_str(order, "createTime")
            .and_then(Self::parse_timestamp)
            .unwrap_or(0);

        Ok(Order {
            id,
            client_order_id: None,
            symbol: instrument.to_string(),
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity,
            filled_quantity: 0.0, // OANDA tracks via trades
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: create_time,
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse multiple orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let orders = response.get("orders")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orders' array".to_string()))?;

        let mut result = Vec::new();
        for order_val in orders {
            if let Ok(order) = Self::parse_order(&serde_json::json!({ "order": order_val }), "") {
                result.push(order);
            }
        }

        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse position
    pub fn parse_position(response: &Value) -> ExchangeResult<Position> {
        let position = response.get("position")
            .ok_or_else(|| ExchangeError::Parse("Missing 'position' field".to_string()))?;

        let instrument = Self::require_str(position, "instrument")?.to_string();

        // OANDA has separate long and short positions
        let long = position.get("long");
        let short = position.get("short");

        let (size, side) = if let Some(l) = long {
            let units = Self::get_f64(l, "units").unwrap_or(0.0);
            if units != 0.0 {
                (units.abs(), PositionSide::Long)
            } else if let Some(s) = short {
                let units = Self::get_f64(s, "units").unwrap_or(0.0);
                (units.abs(), PositionSide::Short)
            } else {
                (0.0, PositionSide::Long)
            }
        } else {
            (0.0, PositionSide::Long)
        };

        let entry_price = long
            .and_then(|l| Self::get_f64(l, "averagePrice"))
            .or_else(|| short.and_then(|s| Self::get_f64(s, "averagePrice")))
            .unwrap_or(0.0);

        let unrealized_pnl = long
            .and_then(|l| Self::get_f64(l, "unrealizedPL"))
            .or_else(|| short.and_then(|s| Self::get_f64(s, "unrealizedPL")))
            .unwrap_or(0.0);

        let margin = Self::get_f64(position, "marginUsed").unwrap_or(0.0);

        Ok(Position {
            symbol: instrument,
            side,
            quantity: size,
            entry_price,
            mark_price: None,
            unrealized_pnl,
            realized_pnl: None,
            liquidation_price: None,
            leverage: 1, // OANDA uses account-level margin, not leverage
            margin_type: MarginType::Cross, // OANDA uses cross margin
            margin: Some(margin),
            take_profit: None,
            stop_loss: None,
        })
    }

    /// Parse multiple positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let positions = response.get("positions")
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'positions' array".to_string()))?;

        let mut result = Vec::new();
        for pos_val in positions {
            if let Ok(pos) = Self::parse_position(&serde_json::json!({ "position": pos_val })) {
                // Only include positions with non-zero quantity
                if pos.quantity != 0.0 {
                    result.push(pos);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "prices": [{
                "instrument": "EUR_USD",
                "bids": [{"price": "1.12157", "liquidity": 10000000}],
                "asks": [{"price": "1.12170", "liquidity": 10000000}]
            }]
        });

        let price = OandaParser::parse_price(&response).unwrap();
        assert!((price - 1.121635).abs() < 0.000001);
    }

    #[test]
    fn test_parse_account_id() {
        let response = json!({
            "accounts": [
                {"id": "001-011-5838423-001", "tags": []}
            ]
        });

        let account_id = OandaParser::parse_account_id(&response).unwrap();
        assert_eq!(account_id, "001-011-5838423-001");
    }
}
