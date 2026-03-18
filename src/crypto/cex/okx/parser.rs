//! # OKX Response Parser
//!
//! Парсинг JSON ответов от OKX API v5.
//!
//! ## OKX Response Format
//!
//! Все ответы имеют структуру:
//! ```json
//! {
//!   "code": "0",
//!   "msg": "",
//!   "data": [...]
//! }
//! ```
//!
//! - `code`: "0" = success, other = error
//! - `msg`: Error message (empty on success)
//! - `data`: Always array, even for single object

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, TimeInForce, MarginType,
    FundingRate, PublicTrade, TradeSide,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    SymbolInfo, CancelAllResponse, OrderResult,
    UserTrade,
    FundingPayment, LedgerEntry, LedgerEntryType,
};
use crate::core::types::AlgoOrderResponse;
use crate::core::types::{
    TransferResponse, DepositAddress, WithdrawResponse, FundsRecord,
    SubAccountResult, SubAccount,
};

/// Order book level pairs (price, quantity)
type OrderBookLevels = Vec<(f64, f64)>;

/// Parsed order book bids and asks
type OrderBookSides = (OrderBookLevels, OrderBookLevels);

/// Парсер ответов OKX API
pub struct OkxParser;

impl OkxParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Проверить код ответа и извлечь data
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check code field
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("0");

        if code != "0" {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code.parse().unwrap_or(-1),
                message: format!("OKX error {}: {}", code, msg),
            });
        }

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Извлечь первый элемент из data array
    pub fn extract_first_data(response: &Value) -> ExchangeResult<&Value> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        arr.first()
            .ok_or_else(|| ExchangeError::Parse("'data' array is empty".to_string()))
    }

    /// Парсить f64 из string или number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Парсить f64 из поля
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Парсить обязательный f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Парсить строку из поля
    pub fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Парсить обязательную строку
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Парсить i64 из string или number
    pub fn parse_i64(value: &Value) -> Option<i64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_i64())
    }

    /// Парсить i64 из поля
    pub fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(Self::parse_i64)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить ticker
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let data = Self::extract_first_data(response)?;

        Ok(Ticker {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "last").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "bidPx"),
            ask_price: Self::get_f64(data, "askPx"),
            high_24h: Self::get_f64(data, "high24h"),
            low_24h: Self::get_f64(data, "low24h"),
            volume_24h: Self::get_f64(data, "vol24h"),
            quote_volume_24h: Self::get_f64(data, "volCcy24h"),
            price_change_24h: None, // OKX doesn't provide this directly
            price_change_percent_24h: None, // Would need to calculate from open24h
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
        })
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_first_data(response)?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            // OKX format: [price, size, deprecated, amount]
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None, // OKX doesn't provide sequence in this endpoint
        })
    }

    /// Парсить klines
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 9 {
                continue;
            }

            // OKX format: [timestamp, open, high, low, close, vol, volCcy, volCcyQuote, confirm]
            let open_time = Self::parse_i64(&candle[0]).unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
                quote_volume: Self::parse_f64(&candle[6]),
                close_time: None,
                trades: None,
            });
        }

        // OKX returns newest first, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    /// Парсить symbols/instruments
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut symbols = Vec::with_capacity(arr.len());

        for item in arr {
            let symbol = Self::get_str(item, "instId").unwrap_or("").to_string();
            let base_asset = Self::get_str(item, "baseCcy").unwrap_or("").to_string();
            let quote_asset = Self::get_str(item, "quoteCcy").unwrap_or("").to_string();

            let min_quantity = Self::get_f64(item, "minSz");
            let max_quantity = Self::get_f64(item, "maxLmtSz");
            let tick_size = Self::get_f64(item, "tickSz");
            let step_size = Self::get_f64(item, "lotSz");
            let min_notional = None; // OKX doesn't provide this directly

            let status = Self::get_str(item, "state").unwrap_or("").to_string();
            let price_precision = 8; // Default
            let quantity_precision = 8; // Default

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status,
                price_precision,
                quantity_precision,
                min_quantity,
                max_quantity,
                tick_size,
                step_size,
                min_notional,
            });
        }

        Ok(symbols)
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let data = Self::extract_first_data(response)?;

        Ok(FundingRate {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "fundingRate")?,
            next_funding_time: Self::get_i64(data, "nextFundingTime"),
            timestamp: Self::get_i64(data, "fundingTime").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order response (place/cancel)
    pub fn parse_order_response(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_first_data(response)?;

        // Check sCode for individual order status
        let s_code = Self::get_str(data, "sCode").unwrap_or("0");
        if s_code != "0" {
            let s_msg = Self::get_str(data, "sMsg").unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: s_code.parse().unwrap_or(-1),
                message: format!("Order error {}: {}", s_code, s_msg),
            });
        }

        let order_id = Self::get_str(data, "ordId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ordId'".to_string()))?
            .to_string();

        Ok(order_id)
    }

    /// Парсить order details
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        let data = Self::extract_first_data(response)?;
        Self::parse_order_data(data)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "ordType").unwrap_or("limit").to_lowercase().as_str() {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "ordId").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "clOrdId").map(String::from),
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "px"),
            stop_price: Self::get_f64(data, "slTriggerPx"),
            quantity: Self::get_f64(data, "sz").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "accFillSz").unwrap_or(0.0),
            average_price: Self::get_f64(data, "avgPx"),
            commission: None, // Would need to get from fills
            commission_asset: None,
            created_at: Self::get_i64(data, "cTime").unwrap_or(0),
            updated_at: Self::get_i64(data, "uTime"),
            time_in_force: TimeInForce::Gtc, // Default
        })
    }

    /// Парсить список orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let orders = arr.iter()
            .filter_map(|item| Self::parse_order_data(item).ok())
            .collect::<Vec<_>>();

        Ok(orders)
    }

    /// Парсить order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "state").unwrap_or("live") {
            "live" => OrderStatus::Open,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "canceled" => OrderStatus::Canceled,
            _ => OrderStatus::Open,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balance
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_first_data(response)?;

        let details = data.get("details")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'details' array".to_string()))?;

        let mut balances = Vec::with_capacity(details.len());

        for item in details {
            let asset = Self::get_str(item, "ccy").unwrap_or("").to_string();
            let free = Self::get_f64(item, "availBal").unwrap_or(0.0);
            let locked = Self::get_f64(item, "frozenBal").unwrap_or(0.0);
            let total = Self::get_f64(item, "eq").unwrap_or(free + locked);

            balances.push(Balance {
                asset,
                free,
                locked,
                total,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut positions = Vec::with_capacity(arr.len());

        for item in arr {
            let pos_side_str = Self::get_str(item, "posSide").unwrap_or("net");
            let pos_qty = Self::get_f64(item, "pos").unwrap_or(0.0);

            // Determine position side
            let side = match pos_side_str {
                "long" => PositionSide::Long,
                "short" => PositionSide::Short,
                "net" => {
                    if pos_qty > 0.0 {
                        PositionSide::Long
                    } else if pos_qty < 0.0 {
                        PositionSide::Short
                    } else {
                        continue; // Skip zero positions
                    }
                }
                _ => continue,
            };

            let quantity = pos_qty.abs();
            if quantity == 0.0 {
                continue; // Skip zero positions
            }

            positions.push(Position {
                symbol: Self::get_str(item, "instId").unwrap_or("").to_string(),
                side,
                quantity,
                entry_price: Self::get_f64(item, "avgPx").unwrap_or(0.0),
                mark_price: Self::get_f64(item, "markPx"),
                liquidation_price: Self::get_f64(item, "liqPx"),
                unrealized_pnl: Self::get_f64(item, "upl").unwrap_or(0.0),
                realized_pnl: None, // OKX doesn't provide realized PnL in position endpoint
                leverage: Self::get_f64(item, "lever").map(|l| l as u32).unwrap_or(1),
                margin: Self::get_f64(item, "margin"),
                margin_type: match Self::get_str(item, "mgnMode") {
                    Some("isolated") => MarginType::Isolated,
                    _ => MarginType::Cross,
                },
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить WebSocket ticker update
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "last").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "bidPx"),
            ask_price: Self::get_f64(data, "askPx"),
            high_24h: Self::get_f64(data, "high24h"),
            low_24h: Self::get_f64(data, "low24h"),
            volume_24h: Self::get_f64(data, "vol24h"),
            quote_volume_24h: Self::get_f64(data, "volCcy24h"),
            price_change_24h: {
                let last = Self::get_f64(data, "last");
                let open24h = Self::get_f64(data, "open24h");
                match (last, open24h) {
                    (Some(l), Some(o)) => Some(l - o),
                    _ => None,
                }
            },
            price_change_percent_24h: {
                let last = Self::get_f64(data, "last");
                let open24h = Self::get_f64(data, "open24h");
                match (last, open24h) {
                    (Some(l), Some(o)) if o != 0.0 => Some(((l - o) / o) * 100.0),
                    _ => None,
                }
            },
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
        })
    }

    /// Парсить WebSocket orderbook update
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<OrderBookSides> {
        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok((parse_levels("asks"), parse_levels("bids")))
    }

    /// Парсить WebSocket trade
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            id: Self::get_str(data, "tradeId").unwrap_or("").to_string(),
            price: Self::require_f64(data, "px")?,
            quantity: Self::require_f64(data, "sz")?,
            side,
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
        })
    }

    /// Парсить WebSocket kline
    pub fn parse_ws_kline(data: &Value) -> ExchangeResult<Kline> {
        let candle = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Kline data is not an array".to_string()))?;

        if candle.len() < 9 {
            return Err(ExchangeError::Parse("Incomplete kline data".to_string()));
        }

        Ok(Kline {
            open_time: Self::parse_i64(&candle[0]).unwrap_or(0),
            open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
            high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
            low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
            close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
            volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
            quote_volume: Self::parse_f64(&candle[6]),
            close_time: None,
            trades: None,
        })
    }

    /// Парсить WebSocket order update
    ///
    /// OKX WS orders channel data item:
    /// `{"instId":"BTC-USDT","ordId":"123","clOrdId":"","px":"67000","sz":"0.1",
    ///   "side":"buy","ordType":"limit","state":"filled","accFillSz":"0.1",
    ///   "avgPx":"67000","fillPx":"67000","fillSz":"0.1","fillFee":"-0.1",
    ///   "fillFeeCcy":"USDT","tradeId":"456","cTime":"1234","uTime":"1234"}`
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let order_id = Self::get_str(data, "ordId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ordId'".to_string()))?
            .to_string();

        let symbol = Self::get_str(data, "instId").unwrap_or("").to_string();

        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "ordType").unwrap_or("limit").to_lowercase().as_str() {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: Self::get_f64(data, "px").unwrap_or(0.0) },
        };

        let status = Self::parse_order_status(data);

        let quantity = Self::get_f64(data, "sz").unwrap_or(0.0);
        let filled_quantity = Self::get_f64(data, "accFillSz").unwrap_or(0.0);
        let timestamp = Self::get_i64(data, "uTime")
            .or_else(|| Self::get_i64(data, "cTime"))
            .unwrap_or(0);

        Ok(OrderUpdateEvent {
            order_id,
            client_order_id: Self::get_str(data, "clOrdId")
                .filter(|s| !s.is_empty())
                .map(String::from),
            symbol,
            side,
            order_type,
            status,
            price: Self::get_f64(data, "px"),
            quantity,
            filled_quantity,
            average_price: Self::get_f64(data, "avgPx"),
            last_fill_price: Self::get_f64(data, "fillPx"),
            last_fill_quantity: Self::get_f64(data, "fillSz"),
            last_fill_commission: Self::get_f64(data, "fillFee"),
            commission_asset: Self::get_str(data, "fillFeeCcy")
                .filter(|s| !s.is_empty())
                .map(String::from),
            trade_id: Self::get_str(data, "tradeId")
                .filter(|s| !s.is_empty())
                .map(String::from),
            timestamp,
        })
    }

    /// Парсить WebSocket balance update
    ///
    /// OKX WS account channel data item:
    /// `{"totalEq":"10000","details":[{"ccy":"USDT","availBal":"5000","frozenBal":"5000","eq":"10000"}]}`
    /// Returns one `BalanceUpdateEvent` per currency in `details`.
    /// Callers that need multi-currency should loop over the raw data array themselves;
    /// this helper returns the first non-empty entry.
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<BalanceUpdateEvent> {
        // Try to get the first currency detail from the `details` array.
        let detail = data.get("details")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("Missing 'details' in balance update".to_string()))?;

        let asset = Self::get_str(detail, "ccy")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ccy' in balance detail".to_string()))?
            .to_string();

        let free = Self::get_f64(detail, "availBal").unwrap_or(0.0);
        let locked = Self::get_f64(detail, "frozenBal").unwrap_or(0.0);
        let total = Self::get_f64(detail, "eq").unwrap_or(free + locked);

        let timestamp = Self::get_i64(data, "uTime").unwrap_or(0);

        Ok(BalanceUpdateEvent {
            asset,
            free,
            locked,
            total,
            delta: None,
            reason: None,
            timestamp,
        })
    }

    /// Парсить WebSocket position update
    ///
    /// OKX WS positions channel data item:
    /// `{"instId":"BTC-USDT","posSide":"long","pos":"0.1","avgPx":"67000",
    ///   "upl":"100","liqPx":"60000","markPx":"67100","lever":"10","mgnMode":"cross","uTime":"1234"}`
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<PositionUpdateEvent> {
        let symbol = Self::get_str(data, "instId").unwrap_or("").to_string();

        let pos_qty = Self::get_f64(data, "pos").unwrap_or(0.0);
        let pos_side_str = Self::get_str(data, "posSide").unwrap_or("net");
        let side = match pos_side_str {
            "long" => PositionSide::Long,
            "short" => PositionSide::Short,
            _ => {
                if pos_qty >= 0.0 { PositionSide::Long } else { PositionSide::Short }
            }
        };

        let margin_type = match Self::get_str(data, "mgnMode") {
            Some("isolated") => Some(MarginType::Isolated),
            _ => Some(MarginType::Cross),
        };

        let timestamp = Self::get_i64(data, "uTime").unwrap_or(0);

        Ok(PositionUpdateEvent {
            symbol,
            side,
            quantity: pos_qty.abs(),
            entry_price: Self::get_f64(data, "avgPx").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "markPx"),
            unrealized_pnl: Self::get_f64(data, "upl").unwrap_or(0.0),
            realized_pnl: None,
            liquidation_price: Self::get_f64(data, "liqPx"),
            leverage: Self::get_f64(data, "lever").map(|l| l as u32),
            margin_type,
            reason: None,
            timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ALGO ORDER PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse algo order placement response.
    ///
    /// OKX algo order response: `data[0] = { algoId, clAlgoId, sCode, sMsg }`
    /// On success `sCode == "0"`.
    ///
    /// Used for: conditional (stop), move_order_stop (trailing), oco, twap, iceberg.
    pub fn parse_algo_order_response(response: &Value) -> ExchangeResult<AlgoOrderResponse> {
        let data = Self::extract_first_data(response)?;

        // Check per-order status code
        let s_code = Self::get_str(data, "sCode").unwrap_or("0");
        if s_code != "0" {
            let s_msg = Self::get_str(data, "sMsg").unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: s_code.parse().unwrap_or(-1),
                message: format!("Algo order error {}: {}", s_code, s_msg),
            });
        }

        let algo_id = Self::get_str(data, "algoId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'algoId' in algo order response".to_string()))?
            .to_string();

        Ok(AlgoOrderResponse {
            algo_id,
            status: "live".to_string(),
            executed_count: None,
            total_count: None,
        })
    }

    /// Parse algo order cancel response.
    ///
    /// OKX cancel-algos response: `data[0] = { algoId, clAlgoId, sCode, sMsg }`
    pub fn parse_algo_cancel_response(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_first_data(response)?;

        let s_code = Self::get_str(data, "sCode").unwrap_or("0");
        if s_code != "0" {
            let s_msg = Self::get_str(data, "sMsg").unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: s_code.parse().unwrap_or(-1),
                message: format!("Algo cancel error {}: {}", s_code, s_msg),
            });
        }

        Ok(Self::get_str(data, "algoId").unwrap_or("").to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OPTIONAL TRAIT PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse cancel-all-after response.
    ///
    /// OKX `cancel-all-after` returns: `data[0] = { triggerTime, ts }`
    /// When `timeOut = "0"` is sent, it cancels immediately and disables the DMS timer.
    /// The response does not list cancelled orders individually.
    pub fn parse_cancel_all_response(response: &Value) -> ExchangeResult<CancelAllResponse> {
        // Just verify the response code is success ("0")
        Self::extract_data(response)?;

        Ok(CancelAllResponse {
            cancelled_count: 0, // OKX does not return individual cancelled order count
            failed_count: 0,
            details: vec![],
        })
    }

    /// Parse amend order response.
    ///
    /// OKX returns: `data[0] = { ordId, clOrdId, sCode, sMsg }`
    /// On success `sCode == "0"`.
    pub fn parse_amend_order_response(response: &Value) -> ExchangeResult<Order> {
        let data = Self::extract_first_data(response)?;

        let s_code = Self::get_str(data, "sCode").unwrap_or("0");
        if s_code != "0" {
            let s_msg = Self::get_str(data, "sMsg").unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: s_code.parse().unwrap_or(-1),
                message: format!("Amend order error {}: {}", s_code, s_msg),
            });
        }

        let id = Self::get_str(data, "ordId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ordId' in amend response".to_string()))?
            .to_string();

        // OKX amend returns minimal info — build a placeholder Order.
        Ok(Order {
            id,
            client_order_id: Self::get_str(data, "clOrdId").map(String::from),
            symbol: String::new(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit { price: 0.0 },
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
            time_in_force: TimeInForce::Gtc,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT TRANSFERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse asset transfer response.
    ///
    /// OKX: `data[0] = { transId, ccy, from, amt, to }`
    pub fn parse_transfer_response(response: &Value) -> ExchangeResult<TransferResponse> {
        let data = Self::extract_first_data(response)?;

        let transfer_id = Self::get_str(data, "transId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'transId' in transfer response".to_string()))?
            .to_string();

        let asset = Self::get_str(data, "ccy")
            .unwrap_or("")
            .to_string();

        let amount = Self::get_f64(data, "amt").unwrap_or(0.0);

        Ok(TransferResponse {
            transfer_id,
            status: "successful".to_string(),
            asset,
            amount,
            timestamp: None,
        })
    }

    /// Parse transfer history from asset/bills or transfer-state response.
    ///
    /// OKX bills: `data = [{ billId, ccy, sz, ts, ... }, ...]`
    /// OKX transfer-state: `data = [{ transId, ccy, from, amt, to, state, ts }, ...]`
    pub fn parse_transfer_history(response: &Value) -> ExchangeResult<Vec<TransferResponse>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let records = arr.iter().map(|item| {
            // transfer-state uses transId, bills use billId
            let transfer_id = Self::get_str(item, "transId")
                .or_else(|| Self::get_str(item, "billId"))
                .unwrap_or("")
                .to_string();

            let asset = Self::get_str(item, "ccy")
                .unwrap_or("")
                .to_string();

            // bills use "sz", transfer-state uses "amt"
            let amount = Self::get_f64(item, "amt")
                .or_else(|| Self::get_f64(item, "sz"))
                .unwrap_or(0.0)
                .abs();

            let status = Self::get_str(item, "state")
                .unwrap_or("successful")
                .to_string();

            let timestamp = Self::get_i64(item, "ts");

            TransferResponse {
                transfer_id,
                status,
                asset,
                amount,
                timestamp,
            }
        }).collect();

        Ok(records)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTODIAL FUNDS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse deposit address response.
    ///
    /// OKX: `data[0] = { addr, tag, chain, ccy, ... }`
    pub fn parse_deposit_address(response: &Value) -> ExchangeResult<DepositAddress> {
        let data = Self::extract_first_data(response)?;

        let address = Self::get_str(data, "addr")
            .ok_or_else(|| ExchangeError::Parse("Missing 'addr' in deposit address response".to_string()))?
            .to_string();

        let asset = Self::get_str(data, "ccy")
            .unwrap_or("")
            .to_string();

        let tag = Self::get_str(data, "tag")
            .filter(|s| !s.is_empty())
            .map(String::from);

        let network = Self::get_str(data, "chain")
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(DepositAddress {
            address,
            tag,
            network,
            asset,
            created_at: Self::get_i64(data, "ts"),
        })
    }

    /// Parse withdrawal response.
    ///
    /// OKX: `data[0] = { wdId, ccy, ... }`
    pub fn parse_withdrawal_response(response: &Value) -> ExchangeResult<WithdrawResponse> {
        let data = Self::extract_first_data(response)?;

        let withdraw_id = Self::get_str(data, "wdId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'wdId' in withdrawal response".to_string()))?
            .to_string();

        Ok(WithdrawResponse {
            withdraw_id,
            status: "pending".to_string(),
            tx_hash: None,
        })
    }

    /// Parse deposit history.
    ///
    /// OKX: `data = [{ depId, ccy, amt, txId, chain, state, ts }, ...]`
    pub fn parse_deposit_history(response: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let records = arr.iter().map(|item| {
            let id = Self::get_str(item, "depId")
                .unwrap_or("")
                .to_string();
            let asset = Self::get_str(item, "ccy")
                .unwrap_or("")
                .to_string();
            let amount = Self::get_f64(item, "amt").unwrap_or(0.0);
            let tx_hash = Self::get_str(item, "txId")
                .filter(|s| !s.is_empty())
                .map(String::from);
            let network = Self::get_str(item, "chain")
                .filter(|s| !s.is_empty())
                .map(String::from);
            // OKX deposit states: 0=waiting, 1=credited, 2=successful, 8=pending review
            let status = match Self::get_str(item, "state").unwrap_or("0") {
                "2" => "Credited",
                "1" => "Credited",
                "0" => "Pending",
                _ => "Pending",
            }.to_string();
            let timestamp = Self::get_i64(item, "ts").unwrap_or(0);

            FundsRecord::Deposit {
                id,
                asset,
                amount,
                tx_hash,
                network,
                status,
                timestamp,
            }
        }).collect();

        Ok(records)
    }

    /// Parse withdrawal history.
    ///
    /// OKX: `data = [{ wdId, ccy, amt, fee, to, tag, txId, chain, state, ts }, ...]`
    pub fn parse_withdrawal_history(response: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let records = arr.iter().map(|item| {
            let id = Self::get_str(item, "wdId")
                .unwrap_or("")
                .to_string();
            let asset = Self::get_str(item, "ccy")
                .unwrap_or("")
                .to_string();
            let amount = Self::get_f64(item, "amt").unwrap_or(0.0);
            let fee = Self::get_f64(item, "fee");
            let address = Self::get_str(item, "to")
                .unwrap_or("")
                .to_string();
            let tag = Self::get_str(item, "tag")
                .filter(|s| !s.is_empty())
                .map(String::from);
            let tx_hash = Self::get_str(item, "txId")
                .filter(|s| !s.is_empty())
                .map(String::from);
            let network = Self::get_str(item, "chain")
                .filter(|s| !s.is_empty())
                .map(String::from);
            // OKX withdrawal states: -3/-2/-1=cancel, 0=email, 1=pending, 2=sent, 3=crypto, 4/5=pending, 7=approved, 10=waiting, 4=success
            let status = match Self::get_str(item, "state").unwrap_or("0") {
                "2" | "3" => "Completed",
                "-3" | "-2" | "-1" => "Failed",
                _ => "Pending",
            }.to_string();
            let timestamp = Self::get_i64(item, "ts").unwrap_or(0);

            FundsRecord::Withdrawal {
                id,
                asset,
                amount,
                fee,
                address,
                tag,
                tx_hash,
                network,
                status,
                timestamp,
            }
        }).collect();

        Ok(records)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUB-ACCOUNTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse sub-account create response.
    ///
    /// OKX: `data[0] = { subAcct, label, ... }`
    pub fn parse_sub_account_create(response: &Value) -> ExchangeResult<SubAccountResult> {
        let data = Self::extract_first_data(response)?;

        let name = Self::get_str(data, "subAcct")
            .unwrap_or("")
            .to_string();

        Ok(SubAccountResult {
            id: Some(name.clone()),
            name: Some(name),
            accounts: vec![],
            transaction_id: None,
        })
    }

    /// Parse sub-account list response.
    ///
    /// OKX: `data = [{ subAcct, label, uid, enable, ts }, ...]`
    pub fn parse_sub_account_list(response: &Value) -> ExchangeResult<SubAccountResult> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let accounts = arr.iter().map(|item| {
            let name = Self::get_str(item, "subAcct")
                .unwrap_or("")
                .to_string();
            let id = Self::get_str(item, "uid")
                .unwrap_or(&name)
                .to_string();
            let status = if Self::get_str(item, "enable").unwrap_or("true") == "true" {
                "Normal".to_string()
            } else {
                "Frozen".to_string()
            };

            SubAccount { id, name, status }
        }).collect();

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts,
            transaction_id: None,
        })
    }

    /// Parse sub-account transfer response.
    ///
    /// OKX: `data[0] = { transId }`
    pub fn parse_sub_account_transfer(response: &Value) -> ExchangeResult<SubAccountResult> {
        let data = Self::extract_first_data(response)?;

        let transaction_id = Self::get_str(data, "transId")
            .map(String::from);

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts: vec![],
            transaction_id,
        })
    }

    /// Parse sub-account balance response.
    ///
    /// OKX: `data[0] = { acctEq, details: [...] }` (same format as regular balance)
    pub fn parse_sub_account_balance(response: &Value) -> ExchangeResult<SubAccountResult> {
        // Verify response succeeds; balance details not mapped into SubAccountResult
        Self::extract_data(response)?;

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts: vec![],
            transaction_id: None,
        })
    }

    /// Parse batch orders response (place or cancel).
    ///
    /// OKX batch response: `data = [{ ordId, clOrdId, sCode, sMsg }, ...]`
    /// Each element has `sCode == "0"` for success.
    pub fn parse_batch_orders_response(response: &Value) -> ExchangeResult<Vec<OrderResult>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array in batch response".to_string()))?;

        let results = arr.iter()
            .map(|item| {
                let s_code = Self::get_str(item, "sCode").unwrap_or("0");
                let success = s_code == "0";
                if success {
                    OrderResult {
                        order: None,
                        client_order_id: Self::get_str(item, "clOrdId").map(String::from),
                        success: true,
                        error: None,
                        error_code: None,
                    }
                } else {
                    let s_msg = Self::get_str(item, "sMsg").unwrap_or("Unknown error").to_string();
                    OrderResult {
                        order: None,
                        client_order_id: Self::get_str(item, "clOrdId").map(String::from),
                        success: false,
                        error: Some(s_msg),
                        error_code: s_code.parse().ok(),
                    }
                }
            })
            .collect();

        Ok(results)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FILLS / USER TRADES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse a single fill record from the `data` array of `/api/v5/trade/fills`
    /// or `/api/v5/trade/fills-history`.
    ///
    /// OKX fill fields:
    /// - `tradeId`  — fill ID
    /// - `ordId`    — parent order ID
    /// - `instId`   — instrument (e.g. "BTC-USDT")
    /// - `side`     — "buy" | "sell"
    /// - `fillPx`   — fill price (string)
    /// - `fillSz`   — fill quantity (string)
    /// - `fee`      — fee amount; negative means cost (string)
    /// - `feeCcy`   — fee currency
    /// - `execType` — "M" (maker) | "T" (taker)
    /// - `ts`       — Unix timestamp in ms (string)
    fn parse_fill_data(data: &Value) -> ExchangeResult<UserTrade> {
        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let price = Self::get_f64(data, "fillPx")
            .ok_or_else(|| ExchangeError::Parse("Missing 'fillPx' in fill".to_string()))?;

        let quantity = Self::get_f64(data, "fillSz")
            .ok_or_else(|| ExchangeError::Parse("Missing 'fillSz' in fill".to_string()))?;

        // `fee` is negative (a cost). Store the absolute value.
        let commission = Self::get_f64(data, "fee")
            .unwrap_or(0.0)
            .abs();

        let commission_asset = Self::get_str(data, "feeCcy")
            .unwrap_or("")
            .to_string();

        // "M" = maker, anything else = taker
        let is_maker = Self::get_str(data, "execType")
            .map(|s| s.eq_ignore_ascii_case("M"))
            .unwrap_or(false);

        let timestamp = data.get("ts")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        Ok(UserTrade {
            id: Self::get_str(data, "tradeId").unwrap_or("").to_string(),
            order_id: Self::get_str(data, "ordId").unwrap_or("").to_string(),
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            side,
            price,
            quantity,
            commission,
            commission_asset,
            is_maker,
            timestamp,
        })
    }

    /// Parse the full response from `/api/v5/trade/fills` or
    /// `/api/v5/trade/fills-history` into a `Vec<UserTrade>`.
    pub fn parse_fills(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array in fills response".to_string()))?;

        let trades = arr.iter()
            .filter_map(|item| Self::parse_fill_data(item).ok())
            .collect::<Vec<_>>();

        Ok(trades)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding payments from `GET /api/v5/account/bills?type=8` (subType 173/174).
    ///
    /// subType 173 = funding fee expense, 174 = funding fee income.
    pub fn parse_funding_payments(response: &Value) -> ExchangeResult<Vec<FundingPayment>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in account/bills data".to_string()))?;

        let mut payments = Vec::with_capacity(arr.len());
        for item in arr {
            let symbol = item.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let payment: f64 = item.get("balChg")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let position_size: f64 = item.get("sz")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let asset = item.get("ccy").and_then(|v| v.as_str()).unwrap_or("USDT").to_string();
            let timestamp: i64 = item.get("ts")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
            payments.push(FundingPayment {
                symbol,
                funding_rate: 0.0,
                position_size,
                payment,
                asset,
                timestamp,
            });
        }
        Ok(payments)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT LEDGER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ledger entries from `GET /api/v5/account/bills` (all types).
    ///
    /// OKX `type` values: 1=transfer, 2=trade, 3=delivery, 5=forced-liquidation,
    /// 7=funding-fee, 8=interest, 9=rebate, 13=borrowing-fee, 20=deposit, 21=withdrawal.
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in account/bills data".to_string()))?;

        let mut entries = Vec::with_capacity(arr.len());
        for item in arr {
            let id = item.get("billId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let inst_id = item.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let bill_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("0");
            let sub_type = item.get("subType").and_then(|v| v.as_str()).unwrap_or("0");
            let amount: f64 = item.get("balChg")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let balance: Option<f64> = item.get("bal")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
            let asset = item.get("ccy").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let timestamp: i64 = item.get("ts")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
            let entry_type = match bill_type {
                "1" => LedgerEntryType::Transfer,
                "2" => LedgerEntryType::Trade,
                "3" => LedgerEntryType::Settlement,
                "5" | "14" => LedgerEntryType::Liquidation,
                "7" | "170" | "173" | "174" => LedgerEntryType::Funding,
                "9" | "12" => LedgerEntryType::Rebate,
                "8" | "13" => LedgerEntryType::Fee,
                "20" => LedgerEntryType::Deposit,
                "21" => LedgerEntryType::Withdrawal,
                _ => LedgerEntryType::Other(format!("type={} subType={}", bill_type, sub_type)),
            };
            let ref_id = item.get("ordId")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty() && *s != "0")
                .map(|s| s.to_string());
            entries.push(LedgerEntry {
                id,
                asset,
                amount,
                balance,
                entry_type,
                description: format!("type={} subType={} {}", bill_type, sub_type, inst_id),
                ref_id,
                timestamp,
            });
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "code": "0",
            "msg": "",
            "data": [{
                "instId": "BTC-USDT",
                "last": "43250.5",
                "bidPx": "43250.0",
                "askPx": "43251.0",
                "high24h": "43500.0",
                "low24h": "42500.0",
                "vol24h": "1850.25",
                "volCcy24h": "79852341.25",
                "ts": "1672841403093"
            }]
        });

        let ticker = OkxParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert_eq!(ticker.last_price, 43250.5);
    }

    #[test]
    fn test_parse_error_response() {
        let response = json!({
            "code": "50111",
            "msg": "Invalid sign",
            "data": []
        });

        let result = OkxParser::extract_data(&response);
        assert!(result.is_err());
        if let Err(ExchangeError::Api { code: _, message }) = result {
            assert!(message.contains("50111"));
            assert!(message.contains("Invalid sign"));
        }
    }
}
