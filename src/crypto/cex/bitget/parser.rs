//! # Bitget Response Parser
//!
//! Парсинг JSON ответов от Bitget API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, OrderBookLevel, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, UserTrade, LedgerEntry, LedgerEntryType,
    AccountType,
};

/// Парсер ответов Bitget API
pub struct BitgetParser;

impl BitgetParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Извлечь data из response
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check for error first
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("00000");

        if code != "00000" {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code.parse().unwrap_or(-1),
                message: msg.to_string(),
            });
        }

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

    /// Парсить i64 из поля
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_i64()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить price (from ticker)
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;

        // V2 API: data is an array of tickers
        let ticker_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty ticker array".to_string()))?
        } else {
            data
        };

        // V2 uses "lastPr" field for last price
        Self::get_f64(ticker_data, "lastPr")
            .or_else(|| Self::get_f64(ticker_data, "close"))
            .or_else(|| Self::get_f64(ticker_data, "last"))
            .ok_or_else(|| ExchangeError::Parse("Missing price field".to_string()))
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

            // Bitget format: [timestamp, open, high, low, close, baseVolume, quoteVolume]
            let open_time = candle[0].as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);

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

        Ok(klines)
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_data(response)?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Парсить ticker (V2 API format)
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;

        // V2 API: data is an array of tickers
        let ticker_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty ticker array".to_string()))?
        } else {
            data
        };

        // V2 uses: lastPr, bidPr, askPr, ts
        let last_price = Self::get_f64(ticker_data, "lastPr")
            .or_else(|| Self::get_f64(ticker_data, "close"))
            .or_else(|| Self::get_f64(ticker_data, "last"))
            .unwrap_or(0.0);

        let bid_price = Self::get_f64(ticker_data, "bidPr")
            .or_else(|| Self::get_f64(ticker_data, "bestBid"));

        let ask_price = Self::get_f64(ticker_data, "askPr")
            .or_else(|| Self::get_f64(ticker_data, "bestAsk"));

        let timestamp = Self::get_i64(ticker_data, "ts")
            .or_else(|| Self::get_i64(ticker_data, "timestamp"))
            .unwrap_or(0);

        Ok(Ticker {
            symbol: Self::get_str(ticker_data, "symbol").unwrap_or("").to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: Self::get_f64(ticker_data, "high24h"),
            low_24h: Self::get_f64(ticker_data, "low24h"),
            volume_24h: Self::get_f64(ticker_data, "baseVolume")
                .or_else(|| Self::get_f64(ticker_data, "baseVol")),
            quote_volume_24h: Self::get_f64(ticker_data, "quoteVolume")
                .or_else(|| Self::get_f64(ticker_data, "quoteVol")),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(ticker_data, "change24h")
                .or_else(|| Self::get_f64(ticker_data, "priceChangePercent"))
                .map(|r| r * 100.0),
            timestamp,
        })
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let data = Self::extract_data(response)?;

        Ok(FundingRate {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "fundingRate")?,
            next_funding_time: Self::get_i64(data, "fundingTime"),
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info (symbol list) from Bitget response
    ///
    /// Spot: data = [{ symbol, baseCoin, quoteCoin, status, minTradeAmount, maxTradeAmount,
    ///                 quantityPrecision, pricePrecision, quotePrecision }]
    /// Futures: data = [{ symbol, baseCoin, quoteCoin, status, sizeMultiplier, priceEndStep,
    ///                    minTradeNum, maxTradeNum }]
    ///
    /// Filters to active symbols only (status == "online" for spot, "normal" for futures).
    pub fn parse_exchange_info(response: &Value, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of symbols".to_string()))?;

        let symbols = arr.iter()
            .filter_map(|item| {
                let symbol = Self::get_str(item, "symbol")?.to_string();
                let base_asset = Self::get_str(item, "baseCoin").unwrap_or("").to_string();
                let quote_asset = Self::get_str(item, "quoteCoin")
                    .or_else(|| Self::get_str(item, "marginCoin"))
                    .unwrap_or("")
                    .to_string();

                // Filter inactive symbols
                let status_raw = Self::get_str(item, "status").unwrap_or("offline");
                match status_raw {
                    "online" | "normal" | "NORMAL" => {}
                    _ => return None,
                }

                let status = "TRADING".to_string();

                // Precision fields differ between spot and futures
                let price_precision = item.get("pricePrecision")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u8>().ok())
                    .or_else(|| item.get("pricePrecision").and_then(|v| v.as_i64()).map(|p| p as u8))
                    .or_else(|| {
                        // Derive from priceEndStep (tick size)
                        Self::get_f64(item, "priceEndStep").map(|t| {
                            let s = format!("{:.10}", t);
                            let trimmed = s.trim_end_matches('0');
                            if let Some(dot_pos) = trimmed.find('.') {
                                (trimmed.len() - dot_pos - 1) as u8
                            } else {
                                0u8
                            }
                        })
                    })
                    .unwrap_or(8);

                let quantity_precision = item.get("quantityPrecision")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u8>().ok())
                    .or_else(|| item.get("quantityPrecision").and_then(|v| v.as_i64()).map(|p| p as u8))
                    .or_else(|| {
                        Self::get_f64(item, "sizeMultiplier").map(|t| {
                            let s = format!("{:.10}", t);
                            let trimmed = s.trim_end_matches('0');
                            if let Some(dot_pos) = trimmed.find('.') {
                                (trimmed.len() - dot_pos - 1) as u8
                            } else {
                                0u8
                            }
                        })
                    })
                    .unwrap_or(8);

                let min_quantity = Self::get_f64(item, "minTradeAmount")
                    .or_else(|| Self::get_f64(item, "minTradeNum"));
                let max_quantity = Self::get_f64(item, "maxTradeAmount")
                    .or_else(|| Self::get_f64(item, "maxTradeNum"));

                // priceEndStep is the explicit tick size (futures); for spot use
                // pricePlace to derive 10^(-pricePlace) when priceEndStep is absent.
                let tick_size = Self::get_f64(item, "priceEndStep")
                    .or_else(|| {
                        item.get("pricePlace")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<u32>().ok())
                            .map(|places| 10f64.powi(-(places as i32)))
                    });

                // sizeMultiplier is the quantity step for futures contracts.
                let step_size = Self::get_f64(item, "sizeMultiplier");

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
                    min_notional: None,
                    account_type,
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

        let order_type = match Self::get_str(data, "orderType").unwrap_or("limit").to_lowercase().as_str() {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "orderId").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "clientOrderId")
                .or_else(|| Self::get_str(data, "clientOid"))
                .map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: None,
            quantity: Self::get_f64(data, "quantity")
                .or_else(|| Self::get_f64(data, "size"))
                .unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "fillQuantity")
                .or_else(|| Self::get_f64(data, "fillSize"))
                .unwrap_or(0.0),
            average_price: Self::get_f64(data, "fillPrice")
                .or_else(|| Self::get_f64(data, "priceAvg")),
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, "cTime").unwrap_or(0),
            updated_at: Self::get_i64(data, "uTime"),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить статус ордера
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("new") {
            "init" => OrderStatus::New,
            "new" => OrderStatus::New,
            "partial_fill" | "partially_filled" => OrderStatus::PartiallyFilled,
            "full_fill" | "filled" => OrderStatus::Filled,
            "canceled" | "cancelled" => OrderStatus::Canceled,
            "failed" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        }
    }

    /// Парсить список ордеров (V2 API format)
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;

        // V2 API: unfilled-orders endpoint returns { orderList: [...], maxId, minId }
        let arr = if let Some(order_list) = data.get("orderList") {
            order_list.as_array()
                .ok_or_else(|| ExchangeError::Parse("orderList is not an array".to_string()))?
        } else {
            // Fallback: data is array directly
            data.as_array()
                .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?
        };

        arr.iter()
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
            let asset = Self::get_str(item, "coin")
                .or_else(|| Self::get_str(item, "coinName"))
                .unwrap_or("")
                .to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "available").unwrap_or(0.0);
            let locked = Self::get_f64(item, "frozen")
                .or_else(|| Self::get_f64(item, "locked"))
                .unwrap_or(0.0);

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    /// Парсить futures account
    pub fn parse_futures_account(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let currency = Self::get_str(data, "marginCoin").unwrap_or("USDT").to_string();
        let available = Self::get_f64(data, "available").unwrap_or(0.0);
        let locked = Self::get_f64(data, "locked").unwrap_or(0.0);

        Ok(vec![Balance {
            asset: currency,
            free: available,
            locked,
            total: available + locked,
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

        // Data might be array or single object
        if let Some(arr) = data.as_array() {
            if let Some(first) = arr.first() {
                return Self::parse_position_data(first)
                    .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()));
            }
        }

        Self::parse_position_data(data)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "symbol")?.to_string();

        // Bitget uses "total" for position size
        let total = Self::get_f64(data, "total").unwrap_or(0.0);

        // Skip empty positions
        if total.abs() < f64::EPSILON {
            return None;
        }

        // Determine side from holdSide field
        let side = match Self::get_str(data, "holdSide").unwrap_or("long") {
            "short" => PositionSide::Short,
            "long" => PositionSide::Long,
            _ => PositionSide::Both,
        };

        Some(Position {
            symbol,
            side,
            quantity: total.abs(),
            entry_price: Self::get_f64(data, "averageOpenPrice")
                .or_else(|| Self::get_f64(data, "openPriceAvg"))
                .unwrap_or(0.0),
            mark_price: Self::get_f64(data, "marketPrice"),
            unrealized_pnl: Self::get_f64(data, "unrealizedPL")
                .or_else(|| Self::get_f64(data, "unrealizedPnl"))
                .unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "achievedProfits"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32).unwrap_or(1),
            liquidation_price: Self::get_f64(data, "liquidationPrice"),
            margin: Self::get_f64(data, "margin"),
            margin_type: match Self::get_str(data, "marginMode").unwrap_or("crossed") {
                "fixed" => crate::core::MarginType::Isolated,
                _ => crate::core::MarginType::Cross,
            },
            take_profit: None,
            stop_loss: None,
        })
    }
    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker data
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        // WebSocket ticker is in data array
        let ticker_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty ticker data array".to_string()))?
        } else {
            data
        };

        // Parse ticker directly from WebSocket data (not a REST response)
        // V2 uses: lastPr, bidPr, askPr, ts
        let last_price = Self::get_f64(ticker_data, "lastPr")
            .or_else(|| Self::get_f64(ticker_data, "close"))
            .or_else(|| Self::get_f64(ticker_data, "last"))
            .unwrap_or(0.0);

        let bid_price = Self::get_f64(ticker_data, "bidPr")
            .or_else(|| Self::get_f64(ticker_data, "bestBid"));

        let ask_price = Self::get_f64(ticker_data, "askPr")
            .or_else(|| Self::get_f64(ticker_data, "bestAsk"));

        let timestamp = Self::get_i64(ticker_data, "ts")
            .or_else(|| Self::get_i64(ticker_data, "timestamp"))
            .unwrap_or(0);

        Ok(Ticker {
            symbol: Self::get_str(ticker_data, "instId")
                .or_else(|| Self::get_str(ticker_data, "symbol"))
                .unwrap_or("")
                .to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: Self::get_f64(ticker_data, "high24h"),
            low_24h: Self::get_f64(ticker_data, "low24h"),
            volume_24h: Self::get_f64(ticker_data, "baseVolume")
                .or_else(|| Self::get_f64(ticker_data, "baseVol")),
            quote_volume_24h: Self::get_f64(ticker_data, "quoteVolume")
                .or_else(|| Self::get_f64(ticker_data, "quoteVol")),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(ticker_data, "change24h")
                .or_else(|| Self::get_f64(ticker_data, "priceChangePercent"))
                .map(|r| r * 100.0),
            timestamp,
        })
    }

    /// Parse WebSocket trade data.
    ///
    /// `inst_id_fallback` is the `instId` extracted from the outer `arg` object of the WS
    /// message. Bitget data-array items should contain `instId` themselves, but the fallback
    /// is used when it is absent (e.g. during format variations or unexpected server responses)
    /// so that the trade can still be surfaced without flooding stderr with parse errors.
    pub fn parse_ws_trade(data: &Value, inst_id_fallback: Option<&str>) -> ExchangeResult<crate::core::PublicTrade> {
        use crate::core::PublicTrade;
        use crate::core::types::TradeSide;

        // WebSocket trade is in data array
        let trade_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty trade data array".to_string()))?
        } else {
            data
        };

        let id = Self::get_str(trade_data, "tradeId")
            .or_else(|| Self::get_str(trade_data, "id"))
            .unwrap_or("0")
            .to_string();

        // Prefer instId/symbol from the data item itself; fall back to the channel's instId from
        // the outer arg so that non-standard responses do not produce spurious parse errors.
        let symbol = Self::get_str(trade_data, "instId")
            .or_else(|| Self::get_str(trade_data, "symbol"))
            .or(inst_id_fallback)
            .ok_or_else(|| ExchangeError::Parse("Missing 'instId' or 'symbol'".to_string()))?
            .to_string();

        // Bitget WS trade channel sends "price"/"size" (matching REST fills format).
        // The "px"/"sz" variants are kept as fallback in case the field names vary by
        // instType or API revision.
        let price = Self::get_f64(trade_data, "price")
            .or_else(|| Self::get_f64(trade_data, "px"))
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'price'/'px'".to_string()))?;
        let quantity = Self::get_f64(trade_data, "size")
            .or_else(|| Self::get_f64(trade_data, "sz"))
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'size'/'sz'".to_string()))?;
        let timestamp = Self::get_i64(trade_data, "ts").unwrap_or(0);
        let side_str = Self::require_str(trade_data, "side")?;

        let side = match side_str {
            "buy" => TradeSide::Buy,
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id,
            symbol,
            price,
            quantity,
            side,
            timestamp,
        })
    }

    /// Parse WebSocket orderbook delta
    pub fn parse_ws_orderbook_delta(data: &Value) -> ExchangeResult<crate::core::StreamEvent> {
        use crate::core::StreamEvent;

        // WebSocket orderbook is in data array
        let ob_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty orderbook data array".to_string()))?
        } else {
            data
        };

        // Parse orderbook directly from WebSocket data (not a REST response)
        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            ob_data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let orderbook = OrderBook {
            timestamp: Self::get_i64(ob_data, "ts").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        };

        Ok(StreamEvent::OrderbookSnapshot(orderbook))
    }

    /// Parse WebSocket kline data
    pub fn parse_ws_kline(data: &Value) -> ExchangeResult<Kline> {
        // WebSocket kline is in data array (array format)
        let kline_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty kline data array".to_string()))?
        } else {
            data
        };

        // Kline can be array [timestamp, open, high, low, close, volume, quoteVolume, usdtVolume]
        if let Some(arr) = kline_data.as_array() {
            if arr.len() >= 7 {
                let open_time = arr[0].as_str().and_then(|s| s.parse().ok()).unwrap_or(0);
                let open = Self::parse_f64(&arr[1]).unwrap_or(0.0);
                let high = Self::parse_f64(&arr[2]).unwrap_or(0.0);
                let low = Self::parse_f64(&arr[3]).unwrap_or(0.0);
                let close = Self::parse_f64(&arr[4]).unwrap_or(0.0);
                let volume = Self::parse_f64(&arr[5]).unwrap_or(0.0);
                let quote_volume = Self::parse_f64(&arr[6]).unwrap_or(0.0);

                return Ok(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume: Some(quote_volume),
                    close_time: None,
                    trades: None,
                });
            }
        }

        // Fallback: try to parse as object (REST API format)
        let open_time = Self::get_i64(kline_data, "ts").unwrap_or(0);
        let open = Self::require_f64(kline_data, "open")?;
        let high = Self::require_f64(kline_data, "high")?;
        let low = Self::require_f64(kline_data, "low")?;
        let close = Self::require_f64(kline_data, "close")?;
        let volume = Self::get_f64(kline_data, "baseVol").unwrap_or(0.0);
        let quote_volume = Self::get_f64(kline_data, "quoteVol").unwrap_or(0.0);

        Ok(Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: Some(quote_volume),
            close_time: None,
            trades: None,
        })
    }

    /// Parse WebSocket order update
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<crate::core::OrderUpdateEvent> {
        use crate::core::OrderUpdateEvent;

        // WebSocket order update is in data array
        let order_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty order update data array".to_string()))?
        } else {
            data
        };

        let order_id = Self::require_str(order_data, "ordId")?.to_string();
        let symbol = Self::require_str(order_data, "instId")?.to_string();
        let status = Self::require_str(order_data, "state")?;
        let side_str = Self::require_str(order_data, "side")?;
        let order_type_str = Self::require_str(order_data, "ordType")?;

        let order_status = match status {
            "live" => OrderStatus::Open,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "canceled" => OrderStatus::Canceled,
            _ => OrderStatus::Open,
        };

        let side = match side_str {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match order_type_str {
            "limit" => OrderType::Limit { price: 0.0 },
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let quantity = Self::get_f64(order_data, "sz").unwrap_or(0.0);
        let filled_quantity = Self::get_f64(order_data, "fillSz").unwrap_or(0.0);

        Ok(OrderUpdateEvent {
            order_id,
            client_order_id: Self::get_str(order_data, "clOrdId").map(|s| s.to_string()),
            symbol,
            side,
            order_type,
            status: order_status,
            price: Self::get_f64(order_data, "px"),
            quantity,
            filled_quantity,
            average_price: Self::get_f64(order_data, "fillPx"),
            last_fill_price: Self::get_f64(order_data, "fillPx"),
            last_fill_quantity: None,
            last_fill_commission: Self::get_f64(order_data, "fee"),
            commission_asset: Self::get_str(order_data, "feeCcy").map(|s| s.to_string()),
            trade_id: None,
            timestamp: Self::get_i64(order_data, "uTime").unwrap_or(0),
        })
    }

    /// Parse WebSocket balance update
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<crate::core::BalanceUpdateEvent> {
        use crate::core::BalanceUpdateEvent;

        // WebSocket balance update is in data array
        let balance_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty balance update data array".to_string()))?
        } else {
            data
        };

        // Extract asset/coin
        let asset = Self::get_str(balance_data, "coinName")
            .or_else(|| Self::get_str(balance_data, "coin"))
            .or_else(|| Self::get_str(balance_data, "marginCoin"))
            .unwrap_or("UNKNOWN")
            .to_string();

        let free = Self::get_f64(balance_data, "available").unwrap_or(0.0);
        let locked = Self::get_f64(balance_data, "locked").unwrap_or(0.0);
        let total = free + locked;

        Ok(BalanceUpdateEvent {
            asset,
            free,
            locked,
            total,
            delta: None, // Not provided in WebSocket updates
            reason: None, // Not provided in WebSocket updates
            timestamp: Self::get_i64(balance_data, "updateTime")
                .or_else(|| Self::get_i64(balance_data, "uTime"))
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket position update
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<crate::core::PositionUpdateEvent> {
        use crate::core::PositionUpdateEvent;

        // WebSocket position update is in data array
        let pos_data = if let Some(arr) = data.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty position update data array".to_string()))?
        } else {
            data
        };

        let symbol = Self::require_str(pos_data, "instId")?.to_string();
        let side_str = Self::require_str(pos_data, "posSide")?;

        let side = match side_str {
            "long" => PositionSide::Long,
            "short" => PositionSide::Short,
            _ => PositionSide::Long,
        };

        let quantity = Self::get_f64(pos_data, "pos").unwrap_or(0.0);
        let entry_price = Self::get_f64(pos_data, "avgPx").unwrap_or(0.0);
        let unrealized_pnl = Self::get_f64(pos_data, "upl").unwrap_or(0.0);

        Ok(PositionUpdateEvent {
            symbol,
            side,
            quantity,
            entry_price,
            mark_price: Self::get_f64(pos_data, "markPx"),
            unrealized_pnl,
            realized_pnl: None, // Not provided in WebSocket updates
            liquidation_price: Self::get_f64(pos_data, "liqPx"),
            leverage: Self::get_f64(pos_data, "lever").map(|l| l as u32),
            margin_type: None, // Not easily determinable from this data
            reason: None, // Not provided
            timestamp: Self::get_i64(pos_data, "uTime").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER TRADES (FILLS)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse user trade fills from Bitget V2 API.
    ///
    /// Spot (`/api/v2/spot/trade/fills`) and Futures (`/api/v2/mix/order/fills`)
    /// share the same `{"data":[...]}` wrapper format.
    ///
    /// Response item fields:
    /// - `tradeId` / `fillId` — trade identifier
    /// - `orderId` — parent order id
    /// - `symbol` — trading pair
    /// - `side` — "buy" / "sell"
    /// - `priceAvg` — fill price
    /// - `size` — fill quantity
    /// - `fee` — negative means paid commission
    /// - `feeCcy` — commission asset
    /// - `role` — "maker" / "taker"
    /// - `cTime` — creation timestamp ms
    pub fn parse_user_trades(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let data = Self::extract_data(response)?;

        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in 'data' for user trades".to_string()))?;

        arr.iter()
            .map(|item| {
                // tradeId field (spot), fillId field (some futures endpoints)
                let id = item.get("tradeId")
                    .or_else(|| item.get("fillId"))
                    .and_then(|v| v.as_str().map(|s| s.to_string())
                        .or_else(|| v.as_i64().map(|n| n.to_string())))
                    .ok_or_else(|| ExchangeError::Parse("Missing 'tradeId'/'fillId' in trade".to_string()))?;

                let order_id = item.get("orderId")
                    .and_then(|v| v.as_str().map(|s| s.to_string())
                        .or_else(|| v.as_i64().map(|n| n.to_string())))
                    .unwrap_or_default();

                let symbol = Self::get_str(item, "symbol")
                    .unwrap_or("")
                    .to_string();

                let side = match Self::get_str(item, "side").unwrap_or("buy").to_lowercase().as_str() {
                    "sell" | "close_long" | "open_short" => OrderSide::Sell,
                    _ => OrderSide::Buy,
                };

                let price = Self::require_f64(item, "priceAvg")?;
                let quantity = Self::require_f64(item, "size")?;

                // fee is negative (e.g. "-0.01"), take abs value
                let commission = Self::get_f64(item, "fee").unwrap_or(0.0).abs();
                let commission_asset = Self::get_str(item, "feeCcy")
                    .unwrap_or("")
                    .to_string();

                let is_maker = Self::get_str(item, "role")
                    .map(|r| r.to_lowercase() == "maker")
                    .unwrap_or(false);

                let timestamp = Self::get_i64(item, "cTime").unwrap_or(0);

                Ok(UserTrade {
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
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT LEDGER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse spot account bill/ledger records from `GET /api/v2/spot/account/bills`.
    ///
    /// Response:
    /// ```json
    /// {"data":[
    ///   {"billId":"123","coin":"USDT","groupType":"transfer","businessType":"deposit",
    ///    "size":"100","fee":"0","balance":"1000","cTime":"1672531200000"}
    /// ]}
    /// ```
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        let data = Self::extract_data(response)?;
        let list = data.as_array()
            .ok_or_else(|| ExchangeError::Parse(
                "Expected array for bills response data".to_string(),
            ))?;

        let mut entries = Vec::with_capacity(list.len());
        for item in list {
            let id = Self::get_str(item, "billId")
                .unwrap_or("")
                .to_string();

            let asset = Self::get_str(item, "coin")
                .unwrap_or("")
                .to_string();

            // size is always positive; direction is encoded in groupType
            let raw_size = Self::get_str(item, "size")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let fee = Self::get_str(item, "fee")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // Determine sign from groupType: "receive" / "buy" = positive, "send" / "sell" = negative
            let group_type = Self::get_str(item, "groupType").unwrap_or("");
            let amount = match group_type {
                "send" | "withdraw" => -(raw_size + fee.abs()),
                _ => raw_size,
            };

            let balance = Self::get_str(item, "balance")
                .and_then(|s| s.parse::<f64>().ok());

            let business_type = Self::get_str(item, "businessType").unwrap_or("");
            let entry_type = Self::map_bitget_business_type(business_type);
            let description = business_type.to_string();

            let timestamp = Self::get_str(item, "cTime")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);

            entries.push(LedgerEntry {
                id,
                asset,
                amount,
                balance,
                entry_type,
                description,
                ref_id: None,
                timestamp,
            });
        }
        Ok(entries)
    }

    /// Map Bitget `businessType` string to `LedgerEntryType`.
    fn map_bitget_business_type(business_type: &str) -> LedgerEntryType {
        // Bitget businessType values (from API docs): deposit, withdraw, trade, fee,
        // transfer, funding_fee, liquidation, settlement, rebate, etc.
        match business_type {
            "trade" | "buy" | "sell" => LedgerEntryType::Trade,
            "deposit" => LedgerEntryType::Deposit,
            "withdraw" => LedgerEntryType::Withdrawal,
            "funding_fee" | "funding" => LedgerEntryType::Funding,
            "fee" => LedgerEntryType::Fee,
            "rebate" | "maker_rebate" => LedgerEntryType::Rebate,
            "transfer" | "internal_transfer" => LedgerEntryType::Transfer,
            "liquidation" | "force_liquidation" => LedgerEntryType::Liquidation,
            "settlement" | "delivery" => LedgerEntryType::Settlement,
            other => LedgerEntryType::Other(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "code": "00000",
            "msg": "success",
            "data": {
                "close": "50000.50"
            }
        });

        let price = BitgetParser::parse_price(&response).unwrap();
        assert!((price - 50000.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "code": "00000",
            "msg": "success",
            "data": {
                "asks": [["50500.50", "0.1000"], ["50501.00", "0.2000"]],
                "bids": [["50499.50", "0.1500"], ["50499.00", "0.2500"]],
                "ts": "1695806875837"
            }
        });

        let orderbook = BitgetParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].price - 50499.50).abs() < f64::EPSILON);
        assert_eq!(orderbook.timestamp, 1695806875837);
    }

    #[test]
    fn test_parse_ticker() {
        // V2 API returns array of tickers
        let response = json!({
            "code": "00000",
            "msg": "success",
            "data": [{
                "symbol": "BTCUSDT",
                "lastPr": "50500.00",
                "bidPr": "50499.50",
                "askPr": "50500.50",
                "high24h": "52000.00",
                "low24h": "49000.00",
                "baseVolume": "3000.5500",
                "quoteVolume": "150000000.50",
                "ts": "1695806875837"
            }]
        });

        let ticker = BitgetParser::parse_ticker(&response).unwrap();
        assert!((ticker.last_price - 50500.0).abs() < f64::EPSILON);
        assert!((ticker.bid_price.unwrap() - 50499.50).abs() < f64::EPSILON);
        assert!((ticker.ask_price.unwrap() - 50500.50).abs() < f64::EPSILON);

        // Verify bid < ask
        assert!(ticker.bid_price.unwrap() < ticker.ask_price.unwrap());
    }

    #[test]
    fn test_error_response() {
        let response = json!({
            "code": "40001",
            "msg": "Invalid parameter",
            "data": null
        });

        let result = BitgetParser::extract_data(&response);
        assert!(result.is_err());

        if let Err(ExchangeError::Api { code, message }) = result {
            assert_eq!(code, 40001);
            assert_eq!(message, "Invalid parameter");
        } else {
            panic!("Expected API error");
        }
    }
}
