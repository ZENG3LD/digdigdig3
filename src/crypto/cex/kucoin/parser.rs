//! # KuCoin Response Parser
//!
//! Парсинг JSON ответов от KuCoin API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, TimeInForce,
    FundingRate, PublicTrade, StreamEvent, TradeSide,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    BalanceChangeReason, PositionChangeReason,
    CancelAllResponse, OrderResult,
    UserTrade,
};

/// Парсер ответов KuCoin API
pub struct KuCoinParser;

impl KuCoinParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Извлечь data из response
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
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
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Парсить обязательную строку
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить price
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;
        Self::require_f64(data, "price")
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

            if candle.len() < 7 {
                continue;
            }

            // KuCoin format: [time, open, close, high, low, volume, turnover]
            let open_time = candle[0].as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0) * 1000; // seconds to ms

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                close: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                high: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                low: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
                quote_volume: Self::parse_f64(&candle[6]),
                close_time: None,
                trades: None,
            });
        }

        // KuCoin returns newest first, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_data(response)?;

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

        Ok(OrderBook {
            // Timestamp field differs: Spot uses "time" (ms), Futures uses "ts" (ns)
            timestamp: data.get("ts")
                .or_else(|| data.get("time"))
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            // Sequence is returned as integer by KuCoin, but handle both formats
            sequence: data.get("sequence")
                .and_then(|s| {
                    // Handle both integer and string formats
                    s.as_i64().map(|n| n.to_string())
                        .or_else(|| s.as_str().map(String::from))
                }),
        })
    }

    /// Парсить ticker (supports both Spot and Futures formats)
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;

        // Spot format uses: last, buy, sell, time
        // Futures format uses: price, bestBidPrice, bestAskPrice, ts
        let last_price = Self::get_f64(data, "last")
            .or_else(|| Self::get_f64(data, "price"))
            .unwrap_or(0.0);

        let bid_price = Self::get_f64(data, "buy")
            .or_else(|| Self::get_f64(data, "bestBidPrice"));

        let ask_price = Self::get_f64(data, "sell")
            .or_else(|| Self::get_f64(data, "bestAskPrice"));

        let timestamp = data.get("time")
            .or_else(|| data.get("ts"))
            .and_then(|t| t.as_i64())
            .unwrap_or(0);

        Ok(Ticker {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: Self::get_f64(data, "high"),
            low_24h: Self::get_f64(data, "low"),
            volume_24h: Self::get_f64(data, "vol"),
            quote_volume_24h: Self::get_f64(data, "volValue"),
            price_change_24h: Self::get_f64(data, "changePrice"),
            price_change_percent_24h: Self::get_f64(data, "changeRate").map(|r| r * 100.0),
            timestamp,
        })
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let data = Self::extract_data(response)?;

        Ok(FundingRate {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "value")?,
            next_funding_time: None,
            timestamp: data.get("timePoint")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info (symbol list) from KuCoin response
    ///
    /// Spot: data = [{ symbol, baseCurrency, quoteCurrency, enableTrading, baseMinSize, baseMaxSize, baseIncrement, priceIncrement }]
    /// Futures: data = [{ symbol, baseCurrency, quoteCurrency, status, lotSize, tickSize }]
    ///
    /// Filters to active/trading symbols only.
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let symbols = arr.iter()
            .filter_map(|item| {
                let symbol = Self::get_str(item, "symbol")?.to_string();
                // Spot uses baseCurrency/quoteCurrency; futures use baseCurrency/quoteCurrency too
                let base_asset = Self::get_str(item, "baseCurrency")
                    .unwrap_or("")
                    .to_string();
                let quote_asset = Self::get_str(item, "quoteCurrency")
                    .unwrap_or("")
                    .to_string();

                // Spot: enableTrading bool; Futures: status string ("Open", etc.)
                let enable_trading = item.get("enableTrading").and_then(|v| v.as_bool());
                let status_str = Self::get_str(item, "status");

                // Filter inactive symbols
                if let Some(enabled) = enable_trading {
                    if !enabled {
                        return None;
                    }
                }
                if let Some(s) = status_str {
                    if s != "Open" && s != "BeingReplenished" {
                        return None;
                    }
                }

                let status = "TRADING".to_string();

                // Quantity precision/step from baseIncrement (spot) or lotSize (futures)
                let step_size = Self::get_f64(item, "baseIncrement")
                    .or_else(|| Self::get_f64(item, "lotSize"));

                let quantity_precision = step_size
                    .map(|t| {
                        let s = format!("{:.10}", t);
                        let trimmed = s.trim_end_matches('0');
                        if let Some(dot_pos) = trimmed.find('.') {
                            (trimmed.len() - dot_pos - 1) as u8
                        } else {
                            0u8
                        }
                    })
                    .unwrap_or(8);

                // Price precision from priceIncrement (spot) or tickSize (futures)
                let tick_size = Self::get_f64(item, "priceIncrement")
                    .or_else(|| Self::get_f64(item, "tickSize"));

                let price_precision = tick_size
                    .map(|t| {
                        let s = format!("{:.10}", t);
                        let trimmed = s.trim_end_matches('0');
                        if let Some(dot_pos) = trimmed.find('.') {
                            (trimmed.len() - dot_pos - 1) as u8
                        } else {
                            0u8
                        }
                    })
                    .unwrap_or(8);

                let min_quantity = Self::get_f64(item, "baseMinSize")
                    .or_else(|| Self::get_f64(item, "lotSize"));
                let max_quantity = Self::get_f64(item, "baseMaxSize");
                let min_notional = Self::get_f64(item, "quoteMinSize");

                Some(crate::core::types::SymbolInfo {
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
                })
            })
            .collect();

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        let data = Self::extract_data(response)?;
        Self::parse_order_data(data, symbol)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value, symbol: &str) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("limit").to_lowercase().as_str() {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "id")
                .or_else(|| Self::get_str(data, "orderId"))
                .unwrap_or("")
                .to_string(),
            client_order_id: Self::get_str(data, "clientOid").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "stopPrice"),
            quantity: Self::get_f64(data, "size").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "dealSize").unwrap_or(0.0),
            average_price: Self::get_f64(data, "dealFunds")
                .and_then(|funds| {
                    Self::get_f64(data, "dealSize")
                        .filter(|&size| size > 0.0)
                        .map(|size| funds / size)
                }),
            commission: None,
            commission_asset: None,
            created_at: data.get("createdAt").and_then(|t| t.as_i64()).unwrap_or(0),
            updated_at: data.get("updatedAt").and_then(|t| t.as_i64()),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить статус ордера
    fn parse_order_status(data: &Value) -> OrderStatus {
        let is_active = data.get("isActive").and_then(|v| v.as_bool()).unwrap_or(true);
        let cancel_exist = data.get("cancelExist").and_then(|v| v.as_bool()).unwrap_or(false);
        let deal_size = Self::get_f64(data, "dealSize").unwrap_or(0.0);
        let size = Self::get_f64(data, "size").unwrap_or(1.0);

        if cancel_exist {
            if deal_size > 0.0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Canceled
            }
        } else if !is_active {
            if deal_size >= size {
                OrderStatus::Filled
            } else {
                OrderStatus::PartiallyFilled
            }
        } else if deal_size > 0.0 {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::New
        }
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;

        // KuCoin wraps orders in "items"
        let items = data.get("items")
            .and_then(|v| v.as_array())
            .or_else(|| data.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        items.iter()
            .map(|item| Self::parse_order_data(item, ""))
            .collect()
    }

    /// Парсить order ID из create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_data(response)?;
        Self::require_str(data, "orderId").map(String::from)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balances
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of accounts".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "currency").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "available").unwrap_or(0.0);
            let locked = Self::get_f64(item, "holds").unwrap_or(0.0);

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    /// Парсить futures account overview
    pub fn parse_futures_account(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let currency = Self::get_str(data, "currency").unwrap_or("USDT").to_string();
        let available = Self::get_f64(data, "availableBalance").unwrap_or(0.0);
        let frozen = Self::get_f64(data, "frozenFunds").unwrap_or(0.0);

        Ok(vec![Balance {
            asset: currency,
            free: available,
            locked: frozen,
            total: available + frozen,
        }])
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        let mut positions = Vec::new();

        for item in arr {
            if let Some(pos) = Self::parse_position_data(item) {
                positions.push(pos);
            }
        }

        Ok(positions)
    }

    /// Парсить single position
    pub fn parse_position(response: &Value) -> ExchangeResult<Position> {
        let data = Self::extract_data(response)?;
        Self::parse_position_data(data)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "symbol")?.to_string();
        let current_qty = Self::get_f64(data, "currentQty").unwrap_or(0.0);

        // Skip empty positions
        if current_qty.abs() < f64::EPSILON {
            return None;
        }

        let side = if current_qty > 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Some(Position {
            symbol,
            side,
            quantity: current_qty.abs(),
            entry_price: Self::get_f64(data, "avgEntryPrice").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "markPrice"),
            unrealized_pnl: Self::get_f64(data, "unrealisedPnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realisedPnl"),
            leverage: Self::get_f64(data, "realLeverage").map(|l| l as u32).unwrap_or(1),
            liquidation_price: Self::get_f64(data, "liquidationPrice"),
            margin: Self::get_f64(data, "maintMargin"),
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker message (`/market/ticker:{symbol}`, subject `trade.ticker`)
    ///
    /// This channel only provides best bid/ask/last — no 24h statistics.
    /// For full 24h stats, see `parse_ws_snapshot_ticker`.
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "price").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "bestBid")
                .or_else(|| Self::get_f64(data, "bestBidPrice")),
            ask_price: Self::get_f64(data, "bestAsk")
                .or_else(|| Self::get_f64(data, "bestAskPrice")),
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: data.get("time")
                .or_else(|| data.get("ts"))
                .and_then(|t| t.as_i64())
                .map(|t| if t > 1_000_000_000_000_000 { t / 1_000_000 } else { t })
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket snapshot ticker (`/market/snapshot:{symbol}`, subject `trade.snapshot`)
    ///
    /// This channel includes full 24h statistics: high, low, vol, volValue,
    /// changePrice, changeRate, as well as current bid/ask/last.
    pub fn parse_ws_snapshot_ticker(data: &Value) -> ExchangeResult<Ticker> {
        // Snapshot uses "lastTradedPrice" for last price, "buy"/"sell" for bid/ask
        let last_price = Self::get_f64(data, "lastTradedPrice")
            .or_else(|| Self::get_f64(data, "last"))
            .unwrap_or(0.0);

        let bid_price = Self::get_f64(data, "buy")
            .or_else(|| Self::get_f64(data, "bestBid"));
        let ask_price = Self::get_f64(data, "sell")
            .or_else(|| Self::get_f64(data, "bestAsk"));

        let timestamp = data.get("datetime")
            .and_then(|t| t.as_i64())
            .or_else(|| data.get("time").and_then(|t| t.as_i64()))
            .unwrap_or(0);

        Ok(Ticker {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: Self::get_f64(data, "high"),
            low_24h: Self::get_f64(data, "low"),
            volume_24h: Self::get_f64(data, "vol"),
            quote_volume_24h: Self::get_f64(data, "volValue"),
            price_change_24h: Self::get_f64(data, "changePrice"),
            price_change_percent_24h: Self::get_f64(data, "changeRate").map(|r| r * 100.0),
            timestamp,
        })
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(data, "tradeId").unwrap_or("").to_string(),
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            price: Self::require_f64(data, "price")?,
            quantity: Self::get_f64(data, "size").unwrap_or(0.0),
            side,
            timestamp: data.get("time")
                .or_else(|| data.get("ts"))
                .and_then(|t| t.as_i64())
                .map(|t| if t > 1_000_000_000_000_000 { t / 1_000_000 } else { t })
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket orderbook delta message
    pub fn parse_ws_orderbook_delta(data: &Value) -> ExchangeResult<StreamEvent> {
        let parse_changes = |key: &str| -> Vec<(f64, f64)> {
            data.get("changes")
                .and_then(|c| c.get(key))
                .and_then(|arr| arr.as_array())
                .map(|changes| {
                    changes.iter()
                        .filter_map(|change| {
                            let pair = change.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        // Futures format (change string)
        if let Some(change_str) = Self::get_str(data, "change") {
            let parts: Vec<&str> = change_str.split(',').collect();
            if parts.len() >= 3 {
                let price = parts[0].parse::<f64>().unwrap_or(0.0);
                let size = parts[2].parse::<f64>().unwrap_or(0.0);
                let side = parts[1];

                let (bids, asks) = if side == "buy" {
                    (vec![(price, size)], vec![])
                } else {
                    (vec![], vec![(price, size)])
                };

                return Ok(StreamEvent::OrderbookDelta {
                    bids,
                    asks,
                    timestamp: data.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
                });
            }
        }

        // Spot format (changes object)
        Ok(StreamEvent::OrderbookDelta {
            bids: parse_changes("bids"),
            asks: parse_changes("asks"),
            timestamp: data.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    /// Parse WebSocket kline message
    pub fn parse_ws_kline(data: &Value) -> ExchangeResult<Kline> {
        let candles = data.get("candles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candles array".to_string()))?;

        if candles.len() < 7 {
            return Err(ExchangeError::Parse("Invalid candles format".to_string()));
        }

        let open_time = Self::parse_f64(&candles[0])
            .map(|t| (t * 1000.0) as i64)
            .unwrap_or(0);

        Ok(Kline {
            open_time,
            open: Self::parse_f64(&candles[1]).unwrap_or(0.0),
            close: Self::parse_f64(&candles[2]).unwrap_or(0.0),
            high: Self::parse_f64(&candles[3]).unwrap_or(0.0),
            low: Self::parse_f64(&candles[4]).unwrap_or(0.0),
            volume: Self::parse_f64(&candles[5]).unwrap_or(0.0),
            quote_volume: Self::parse_f64(&candles[6]),
            close_time: None,
            trades: None,
        })
    }

    /// Parse WebSocket mark price message
    pub fn parse_ws_mark_price(data: &Value) -> ExchangeResult<StreamEvent> {
        Ok(StreamEvent::MarkPrice {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            mark_price: Self::require_f64(data, "markPrice")?,
            index_price: Self::get_f64(data, "indexPrice"),
            timestamp: data.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    /// Parse WebSocket funding rate message
    pub fn parse_ws_funding_rate(data: &Value) -> ExchangeResult<StreamEvent> {
        Ok(StreamEvent::FundingRate {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "fundingRate")?,
            next_funding_time: None,
            timestamp: data.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    /// Parse WebSocket order update message
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "orderType").unwrap_or("limit") {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match Self::get_str(data, "type").unwrap_or("open") {
            "open" => OrderStatus::New,
            "match" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "canceled" => OrderStatus::Canceled,
            _ => OrderStatus::New,
        };

        Ok(OrderUpdateEvent {
            order_id: Self::get_str(data, "orderId").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "clientOid").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            quantity: Self::get_f64(data, "size").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "filledSize").unwrap_or(0.0),
            average_price: None,
            last_fill_price: Self::get_f64(data, "matchPrice"),
            last_fill_quantity: Self::get_f64(data, "matchSize"),
            last_fill_commission: None,
            commission_asset: None,
            trade_id: Self::get_str(data, "tradeId").map(String::from),
            timestamp: data.get("ts")
                .and_then(|t| t.as_i64())
                .map(|t| if t > 1_000_000_000_000_000 { t / 1_000_000 } else { t })
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket balance update message
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<BalanceUpdateEvent> {
        let asset = Self::get_str(data, "currency").unwrap_or("").to_string();
        let free = Self::get_f64(data, "available")
            .or_else(|| Self::get_f64(data, "availableBalance"))
            .unwrap_or(0.0);
        let locked = Self::get_f64(data, "hold")
            .or_else(|| Self::get_f64(data, "holdBalance"))
            .unwrap_or(0.0);
        let total = Self::get_f64(data, "total")
            .or_else(|| Self::get_f64(data, "walletBalance"))
            .unwrap_or(free + locked);
        let delta = Self::get_f64(data, "availableChange");

        // Parse reason from relationEvent
        let reason = match Self::get_str(data, "relationEvent").unwrap_or("") {
            "trade.hold" => Some(BalanceChangeReason::Trade),
            "trade.setted" => Some(BalanceChangeReason::Trade),
            "main.deposit" => Some(BalanceChangeReason::Deposit),
            "main.withdraw" => Some(BalanceChangeReason::Withdraw),
            _ => Some(BalanceChangeReason::Other),
        };

        Ok(BalanceUpdateEvent {
            asset,
            free,
            locked,
            total,
            delta,
            reason,
            timestamp: data.get("time")
                .and_then(|t| t.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| data.get("timestamp").and_then(|t| t.as_i64()))
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket position update message
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<PositionUpdateEvent> {
        let quantity = Self::get_f64(data, "currentQty")
            .or_else(|| Self::get_f64(data, "qty"))
            .unwrap_or(0.0);

        let side = if quantity > 0.0 {
            PositionSide::Long
        } else if quantity < 0.0 {
            PositionSide::Short
        } else {
            PositionSide::Both
        };

        let reason = match Self::get_str(data, "changeReason").unwrap_or("") {
            "positionChange" => Some(PositionChangeReason::Trade),
            "markPriceChange" => Some(PositionChangeReason::Other),
            _ => None,
        };

        Ok(PositionUpdateEvent {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            side,
            quantity: quantity.abs(),
            entry_price: Self::get_f64(data, "avgEntryPrice").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "markPrice"),
            unrealized_pnl: Self::get_f64(data, "unrealisedPnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realisedPnl"),
            liquidation_price: Self::get_f64(data, "liquidationPrice"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32),
            margin_type: if data.get("crossMode").and_then(|v| v.as_bool()).unwrap_or(false) {
                Some(crate::core::MarginType::Cross)
            } else {
                Some(crate::core::MarginType::Isolated)
            },
            reason,
            timestamp: data.get("currentTimestamp")
                .or_else(|| data.get("ts"))
                .and_then(|t| t.as_i64())
                .map(|t| if t > 1_000_000_000_000_000 { t / 1_000_000 } else { t })
                .unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CANCEL ALL
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from DELETE /api/v1/orders (cancel all).
    ///
    /// KuCoin returns `{"code":"200000","data":{"cancelledOrderIds":["id1","id2"]}}`.
    pub fn parse_cancel_all_response(response: &Value) -> ExchangeResult<CancelAllResponse> {
        let data = Self::extract_data(response)?;

        let cancelled_ids = data.get("cancelledOrderIds")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len() as u32)
            .unwrap_or(0);

        Ok(CancelAllResponse {
            cancelled_count: cancelled_ids,
            failed_count: 0,
            details: vec![],
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AMEND ORDER (Futures only)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from POST /api/v1/orders/{orderId} amend.
    ///
    /// KuCoin Futures returns the amended order object under `data`.
    pub fn parse_amend_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        let data = Self::extract_data(response)?;
        Self::parse_order_data(data, symbol)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BATCH ORDERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from POST /api/v1/hf/orders/multi (Spot) or
    /// POST /api/v1/orders/multi (Futures).
    ///
    /// KuCoin batch response is an array of per-order results under `data`.
    /// Each element may contain `orderId` (success) or `msg`/`code` (failure).
    pub fn parse_batch_orders_response(response: &Value) -> ExchangeResult<Vec<OrderResult>> {
        let data = Self::extract_data(response)?;

        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Batch orders response 'data' is not an array".to_string()))?;

        let results = arr.iter().map(|item| {
            // KuCoin batch: each item has `orderId` on success, or `code`+`msg` on failure
            let success = item.get("orderId").and_then(|v| v.as_str()).is_some();
            let order_id = item.get("orderId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let client_oid = item.get("clientOid")
                .and_then(|v| v.as_str())
                .map(String::from);
            let error = if !success {
                item.get("msg")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .or_else(|| Some("Unknown batch order error".to_string()))
            } else {
                None
            };
            let error_code = if !success {
                item.get("code")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i32>().ok())
            } else {
                None
            };

            let order = if success {
                Some(Order {
                    id: order_id,
                    client_order_id: client_oid.clone(),
                    symbol: String::new(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit { price: 0.0 },
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
            } else {
                None
            };

            OrderResult {
                order,
                client_order_id: client_oid,
                success,
                error,
                error_code,
            }
        }).collect();

        Ok(results)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER TRADES (FILLS)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse filled trades from `GET /api/v1/fills` (spot or futures).
    ///
    /// Both spot and futures responses share the same paginated envelope:
    /// `{ "data": { "items": [...], "currentPage": 1, "pageSize": 50, "totalNum": 100, "totalPage": 2 } }`
    pub fn parse_fills(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let data = Self::extract_data(response)?;

        // KuCoin wraps paginated results in a nested object under "data"
        let items = data
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data.items' array".to_string()))?;

        let mut trades = Vec::with_capacity(items.len());

        for item in items {
            let id = Self::require_str(item, "tradeId")?.to_string();
            let order_id = Self::require_str(item, "orderId")?.to_string();
            let symbol = Self::require_str(item, "symbol")?.to_string();

            let side = match Self::require_str(item, "side")? {
                "buy" => OrderSide::Buy,
                _ => OrderSide::Sell,
            };

            let price = Self::require_f64(item, "price")?;
            let quantity = Self::require_f64(item, "size")?;
            let commission = Self::require_f64(item, "fee")?;
            let commission_asset = Self::require_str(item, "feeCurrency")?.to_string();

            let is_maker = Self::get_str(item, "liquidity")
                .map(|l| l == "maker")
                .unwrap_or(false);

            // "createdAt" is Unix milliseconds
            let timestamp = item
                .get("createdAt")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ExchangeError::Parse("Missing 'createdAt'".to_string()))?;

            trades.push(UserTrade {
                id,
                order_id,
                symbol,
                side,
                price,
                quantity,
                commission,
                commission_asset,
                is_maker,
                timestamp,
            });
        }

        Ok(trades)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "code": "200000",
            "data": {
                "price": "42000.50"
            }
        });

        let price = KuCoinParser::parse_price(&response).unwrap();
        assert!((price - 42000.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook_spot() {
        // Spot format: "time" field, string sequence
        let response = json!({
            "code": "200000",
            "data": {
                "time": 1234567890,
                "sequence": "123",
                "bids": [["42000", "1.5"], ["41999", "2.0"]],
                "asks": [["42001", "1.0"], ["42002", "0.5"]]
            }
        });

        let orderbook = KuCoinParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 42000.0).abs() < f64::EPSILON);
        assert_eq!(orderbook.timestamp, 1234567890);
        assert_eq!(orderbook.sequence, Some("123".to_string()));
    }

    #[test]
    fn test_parse_orderbook_futures() {
        // Futures format: "ts" field (nanoseconds), integer sequence
        let response = json!({
            "code": "200000",
            "data": {
                "symbol": "XBTUSDM",
                "sequence": 100,
                "ts": 1604643655040584408i64,
                "bids": [["3200.0", 800], ["3100.0", 100]],
                "asks": [["5000.0", 1000], ["6000.0", 1983]]
            }
        });

        let orderbook = KuCoinParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 3200.0).abs() < f64::EPSILON);
        assert_eq!(orderbook.timestamp, 1604643655040584408i64);
        assert_eq!(orderbook.sequence, Some("100".to_string()));
    }

    #[test]
    fn test_parse_ticker() {
        // KuCoin naming convention:
        // - "buy" = best bid (highest price buyers willing to pay)
        // - "sell" = best ask (lowest price sellers willing to accept)
        // In a normal market: buy < sell (bid < ask)
        let response = json!({
            "code": "200000",
            "data": {
                "time": 1602832092060i64,
                "symbol": "BTC-USDT",
                "buy": "11328.9",    // bestBid
                "sell": "11329.0",   // bestAsk (higher than bid)
                "last": "11328.9",
                "high": "11610",
                "low": "11200",
                "vol": "2282.70993217",
                "volValue": "25550000",
                "changePrice": "100.5",
                "changeRate": "0.0089"
            }
        });

        let ticker = KuCoinParser::parse_ticker(&response).unwrap();

        // bid_price from "buy", ask_price from "sell"
        assert!((ticker.bid_price.unwrap() - 11328.9).abs() < f64::EPSILON,
            "bid_price should be from 'buy' field (11328.9)");
        assert!((ticker.ask_price.unwrap() - 11329.0).abs() < f64::EPSILON,
            "ask_price should be from 'sell' field (11329.0)");

        // Verify bid < ask (sanity check)
        assert!(ticker.bid_price.unwrap() < ticker.ask_price.unwrap(),
            "bid_price must be less than ask_price");

        // Verify other fields
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert!((ticker.last_price - 11328.9).abs() < f64::EPSILON);
        assert_eq!(ticker.timestamp, 1602832092060i64);
    }
}
