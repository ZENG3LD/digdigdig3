//! Zerodha Kite Connect response parsers

use serde_json::Value;
use crate::core::types::*;

pub struct ZerodhaParser;

impl ZerodhaParser {
    // Extract data from response envelope
    fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        let status = response.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");

        if status == "error" {
            let message = response.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
            let error_type = response.get("error_type").and_then(|t| t.as_str()).unwrap_or("UnknownException");
            return Err(ExchangeError::Api {
                code: -1,
                message: format!("{}: {}", error_type, message),
            });
        }

        response.get("data").ok_or_else(|| ExchangeError::Parse("Missing data field".to_string()))
    }

    /// Parse LTP response
    pub fn parse_ltp(response: &Value, symbol_key: &str) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;
        data.get(symbol_key)
            .and_then(|inst| inst.get("last_price"))
            .and_then(|p| p.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing last_price for {}", symbol_key)))
    }

    /// Parse full quote
    pub fn parse_quote(response: &Value, symbol_key: &str) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;
        let inst = data.get(symbol_key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing data for {}", symbol_key)))?;

        let ohlc = inst.get("ohlc");
        let depth = inst.get("depth");

        Ok(Ticker {
            symbol: symbol_key.to_string(),
            last_price: Self::require_f64(inst, "last_price")?,
            bid_price: depth
                .and_then(|d| d.get("buy"))
                .and_then(|b| b.as_array())
                .and_then(|arr| arr.first())
                .and_then(|level| level.get("price"))
                .and_then(|p| p.as_f64()),
            ask_price: depth
                .and_then(|d| d.get("sell"))
                .and_then(|s| s.as_array())
                .and_then(|arr| arr.first())
                .and_then(|level| level.get("price"))
                .and_then(|p| p.as_f64()),
            high_24h: ohlc.and_then(|o| o.get("high")).and_then(|h| h.as_f64()),
            low_24h: ohlc.and_then(|o| o.get("low")).and_then(|l| l.as_f64()),
            volume_24h: Self::get_f64(inst, "volume"),
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(inst, "net_change"),
            price_change_percent_24h: None,
            timestamp: Self::get_i64(inst, "timestamp").unwrap_or_else(|| {
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("System time is before UNIX epoch").as_secs() as i64
            }),
        })
    }

    /// Parse orderbook
    pub fn parse_orderbook(response: &Value, symbol_key: &str) -> ExchangeResult<OrderBook> {
        let data = Self::extract_data(response)?;
        let inst = data.get(symbol_key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing data for {}", symbol_key)))?;

        let depth = inst.get("depth")
            .ok_or_else(|| ExchangeError::Parse("Missing depth field".to_string()))?;

        let bids = Self::parse_depth_levels(depth.get("buy"))?;
        let asks = Self::parse_depth_levels(depth.get("sell"))?;

        Ok(OrderBook {
            bids,
            asks,
            timestamp: Self::get_i64(inst, "timestamp").unwrap_or_else(|| {
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("System time is before UNIX epoch").as_secs() as i64
            }),
            sequence: None,
        })
    }

    fn parse_depth_levels(value: Option<&Value>) -> ExchangeResult<Vec<(f64, f64)>> {
        let array = value.and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Invalid depth levels".to_string()))?;

        let mut levels = Vec::new();
        for level in array {
            let price = level.get("price").and_then(|p| p.as_f64())
                .ok_or_else(|| ExchangeError::Parse("Missing depth price".to_string()))?;
            let quantity = level.get("quantity").and_then(|q| q.as_f64())
                .ok_or_else(|| ExchangeError::Parse("Missing depth quantity".to_string()))?;
            levels.push((price, quantity));
        }

        Ok(levels)
    }

    /// Parse historical candles
    pub fn _parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;
        let candles = data.get("candles").and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candles array".to_string()))?;

        candles.iter().map(|candle| {
            let arr = candle.as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid candle format".to_string()))?;

            if arr.len() < 6 {
                return Err(ExchangeError::Parse("Candle array too short".to_string()));
            }

            // Simplified timestamp parsing - use current time for now
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before UNIX epoch")
                .as_secs() as i64;

            Ok(Kline {
                open_time: timestamp,
                open: arr[1].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid open".to_string()))?,
                high: arr[2].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid high".to_string()))?,
                low: arr[3].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid low".to_string()))?,
                close: arr[4].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid close".to_string()))?,
                volume: arr[5].as_f64().ok_or_else(|| ExchangeError::Parse("Invalid volume".to_string()))?,
                quote_volume: None,
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    /// Parse order response
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        let data = Self::extract_data(response)?;

        // Check if simple order_id response
        if let Some(order_id_str) = data.get("order_id").and_then(|id| id.as_str()) {
            return Ok(Order {
                id: order_id_str.to_string(),
                client_order_id: None,
                symbol: String::new(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                status: OrderStatus::Open,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: 0,
                updated_at: None,
                time_in_force: TimeInForce::GTC,
            });
        }

        // Full order object
        let order_id = Self::require_str(data, "order_id")?.to_string();
        let status_str = Self::require_str(data, "status")?;
        let status = Self::parse_order_status(status_str);

        let quantity = Self::get_f64(data, "quantity").unwrap_or(0.0);
        let filled_quantity = Self::get_f64(data, "filled_quantity").unwrap_or(0.0);

        Ok(Order {
            id: order_id,
            client_order_id: Self::get_str(data, "tag").map(|s| s.to_string()),
            symbol: Self::get_str(data, "tradingsymbol").unwrap_or("").to_string(),
            side: if Self::get_str(data, "transaction_type") == Some("BUY") {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            },
            order_type: if Self::get_str(data, "order_type") == Some("MARKET") {
                OrderType::Market
            } else {
                OrderType::Limit
            },
            status,
            price: Self::get_f64(data, "price"),
            stop_price: None,
            quantity,
            filled_quantity,
            average_price: Self::get_f64(data, "average_price"),
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, "order_timestamp").unwrap_or(0),
            updated_at: Self::get_i64(data, "exchange_update_timestamp"),
            time_in_force: TimeInForce::GTC,
        })
    }

    fn parse_order_status(status_str: &str) -> OrderStatus {
        match status_str {
            "OPEN" => OrderStatus::Open,
            "COMPLETE" => OrderStatus::Filled,
            "CANCELLED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "TRIGGER PENDING" | "MODIFY PENDING" => OrderStatus::Open,
            "CANCEL PENDING" => OrderStatus::Canceled,
            _ => OrderStatus::New,
        }
    }

    /// Parse orders list
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;
        let orders_array = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        orders_array.iter().map(|order| {
            let order_id = Self::require_str(order, "order_id")?.to_string();
            let status_str = Self::require_str(order, "status")?;
            let status = Self::parse_order_status(status_str);

            let quantity = Self::get_f64(order, "quantity").unwrap_or(0.0);
            let filled_quantity = Self::get_f64(order, "filled_quantity").unwrap_or(0.0);

            Ok(Order {
                id: order_id,
                client_order_id: Self::get_str(order, "tag").map(|s| s.to_string()),
                symbol: Self::get_str(order, "tradingsymbol").unwrap_or("").to_string(),
                side: if Self::get_str(order, "transaction_type") == Some("BUY") {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                order_type: if Self::get_str(order, "order_type") == Some("MARKET") {
                    OrderType::Market
                } else {
                    OrderType::Limit
                },
                status,
                price: Self::get_f64(order, "price"),
                stop_price: None,
                quantity,
                filled_quantity,
                average_price: Self::get_f64(order, "average_price"),
                commission: None,
                commission_asset: None,
                created_at: Self::get_i64(order, "order_timestamp").unwrap_or(0),
                updated_at: Self::get_i64(order, "exchange_update_timestamp"),
                time_in_force: TimeInForce::GTC,
            })
        }).collect()
    }

    /// Parse balance
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;
        let mut balances = Vec::new();

        if let Some(equity) = data.get("equity") {
            if let Some(net) = Self::get_f64(equity, "net") {
                balances.push(Balance {
                    asset: "INR".to_string(),
                    free: net,
                    locked: 0.0,
                    total: net,
                });
            }
        }

        if balances.is_empty() {
            return Err(ExchangeError::Parse("No balance data found".to_string()));
        }

        Ok(balances)
    }

    /// Parse account info
    pub fn parse_account_info(response: &Value) -> ExchangeResult<AccountInfo> {
        let _data = Self::extract_data(response)?;
        Ok(AccountInfo {
            account_type: AccountType::Spot,
            can_trade: true,
            can_withdraw: false,
            can_deposit: true,
            maker_commission: 0.03,  // Zerodha charges 0.03% brokerage on intraday
            taker_commission: 0.03,
            balances: vec![],
        })
    }

    /// Parse positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let net_positions = data.get("net").and_then(|n| n.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing net positions array".to_string()))?;

        net_positions.iter()
            .filter(|pos| Self::get_f64(pos, "quantity").unwrap_or(0.0).abs() > 0.001)
            .map(|pos| {
                let tradingsymbol = Self::require_str(pos, "tradingsymbol")?;
                let exchange = Self::require_str(pos, "exchange")?;
                let symbol_key = format!("{}:{}", exchange, tradingsymbol);

                let quantity = Self::require_f64(pos, "quantity")?;
                let side = if quantity > 0.0 { PositionSide::Long } else { PositionSide::Short };

                Ok(Position {
                    symbol: symbol_key,
                    side,
                    quantity: quantity.abs(),
                    entry_price: Self::get_f64(pos, "average_price").unwrap_or(0.0),
                    mark_price: Some(Self::get_f64(pos, "last_price").unwrap_or(0.0)),
                    unrealized_pnl: Self::get_f64(pos, "unrealised").unwrap_or(0.0),
                    realized_pnl: Self::get_f64(pos, "realised"),
                    liquidation_price: None,
                    leverage: 1,
                    margin_type: MarginType::Cross,
                    margin: None,
                    take_profit: None,
                    stop_loss: None,
                })
            })
            .collect()
    }

    // Helper methods
    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}
