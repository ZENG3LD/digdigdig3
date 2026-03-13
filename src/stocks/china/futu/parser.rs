//! Futu OpenAPI response parsers
//!
//! Futu uses Protocol Buffers over TCP for all communications. In production
//! these parsers would consume decoded protobuf structs.  In this stub
//! implementation the parsers operate on `serde_json::Value` representing
//! the same fields, allowing business logic to be validated before the
//! actual protobuf transport is connected.
//!
//! ## Field mapping (proto → our types)
//!
//! **Order (Trd_Common.Order)**
//! ```text
//! orderID          → id
//! code             → symbol
//! trdSide          → side  (1=Buy, 2=Sell, 3=SellShort, 4=BuyBack)
//! orderType        → order_type
//! orderStatus      → status
//! price            → price
//! auxPrice         → stop_price
//! qty              → quantity
//! fillQty          → filled_quantity
//! fillAvgPrice     → average_price
//! createTimestamp  → created_at  (seconds → ms)
//! updateTimestamp  → updated_at  (seconds → ms)
//! ```
//!
//! **Funds (Trd_GetFunds.S2C.Funds)**
//! ```text
//! cash             → Balance.free  (asset="CASH")
//! totalAssets      → Balance total composite
//! marketVal        → locked (in positions)
//! power            → buying power annotation
//! ```
//!
//! **Position (Trd_GetPositionList.S2C.Position)**
//! ```text
//! code             → symbol
//! positionSide     → side  (1=Long, 2=Short)
//! qty              → quantity
//! costPrice        → entry_price
//! curPrice         → mark_price
//! unrealizedPL     → unrealized_pnl
//! realizedPL       → realized_pnl
//! ```

use serde_json::Value;
use crate::core::types::*;
use super::endpoints::order_status;

pub struct FutuParser;

impl FutuParser {
    // ─────────────────────────────────────────────────────────────────────────
    // HELPERS
    // ─────────────────────────────────────────────────────────────────────────

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)?.as_f64()
            .or_else(|| obj.get(field)?.as_str()?.parse().ok())
    }

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        Self::get_f64(obj, field).ok_or_else(|| {
            ExchangeError::Parse(format!("missing or invalid float field '{}'", field))
        })
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)?.as_i64()
            .or_else(|| obj.get(field)?.as_str()?.parse().ok())
    }

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        Self::get_i64(obj, field).ok_or_else(|| {
            ExchangeError::Parse(format!("missing or invalid integer field '{}'", field))
        })
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field)?.as_str()
    }

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        Self::get_str(obj, field).ok_or_else(|| {
            ExchangeError::Parse(format!("missing or invalid string field '{}'", field))
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CHECK RETTYPE
    // ─────────────────────────────────────────────────────────────────────────

    /// Verify the top-level retType == 0 (success).
    /// Returns the s2c sub-object on success.
    pub fn check_response(response: &Value) -> ExchangeResult<&Value> {
        let ret_type = Self::get_i64(response, "retType").unwrap_or(-1);
        if ret_type != 0 {
            let msg = Self::get_str(response, "retMsg")
                .unwrap_or("unknown error")
                .to_string();
            let code = Self::get_i64(response, "errCode").unwrap_or(ret_type) as i32;
            return Err(ExchangeError::Api { code, message: msg });
        }
        response.get("s2c").ok_or_else(|| {
            ExchangeError::Parse("response missing 's2c' field".to_string())
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ORDER STATUS
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_order_status(code: i32) -> OrderStatus {
        match code {
            order_status::SUBMITTED => OrderStatus::Open,
            order_status::WAITING_SUBMIT | order_status::SUBMITTING => OrderStatus::New,
            order_status::FILLED_PART => OrderStatus::PartiallyFilled,
            order_status::FILLED_ALL => OrderStatus::Filled,
            order_status::CANCELLED_PART | order_status::CANCELLED_ALL
            | order_status::CANCELLING_PART | order_status::CANCELLING_ALL => OrderStatus::Canceled,
            order_status::SUBMIT_FAILED | order_status::FAILED => OrderStatus::Rejected,
            order_status::TIMEOUT => OrderStatus::Expired,
            _ => OrderStatus::Rejected,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ORDER SIDE
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_trd_side(code: i64) -> ExchangeResult<OrderSide> {
        match code {
            1 | 4 => Ok(OrderSide::Buy),  // Buy or BuyBack
            2 | 3 => Ok(OrderSide::Sell), // Sell or SellShort
            _ => Err(ExchangeError::Parse(format!("unknown trdSide: {}", code))),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ORDER TYPE
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_order_type(code: i64, price: Option<f64>, aux_price: Option<f64>) -> OrderType {
        match code {
            1 | 2 => {
                // Normal limit / market
                match price {
                    Some(p) if p > 0.0 => OrderType::Limit { price: p },
                    _ => OrderType::Market,
                }
            }
            3 => {
                // EnhancedLimit ≈ StopMarket
                OrderType::StopMarket {
                    stop_price: aux_price.unwrap_or(0.0),
                }
            }
            4 => {
                // StopLimit
                OrderType::StopLimit {
                    stop_price: aux_price.unwrap_or(0.0),
                    limit_price: price.unwrap_or(0.0),
                }
            }
            5 => {
                // StopMarket
                OrderType::StopMarket {
                    stop_price: aux_price.unwrap_or(0.0),
                }
            }
            _ => OrderType::Limit { price: price.unwrap_or(0.0) },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PARSE SINGLE ORDER
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse a single order object from Trd_Common.Order proto fields.
    pub fn parse_order(obj: &Value) -> ExchangeResult<Order> {
        let order_id = match obj.get("orderID") {
            Some(v) => match v.as_i64() {
                Some(n) => n.to_string(),
                None => v.as_str().unwrap_or("0").to_string(),
            },
            None => return Err(ExchangeError::Parse("missing orderID".to_string())),
        };

        let symbol = Self::require_str(obj, "code")?.to_string();
        let side_code = Self::require_i64(obj, "trdSide")?;
        let side = Self::parse_trd_side(side_code)?;

        let price = Self::get_f64(obj, "price");
        let aux_price = Self::get_f64(obj, "auxPrice");
        let order_type_code = Self::get_i64(obj, "orderType").unwrap_or(1);
        let order_type = Self::parse_order_type(order_type_code, price, aux_price);

        let status_code = Self::get_i64(obj, "orderStatus").unwrap_or(-1) as i32;
        let status = Self::parse_order_status(status_code);

        let quantity = Self::require_f64(obj, "qty")?;
        let filled_quantity = Self::get_f64(obj, "fillQty").unwrap_or(0.0);
        let average_price = Self::get_f64(obj, "fillAvgPrice");

        // Timestamps: Futu returns Unix seconds (float), convert to ms
        let created_at = Self::get_f64(obj, "createTimestamp")
            .map(|ts| (ts * 1000.0) as i64)
            .unwrap_or(0);
        let updated_at = Self::get_f64(obj, "updateTimestamp")
            .map(|ts| (ts * 1000.0) as i64);

        let client_order_id = obj.get("remark")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(Order {
            id: order_id,
            client_order_id,
            symbol,
            side,
            order_type,
            status,
            price,
            stop_price: aux_price,
            quantity,
            filled_quantity,
            average_price,
            commission: None,
            commission_asset: None,
            created_at,
            updated_at,
            time_in_force: TimeInForce::Gtc,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PARSE ORDER LIST
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse the orderList array from Trd_GetOrderList.S2C
    pub fn parse_order_list(s2c: &Value) -> ExchangeResult<Vec<Order>> {
        let arr = s2c.get("orderList")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("missing orderList".to_string()))?;

        let mut orders = Vec::with_capacity(arr.len());
        for item in arr {
            orders.push(Self::parse_order(item)?);
        }
        Ok(orders)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PARSE PLACE ORDER RESPONSE
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse the response from Trd_PlaceOrder (proto_id 2202).
    ///
    /// S2C contains:
    /// - orderID: u64
    /// - order: Order (optional, some versions return it inline)
    pub fn parse_place_order(s2c: &Value, request_symbol: &str) -> ExchangeResult<Order> {
        // Try to get the embedded order object first
        if let Some(order_obj) = s2c.get("order") {
            if order_obj.is_object() {
                return Self::parse_order(order_obj);
            }
        }

        // Fallback: construct a minimal Order from the returned orderID
        let order_id = match s2c.get("orderID") {
            Some(v) => match v.as_i64() {
                Some(n) => n.to_string(),
                None => v.as_str().unwrap_or("0").to_string(),
            },
            None => return Err(ExchangeError::Parse("missing orderID in PlaceOrder response".to_string())),
        };

        // Return a minimal in-flight order — full state requires a subsequent GetOrderList call
        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: request_symbol.to_string(),
            side: OrderSide::Buy, // placeholder; caller should refresh
            order_type: OrderType::Market,
            status: OrderStatus::New,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PARSE FUNDS (BALANCE)
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse Trd_GetFunds.S2C.funds into a Vec<Balance>.
    ///
    /// Futu returns a single Funds object per account, not per-asset.
    /// We produce a synthetic balance list:
    /// - "CASH" → free cash
    /// - "SECURITIES" → value locked in positions
    pub fn parse_funds(s2c: &Value, currency: &str) -> ExchangeResult<Vec<Balance>> {
        let funds = s2c.get("funds")
            .ok_or_else(|| ExchangeError::Parse("missing 'funds' in GetFunds response".to_string()))?;

        let cash = Self::get_f64(funds, "cash").unwrap_or(0.0);
        let market_val = Self::get_f64(funds, "marketVal").unwrap_or(0.0);
        let total_assets = Self::get_f64(funds, "totalAssets").unwrap_or(cash + market_val);

        Ok(vec![
            Balance {
                asset: currency.to_string(),
                free: cash,
                locked: market_val,
                total: total_assets,
            },
        ])
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PARSE ACCOUNT INFO
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse Trd_GetFunds.S2C into an AccountInfo.
    pub fn parse_account_info(s2c: &Value, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let funds = s2c.get("funds")
            .ok_or_else(|| ExchangeError::Parse("missing 'funds' in GetFunds response".to_string()))?;

        let cash = Self::get_f64(funds, "cash").unwrap_or(0.0);
        let market_val = Self::get_f64(funds, "marketVal").unwrap_or(0.0);
        let total_assets = Self::get_f64(funds, "totalAssets").unwrap_or(cash + market_val);
        let currency = Self::get_str(funds, "currency").unwrap_or("USD").to_string();

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0008, // Futu standard 0.08% for US
            taker_commission: 0.0008,
            balances: vec![
                Balance {
                    asset: currency,
                    free: cash,
                    locked: market_val,
                    total: total_assets,
                },
            ],
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PARSE POSITIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse the positionList array from Trd_GetPositionList.S2C.
    pub fn parse_position_list(s2c: &Value) -> ExchangeResult<Vec<Position>> {
        let arr = s2c.get("positionList")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("missing positionList".to_string()))?;

        let mut positions = Vec::with_capacity(arr.len());
        for item in arr {
            positions.push(Self::parse_position(item)?);
        }
        Ok(positions)
    }

    /// Parse a single position object from Trd_Common.Position.
    fn parse_position(obj: &Value) -> ExchangeResult<Position> {
        let symbol = Self::require_str(obj, "code")?.to_string();

        // positionSide: 1=Long, 2=Short
        let pos_side_code = Self::get_i64(obj, "positionSide").unwrap_or(1);
        let side = match pos_side_code {
            2 => PositionSide::Short,
            _ => PositionSide::Long,
        };

        let quantity = Self::get_f64(obj, "qty").unwrap_or(0.0);
        let entry_price = Self::get_f64(obj, "costPrice").unwrap_or(0.0);
        let mark_price = Self::get_f64(obj, "curPrice");
        let unrealized_pnl = Self::get_f64(obj, "unrealizedPL").unwrap_or(0.0);
        let realized_pnl = Self::get_f64(obj, "realizedPL");

        Ok(Position {
            symbol,
            side,
            quantity,
            entry_price,
            mark_price,
            unrealized_pnl,
            realized_pnl,
            liquidation_price: None,
            leverage: 1, // Futu stocks are not leveraged by default
            margin_type: MarginType::Cross,
            margin: None,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MARKET DATA (stubs — protocol buffer transport not connected)
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse price from Qot_GetSecuritySnapshot.S2C
    pub fn parse_price(s2c: &Value) -> ExchangeResult<f64> {
        let snapshot_list = s2c.get("snapshotList")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("missing snapshotList".to_string()))?;
        let first = snapshot_list.first()
            .ok_or_else(|| ExchangeError::Parse("empty snapshotList".to_string()))?;
        let basic = first.get("basic")
            .ok_or_else(|| ExchangeError::Parse("missing basic in snapshot".to_string()))?;
        Self::require_f64(basic, "curPrice")
    }

    /// Parse ticker from Qot_GetSecuritySnapshot.S2C
    pub fn parse_ticker(s2c: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let snapshot_list = s2c.get("snapshotList")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("missing snapshotList".to_string()))?;
        let first = snapshot_list.first()
            .ok_or_else(|| ExchangeError::Parse("empty snapshotList".to_string()))?;
        let basic = first.get("basic")
            .ok_or_else(|| ExchangeError::Parse("missing basic in snapshot".to_string()))?;

        let last_price = Self::require_f64(basic, "curPrice")?;
        let timestamp = Self::get_f64(basic, "updateTime")
            .map(|ts| (ts * 1000.0) as i64)
            .unwrap_or(0);

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: Self::get_f64(basic, "bidPrice"),
            ask_price: Self::get_f64(basic, "askPrice"),
            high_24h: Self::get_f64(basic, "highPrice"),
            low_24h: Self::get_f64(basic, "lowPrice"),
            volume_24h: Self::get_f64(basic, "volume").map(|v| v as f64),
            quote_volume_24h: Self::get_f64(basic, "turnover"),
            price_change_24h: Self::get_f64(basic, "priceChange"),
            price_change_percent_24h: Self::get_f64(basic, "changeRate"),
            timestamp,
        })
    }

    /// Parse klines from Qot_RequestHistoryKL.S2C
    pub fn parse_klines(s2c: &Value) -> ExchangeResult<Vec<Kline>> {
        let kl_list = s2c.get("klList")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("missing klList".to_string()))?;

        let mut klines = Vec::with_capacity(kl_list.len());
        for item in kl_list {
            let open_time = Self::get_str(item, "time")
                .and_then(|s| chrono_timestamp_ms(s))
                .unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::get_f64(item, "openPrice").unwrap_or(0.0),
                high: Self::get_f64(item, "highPrice").unwrap_or(0.0),
                low: Self::get_f64(item, "lowPrice").unwrap_or(0.0),
                close: Self::get_f64(item, "closePrice").unwrap_or(0.0),
                volume: Self::get_f64(item, "volume").unwrap_or(0.0),
                quote_volume: Self::get_f64(item, "turnover"),
                close_time: None,
                trades: Self::get_i64(item, "changeRate").map(|_| 0u64), // Futu doesn't provide trade count
            });
        }
        Ok(klines)
    }

    /// Parse orderbook from Qot_GetOrderBook.S2C
    pub fn parse_orderbook(s2c: &Value) -> ExchangeResult<OrderBook> {
        let bids = parse_book_side(s2c.get("buyOrderBookList"));
        let asks = parse_book_side(s2c.get("sellOrderBookList"));

        Ok(OrderBook {
            bids,
            asks,
            timestamp: 0,
            sequence: None,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ACCOUNT LIST
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse the accList from Trd_GetAccList.S2C, returning (accID, trdMarket) pairs.
    pub fn parse_acc_list(s2c: &Value) -> ExchangeResult<Vec<(u64, i32)>> {
        let arr = s2c.get("accList")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("missing accList".to_string()))?;

        let mut result = Vec::new();
        for item in arr {
            let acc_id = item.get("accID")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let trd_market = item.get("trdMarket")
                .and_then(|v| v.as_i64())
                .unwrap_or(2) as i32;
            result.push((acc_id, trd_market));
        }
        Ok(result)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HELPERS (module-private)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse Futu's datetime string ("2024-01-15 09:30:00") into Unix ms.
/// Falls back to 0 if parsing fails (avoids hard dependency on chrono).
fn chrono_timestamp_ms(s: &str) -> Option<i64> {
    // Simple fallback: we cannot parse without chrono; return None so callers
    // use 0. Real implementation would use chrono::NaiveDateTime::parse_from_str.
    let _ = s;
    None
}

/// Parse an order book side from a JSON array of {price, volume} objects.
fn parse_book_side(value: Option<&Value>) -> Vec<(f64, f64)> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let price = item.get("price")?.as_f64()?;
                    let volume = item.get("volume")?.as_f64()
                        .or_else(|| item.get("qty")?.as_f64())
                        .unwrap_or(0.0);
                    Some((price, volume))
                })
                .collect()
        })
        .unwrap_or_default()
}
