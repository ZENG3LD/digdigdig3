//! # Binance Response Parser
//!
//! Парсинг JSON ответов от Binance API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, SymbolInfo, FeeInfo,
    OcoResponse, CancelAllResponse, OrderResult,
    AlgoOrderResponse, BracketResponse,
    TransferResponse, DepositAddress, WithdrawResponse, FundsRecord,
    SubAccountResult, SubAccount,
    UserTrade,
    FundingPayment, LedgerEntry, LedgerEntryType,
    AccountType,
};

/// Парсер ответов Binance API
pub struct BinanceParser;

impl BinanceParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

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
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Проверить и обработать ошибки API
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(code) = response.get("code").and_then(|c| c.as_i64()) {
            if code != 200 {
                let msg = response.get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: msg.to_string(),
                });
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить price
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_error(response)?;
        Self::require_f64(response, "price")
    }

    /// Парсить klines
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 12 {
                continue;
            }

            // Binance format: [time, open, high, low, close, volume, close_time, quote_volume, trades, ...]
            let open_time = candle[0].as_i64().unwrap_or(0);
            let close_time = candle[6].as_i64().unwrap_or(0);
            let trades = candle[8].as_i64().unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
                close_time: Some(close_time),
                quote_volume: Self::parse_f64(&candle[7]),
                trades: Some(trades as u64),
            });
        }

        Ok(klines)
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            response.get(key)
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
            timestamp: 0, // Binance doesn't provide timestamp in orderbook
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: response.get("lastUpdateId")
                .and_then(|s| s.as_i64())
                .map(|n| n.to_string()),
        })
    }

    /// Парсить ticker
    ///
    /// Supports both REST API format (long field names) and WebSocket format (short field names)
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        // Detect format by checking for WebSocket-specific short field names
        let is_websocket = response.get("s").is_some() && response.get("c").is_some();

        if is_websocket {
            // WebSocket format: uses short field names
            // Reference: https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-ticker-streams
            Ok(Ticker {
                symbol: Self::get_str(response, "s").unwrap_or("").to_string(),
                last_price: Self::get_f64(response, "c").unwrap_or(0.0),
                bid_price: Self::get_f64(response, "b"),
                ask_price: Self::get_f64(response, "a"),
                high_24h: Self::get_f64(response, "h"),
                low_24h: Self::get_f64(response, "l"),
                volume_24h: Self::get_f64(response, "v"),
                quote_volume_24h: Self::get_f64(response, "q"),
                price_change_24h: Self::get_f64(response, "p"),
                price_change_percent_24h: Self::get_f64(response, "P"),
                timestamp: response.get("E").and_then(|t| t.as_i64()).unwrap_or(0),
            })
        } else {
            // REST API format: uses long field names
            // Reference: https://binance-docs.github.io/apidocs/spot/en/#24hr-ticker-price-change-statistics
            Ok(Ticker {
                symbol: Self::get_str(response, "symbol").unwrap_or("").to_string(),
                last_price: Self::get_f64(response, "lastPrice").unwrap_or(0.0),
                bid_price: Self::get_f64(response, "bidPrice"),
                ask_price: Self::get_f64(response, "askPrice"),
                high_24h: Self::get_f64(response, "highPrice"),
                low_24h: Self::get_f64(response, "lowPrice"),
                volume_24h: Self::get_f64(response, "volume"),
                quote_volume_24h: Self::get_f64(response, "quoteVolume"),
                price_change_24h: Self::get_f64(response, "priceChange"),
                price_change_percent_24h: Self::get_f64(response, "priceChangePercent"),
                timestamp: response.get("closeTime").and_then(|t| t.as_i64()).unwrap_or(0),
            })
        }
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_error(response)?;

        // Response is an array, take first element
        let data = if let Some(arr) = response.as_array() {
            arr.first().ok_or_else(|| ExchangeError::Parse("Empty funding rate array".to_string()))?
        } else {
            response
        };

        Ok(FundingRate {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "fundingRate")?,
            next_funding_time: data.get("fundingTime").and_then(|t| t.as_i64()),
            timestamp: data.get("fundingTime").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(response: &Value, _symbol: &str) -> ExchangeResult<Order> {
        Self::check_error(response)?;
        Self::parse_order_data(response)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("BUY").to_uppercase().as_str() {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("LIMIT").to_uppercase().as_str() {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: data.get("orderId")
                .and_then(|id| id.as_i64())
                .map(|id| id.to_string())
                .unwrap_or_default(),
            client_order_id: Self::get_str(data, "clientOrderId").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "stopPrice"),
            quantity: Self::get_f64(data, "origQty").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "executedQty").unwrap_or(0.0),
            average_price: Self::get_f64(data, "avgPrice")
                .or_else(|| {
                    // Calculate from cummulativeQuoteQty / executedQty
                    let quote = Self::get_f64(data, "cummulativeQuoteQty")?;
                    let qty = Self::get_f64(data, "executedQty")?;
                    if qty > 0.0 {
                        Some(quote / qty)
                    } else {
                        None
                    }
                }),
            commission: None, // Not in standard order response
            commission_asset: None,
            created_at: data.get("time")
                .or_else(|| data.get("transactTime"))
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            updated_at: data.get("updateTime").and_then(|t| t.as_i64()),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить статус ордера
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("NEW").to_uppercase().as_str() {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "PENDING_CANCEL" => OrderStatus::Canceled, // Treat as Canceled
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            "EXPIRED_IN_MATCH" => OrderStatus::Expired,
            _ => OrderStatus::New,
        }
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        arr.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FILLS / USER TRADES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse a list of user trade fills from `/api/v3/myTrades` (spot) or
    /// `/fapi/v1/userTrades` (futures).
    ///
    /// Spot uses `isBuyer: bool` to determine side; futures uses `side: "BUY"/"SELL"`.
    /// Both are handled transparently — whichever field is present wins.
    pub fn parse_user_trades(response: &Value, is_futures: bool) -> ExchangeResult<Vec<UserTrade>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of user trades".to_string()))?;

        arr.iter()
            .map(|item| {
                // Determine side.
                // Futures: "side" field ("BUY" / "SELL").
                // Spot:    "isBuyer" bool field (true = Buy, false = Sell).
                let side = if is_futures {
                    match Self::get_str(item, "side").unwrap_or("BUY").to_uppercase().as_str() {
                        "SELL" => OrderSide::Sell,
                        _ => OrderSide::Buy,
                    }
                } else {
                    let is_buyer = item.get("isBuyer")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    if is_buyer { OrderSide::Buy } else { OrderSide::Sell }
                };

                // Trade id — numeric on both endpoints.
                let id = item.get("id")
                    .and_then(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .ok_or_else(|| ExchangeError::Parse("Missing 'id' in trade".to_string()))?;

                // Order id — numeric on both endpoints.
                let order_id = item.get("orderId")
                    .and_then(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .ok_or_else(|| ExchangeError::Parse("Missing 'orderId' in trade".to_string()))?;

                let symbol = Self::get_str(item, "symbol")
                    .unwrap_or("")
                    .to_string();

                let price = Self::require_f64(item, "price")?;

                // Spot uses "qty", futures uses "qty" as well (both consistent).
                let quantity = Self::require_f64(item, "qty")?;

                let commission = Self::get_f64(item, "commission").unwrap_or(0.0);

                let commission_asset = Self::get_str(item, "commissionAsset")
                    .unwrap_or("")
                    .to_string();

                // is_maker: spot uses "isMaker", futures uses "maker".
                let is_maker = item.get("isMaker")
                    .or_else(|| item.get("maker"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let timestamp = item.get("time")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

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
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balances
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.get("balances")
            .and_then(|b| b.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'balances' array".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "asset").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "free").unwrap_or(0.0);
            let locked = Self::get_f64(item, "locked").unwrap_or(0.0);

            // Skip zero balances
            if free == 0.0 && locked == 0.0 {
                continue;
            }

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    /// Парсить futures account balances
    pub fn parse_futures_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.get("assets")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'assets' array".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "asset").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "availableBalance").unwrap_or(0.0);
            let locked = Self::get_f64(item, "initialMargin")
                .unwrap_or(0.0);

            // Skip zero balances
            if free == 0.0 && locked == 0.0 {
                continue;
            }

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        let mut positions = Vec::new();

        for item in arr {
            if let Some(pos) = Self::parse_position_data(item) {
                positions.push(pos);
            }
        }

        Ok(positions)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить список торговых символов из exchangeInfo
    ///
    /// Возвращает только символы со статусом `TRADING`.
    pub fn parse_exchange_info(response: &serde_json::Value, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let symbols = response["symbols"].as_array()
            .ok_or_else(|| ExchangeError::Parse("missing symbols array".into()))?;

        let mut result = Vec::with_capacity(symbols.len());
        for s in symbols {
            let status = s["status"].as_str().unwrap_or("").to_string();
            if status != "TRADING" { continue; }

            let filters = s["filters"].as_array();

            let tick_size = filters.and_then(|f| {
                f.iter()
                    .find(|x| x["filterType"] == "PRICE_FILTER")
                    .and_then(|x| x["tickSize"].as_str())
                    .and_then(|s| s.parse::<f64>().ok())
            });

            let lot_size_filter = filters.and_then(|f| {
                f.iter().find(|x| x["filterType"] == "LOT_SIZE")
            });

            let step_size = lot_size_filter
                .and_then(|x| x["stepSize"].as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_quantity = lot_size_filter
                .and_then(|x| x["minQty"].as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let max_quantity = lot_size_filter
                .and_then(|x| x["maxQty"].as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_notional = filters.and_then(|f| {
                f.iter()
                    .find(|x| x["filterType"] == "MIN_NOTIONAL" || x["filterType"] == "NOTIONAL")
                    .and_then(|x| x["minNotional"].as_str())
                    .and_then(|s| s.parse::<f64>().ok())
            });

            result.push(SymbolInfo {
                symbol: s["symbol"].as_str().unwrap_or("").to_string(),
                base_asset: s["baseAsset"].as_str().unwrap_or("").to_string(),
                quote_asset: s["quoteAsset"].as_str().unwrap_or("").to_string(),
                status,
                price_precision: s["pricePrecision"].as_u64().unwrap_or(8) as u8,
                quantity_precision: s["quantityPrecision"].as_u64().unwrap_or(8) as u8,
                min_quantity,
                max_quantity,
                tick_size,
                step_size,
                min_notional,
                account_type,
            });
        }
        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OCO / CANCEL ALL / BATCH
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить ответ OCO ордера
    ///
    /// Binance returns `orderReports` array with 2 orders plus a `listOrderId`.
    pub fn parse_oco_response(response: &Value) -> ExchangeResult<OcoResponse> {
        Self::check_error(response)?;

        let reports = response.get("orderReports")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orderReports' in OCO response".to_string()))?;

        if reports.len() < 2 {
            return Err(ExchangeError::Parse(format!(
                "Expected 2 orders in OCO response, got {}", reports.len()
            )));
        }

        let first_order = Self::parse_order_data(&reports[0])?;
        let second_order = Self::parse_order_data(&reports[1])?;

        let list_id = response.get("listClientOrderId")
            .or_else(|| response.get("orderListId"))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else {
                    v.as_i64().map(|n| n.to_string())
                }
            });

        Ok(OcoResponse {
            first_order,
            second_order,
            list_id,
        })
    }

    /// Парсить ответ OTOCO ордера (Bracket на Binance Spot)
    ///
    /// Binance OTOCO returns `orderReports` array with 3 orders:
    /// - [0] working order (entry)
    /// - [1] pending take-profit
    /// - [2] pending stop-loss
    pub fn parse_otoco_response(response: &Value) -> ExchangeResult<BracketResponse> {
        Self::check_error(response)?;

        let reports = response.get("orderReports")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orderReports' in OTOCO response".to_string()))?;

        if reports.len() < 3 {
            return Err(ExchangeError::Parse(format!(
                "Expected 3 orders in OTOCO response, got {}", reports.len()
            )));
        }

        let entry_order = Self::parse_order_data(&reports[0])?;
        let tp_order = Self::parse_order_data(&reports[1])?;
        let sl_order = Self::parse_order_data(&reports[2])?;

        Ok(BracketResponse {
            entry_order,
            tp_order,
            sl_order,
        })
    }

    /// Парсить ответ Algo ордера (TWAP, VP и т.д.)
    ///
    /// Binance Algo API returns: `{"code": 0, "msg": "", "data": {"clientAlgoId": "...", "success": true}}`
    pub fn parse_algo_order_response(response: &Value) -> ExchangeResult<AlgoOrderResponse> {
        // Check top-level code (Algo API uses 0 for success, not 200)
        if let Some(code) = response.get("code").and_then(|c| c.as_i64()) {
            if code != 0 {
                let msg = response.get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Algo order failed");
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: msg.to_string(),
                });
            }
        }

        // Extract from nested `data` object if present
        let data = response.get("data").unwrap_or(response);

        let algo_id = data.get("clientAlgoId")
            .or_else(|| data.get("algoId"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        let status = if data.get("success").and_then(|v| v.as_bool()).unwrap_or(true) {
            "Running".to_string()
        } else {
            "Failed".to_string()
        };

        Ok(AlgoOrderResponse {
            algo_id,
            status,
            executed_count: None,
            total_count: None,
        })
    }

    /// Парсить ответ cancel-all (массив отменённых ордеров или пустой объект)
    ///
    /// Binance Spot `DELETE /api/v3/openOrders` returns an array of cancelled orders.
    /// Binance Futures `DELETE /fapi/v1/allOpenOrders` returns `{"code": 200, "msg": "The operation of cancel all open order is done."}`.
    pub fn parse_cancel_all_response(response: &Value) -> ExchangeResult<CancelAllResponse> {
        Self::check_error(response)?;

        // Futures returns a success code object
        if response.is_object() && !response.as_object().map(|o| o.contains_key("code")).unwrap_or(false) {
            // Spot case: might be an array wrapped in object — shouldn't happen, but handle
            return Ok(CancelAllResponse {
                cancelled_count: 0,
                failed_count: 0,
                details: vec![],
            });
        }

        // Spot case: array of cancelled orders
        if let Some(arr) = response.as_array() {
            let details: Vec<OrderResult> = arr.iter().map(|item| {
                match Self::parse_order_data(item) {
                    Ok(order) => OrderResult {
                        order: Some(order),
                        client_order_id: None,
                        success: true,
                        error: None,
                        error_code: None,
                    },
                    Err(e) => OrderResult {
                        order: None,
                        client_order_id: None,
                        success: false,
                        error: Some(e.to_string()),
                        error_code: None,
                    },
                }
            }).collect();

            let cancelled_count = details.iter().filter(|d| d.success).count() as u32;
            let failed_count = details.iter().filter(|d| !d.success).count() as u32;

            return Ok(CancelAllResponse {
                cancelled_count,
                failed_count,
                details,
            });
        }

        // Futures success object: {"code": 200, "msg": "..."}
        Ok(CancelAllResponse {
            cancelled_count: 0, // Futures does not return individual cancelled orders
            failed_count: 0,
            details: vec![],
        })
    }

    /// Парсить ответ batch orders
    ///
    /// Binance Futures batch returns an array where each element is either
    /// an order object or an error object with `code`/`msg`.
    pub fn parse_batch_orders_response(response: &Value) -> ExchangeResult<Vec<OrderResult>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in batch orders response".to_string()))?;

        Ok(arr.iter().map(|item| {
            // Check if this item is an error
            if let Some(code) = item.get("code").and_then(|c| c.as_i64()) {
                if code < 0 {
                    let msg = item.get("msg")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown batch error")
                        .to_string();
                    return OrderResult {
                        order: None,
                        client_order_id: item.get("clientOrderId")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        success: false,
                        error: Some(msg),
                        error_code: Some(code as i32),
                    };
                }
            }

            match Self::parse_order_data(item) {
                Ok(order) => OrderResult {
                    client_order_id: order.client_order_id.clone(),
                    order: Some(order),
                    success: true,
                    error: None,
                    error_code: None,
                },
                Err(e) => OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(e.to_string()),
                    error_code: None,
                },
            }
        }).collect())
    }

    /// Парсить fee info из:
    /// - `/sapi/v1/asset/tradeFee` — array of `{symbol, makerCommission, takerCommission}`
    /// - `/api/v3/account` — object with `commissionRates.{maker, taker}`
    /// - `/fapi/v1/commissionRate` — object with `{makerCommissionRate, takerCommissionRate}`
    pub fn parse_fee_info(response: &Value, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Self::check_error(response)?;

        // Spot trade fee endpoint: array of {symbol, makerCommission, takerCommission}
        if let Some(arr) = response.as_array() {
            if let Some(first) = arr.first() {
                let maker_rate = Self::get_f64(first, "makerCommission").unwrap_or(0.001);
                let taker_rate = Self::get_f64(first, "takerCommission").unwrap_or(0.001);
                return Ok(FeeInfo {
                    maker_rate,
                    taker_rate,
                    symbol: Self::get_str(first, "symbol").map(String::from),
                    tier: None,
                });
            }
            return Err(ExchangeError::Parse("Empty fee array".to_string()));
        }

        // Futures commissionRate endpoint: {symbol, makerCommissionRate, takerCommissionRate}
        if let (Some(maker_rate), Some(taker_rate)) = (
            Self::get_f64(response, "makerCommissionRate"),
            Self::get_f64(response, "takerCommissionRate"),
        ) {
            return Ok(FeeInfo {
                maker_rate,
                taker_rate,
                symbol: Self::get_str(response, "symbol")
                    .map(String::from)
                    .or_else(|| symbol.map(String::from)),
                tier: None,
            });
        }

        // Spot account endpoint: commissionRates object
        if let Some(rates) = response.get("commissionRates") {
            let maker_rate = rates.get("maker")
                .and_then(Self::parse_f64)
                .unwrap_or(0.001);
            let taker_rate = rates.get("taker")
                .and_then(Self::parse_f64)
                .unwrap_or(0.001);
            return Ok(FeeInfo {
                maker_rate,
                taker_rate,
                symbol: symbol.map(String::from),
                tier: None,
            });
        }

        // Futures account endpoint: feeTier (int) with no explicit rates in base response
        // feeTier 0 = Regular (0.02%/0.04%), each tier reduces rates by ~10%
        if let Some(fee_tier) = response.get("feeTier").and_then(|v| v.as_u64()) {
            // Standard Binance USDT-M fee schedule (VIP 0 baseline)
            let (maker_rate, taker_rate) = match fee_tier {
                0 => (0.0002, 0.0004),
                1 => (0.00016, 0.0004),
                2 => (0.00014, 0.00035),
                3 => (0.00012, 0.00032),
                4 => (0.0001, 0.0003),
                5 => (0.00008, 0.00027),
                6 => (0.00006, 0.00025),
                7 => (0.00005, 0.00022),
                8 => (0.00003, 0.0002),
                9 => (0.0, 0.00017),
                _ => (0.0002, 0.0004),
            };
            return Ok(FeeInfo {
                maker_rate,
                taker_rate,
                symbol: symbol.map(String::from),
                tier: Some(format!("VIP{}", fee_tier)),
            });
        }

        Err(ExchangeError::Parse("Cannot extract fee info from response".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT TRANSFERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse transfer response from POST /sapi/v1/asset/transfer
    ///
    /// Binance returns: `{"tranId": 13526853623}`
    pub fn parse_transfer_response(response: &Value, asset: &str, amount: f64) -> ExchangeResult<TransferResponse> {
        Self::check_error(response)?;

        let transfer_id = response.get("tranId")
            .and_then(|v| {
                if let Some(n) = v.as_i64() {
                    Some(n.to_string())
                } else {
                    v.as_str().map(String::from)
                }
            })
            .ok_or_else(|| ExchangeError::Parse("Missing 'tranId' in transfer response".to_string()))?;

        Ok(TransferResponse {
            transfer_id,
            status: "Successful".to_string(),
            asset: asset.to_string(),
            amount,
            timestamp: None,
        })
    }

    /// Parse transfer history from GET /sapi/v1/asset/transfer
    ///
    /// Binance returns: `{"total": N, "rows": [{...}, ...]}`
    pub fn parse_transfer_history(response: &Value) -> ExchangeResult<Vec<TransferResponse>> {
        Self::check_error(response)?;

        let rows = response.get("rows")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'rows' in transfer history response".to_string()))?;

        let mut result = Vec::with_capacity(rows.len());
        for item in rows {
            let transfer_id = item.get("tranId")
                .and_then(|v| {
                    if let Some(n) = v.as_i64() {
                        Some(n.to_string())
                    } else {
                        v.as_str().map(String::from)
                    }
                })
                .unwrap_or_default();

            let asset = Self::get_str(item, "asset").unwrap_or("").to_string();
            let amount = Self::get_f64(item, "amount").unwrap_or(0.0);
            let status = Self::get_str(item, "status").unwrap_or("").to_string();
            let timestamp = item.get("timestamp").and_then(|t| t.as_i64());

            result.push(TransferResponse {
                transfer_id,
                status,
                asset,
                amount,
                timestamp,
            });
        }

        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTODIAL FUNDS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse deposit address from GET /sapi/v1/capital/deposit/address
    ///
    /// Binance returns: `{"address": "...", "coin": "BTC", "tag": "", "url": "..."}`
    pub fn parse_deposit_address(response: &Value) -> ExchangeResult<DepositAddress> {
        Self::check_error(response)?;

        let address = Self::get_str(response, "address")
            .ok_or_else(|| ExchangeError::Parse("Missing 'address' in deposit address response".to_string()))?
            .to_string();

        let asset = Self::get_str(response, "coin").unwrap_or("").to_string();
        let tag = Self::get_str(response, "tag")
            .filter(|s| !s.is_empty())
            .map(String::from);
        let network = Self::get_str(response, "network")
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(DepositAddress {
            address,
            tag,
            network,
            asset,
            created_at: None,
        })
    }

    /// Parse withdraw response from POST /sapi/v1/capital/withdraw/apply
    ///
    /// Binance returns: `{"id": "7213fea8e94b4a5593d507237e5a555b"}`
    pub fn parse_withdraw_response(response: &Value) -> ExchangeResult<WithdrawResponse> {
        Self::check_error(response)?;

        let withdraw_id = Self::get_str(response, "id")
            .ok_or_else(|| ExchangeError::Parse("Missing 'id' in withdraw response".to_string()))?
            .to_string();

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Parse deposit history from GET /sapi/v1/capital/deposit/hisrec
    ///
    /// Binance returns an array of deposit records.
    pub fn parse_deposit_history(response: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in deposit history response".to_string()))?;

        let mut result = Vec::with_capacity(arr.len());
        for item in arr {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let asset = Self::get_str(item, "coin").unwrap_or("").to_string();
            let amount = Self::get_f64(item, "amount").unwrap_or(0.0);
            let tx_hash = Self::get_str(item, "txId")
                .filter(|s| !s.is_empty())
                .map(String::from);
            let network = Self::get_str(item, "network")
                .filter(|s| !s.is_empty())
                .map(String::from);
            // Binance deposit status: 0=pending, 1=credited, 6=wrong_deposit, 7=waiting_user_confirm
            let status_code = item.get("status").and_then(|s| s.as_i64()).unwrap_or(0);
            let status = match status_code {
                0 => "Pending",
                1 => "Credited",
                _ => "Unknown",
            }.to_string();
            let timestamp = item.get("insertTime").and_then(|t| t.as_i64()).unwrap_or(0);

            result.push(FundsRecord::Deposit {
                id,
                asset,
                amount,
                tx_hash,
                network,
                status,
                timestamp,
            });
        }

        Ok(result)
    }

    /// Parse withdrawal history from GET /sapi/v1/capital/withdraw/history
    ///
    /// Binance returns an array of withdrawal records.
    pub fn parse_withdrawal_history(response: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in withdrawal history response".to_string()))?;

        let mut result = Vec::with_capacity(arr.len());
        for item in arr {
            let id = Self::get_str(item, "id").unwrap_or("").to_string();
            let asset = Self::get_str(item, "coin").unwrap_or("").to_string();
            let amount = Self::get_f64(item, "amount").unwrap_or(0.0);
            let fee = Self::get_f64(item, "transactionFee");
            let address = Self::get_str(item, "address").unwrap_or("").to_string();
            let tag = Self::get_str(item, "addressTag")
                .filter(|s| !s.is_empty())
                .map(String::from);
            let tx_hash = Self::get_str(item, "txId")
                .filter(|s| !s.is_empty())
                .map(String::from);
            let network = Self::get_str(item, "network")
                .filter(|s| !s.is_empty())
                .map(String::from);
            // Binance withdrawal status: 0=email_sent, 1=cancelled, 2=awaiting_approval,
            //   3=rejected, 4=processing, 5=failure, 6=completed
            let status_code = item.get("status").and_then(|s| s.as_i64()).unwrap_or(0);
            let status = match status_code {
                0 => "EmailSent",
                1 => "Cancelled",
                2 => "AwaitingApproval",
                3 => "Rejected",
                4 => "Processing",
                5 => "Failed",
                6 => "Completed",
                _ => "Unknown",
            }.to_string();
            let timestamp = item.get("applyTime")
                .and_then(|t| t.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| item.get("applyTime").and_then(|t| t.as_i64()))
                .unwrap_or(0);

            result.push(FundsRecord::Withdrawal {
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
            });
        }

        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUB-ACCOUNTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse sub-account create response from POST /sapi/v1/sub-account/virtualSubAccount
    ///
    /// Binance returns: `{"email": "virtual_sub_email@binance.com"}`
    pub fn parse_sub_account_create(response: &Value) -> ExchangeResult<SubAccountResult> {
        Self::check_error(response)?;

        let email = Self::get_str(response, "email").unwrap_or("").to_string();

        Ok(SubAccountResult {
            id: Some(email.clone()),
            name: Some(email),
            accounts: Vec::new(),
            transaction_id: None,
        })
    }

    /// Parse sub-account list from GET /sapi/v1/sub-account/list
    ///
    /// Binance returns: `{"subAccounts": [{...}, ...]}`
    pub fn parse_sub_account_list(response: &Value) -> ExchangeResult<SubAccountResult> {
        Self::check_error(response)?;

        let arr = response.get("subAccounts")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'subAccounts' in list response".to_string()))?;

        let accounts: Vec<SubAccount> = arr.iter().map(|item| {
            SubAccount {
                id: Self::get_str(item, "email").unwrap_or("").to_string(),
                name: Self::get_str(item, "email").unwrap_or("").to_string(),
                status: if item.get("isFreeze").and_then(|v| v.as_bool()).unwrap_or(false) {
                    "Frozen".to_string()
                } else {
                    "Normal".to_string()
                },
            }
        }).collect();

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts,
            transaction_id: None,
        })
    }

    /// Parse sub-account universal transfer response from POST /sapi/v1/sub-account/universalTransfer
    ///
    /// Binance returns: `{"tranId": 12345, "clientTranId": "..."}`
    pub fn parse_sub_account_transfer(response: &Value) -> ExchangeResult<SubAccountResult> {
        Self::check_error(response)?;

        let tran_id = response.get("tranId")
            .and_then(|v| {
                if let Some(n) = v.as_i64() {
                    Some(n.to_string())
                } else {
                    v.as_str().map(String::from)
                }
            });

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts: Vec::new(),
            transaction_id: tran_id,
        })
    }

    /// Parse sub-account assets/balance from GET /sapi/v3/sub-account/assets
    ///
    /// Binance returns: `{"balances": [{...}, ...]}`
    pub fn parse_sub_account_assets(response: &Value) -> ExchangeResult<SubAccountResult> {
        Self::check_error(response)?;

        // The balance data is present but we store the transaction_id as a summary marker.
        // Callers who want detailed balance data should parse the raw response themselves.
        let balance_count = response.get("balances")
            .and_then(|b| b.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts: Vec::new(),
            transaction_id: Some(format!("balance_assets_count={}", balance_count)),
        })
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "symbol")?.to_string();
        let position_amt = Self::get_f64(data, "positionAmt").unwrap_or(0.0);

        // Skip empty positions
        if position_amt.abs() < f64::EPSILON {
            return None;
        }

        // Parse position side
        let side = match Self::get_str(data, "positionSide").unwrap_or("BOTH") {
            "LONG" => PositionSide::Long,
            "SHORT" => PositionSide::Short,
            _ => {
                // For BOTH mode, determine side from position amount
                if position_amt > 0.0 {
                    PositionSide::Long
                } else {
                    PositionSide::Short
                }
            }
        };

        let leverage = Self::get_f64(data, "leverage")
            .map(|l| l as u32)
            .unwrap_or(1);

        let margin_type = match Self::get_str(data, "marginType").unwrap_or("cross") {
            "isolated" => crate::core::MarginType::Isolated,
            _ => crate::core::MarginType::Cross,
        };

        Some(Position {
            symbol,
            side,
            quantity: position_amt.abs(),
            entry_price: Self::get_f64(data, "entryPrice").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "markPrice"),
            unrealized_pnl: Self::get_f64(data, "unRealizedProfit").unwrap_or(0.0),
            realized_pnl: None, // Not in positionRisk response
            leverage,
            liquidation_price: Self::get_f64(data, "liquidationPrice"),
            margin: Self::get_f64(data, "isolatedMargin"),
            margin_type,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding payments from `GET /fapi/v1/income?incomeType=FUNDING_FEE`
    ///
    /// Response array item: `{"symbol":"BTCUSDT","incomeType":"FUNDING_FEE","income":"-0.01","asset":"USDT","time":1672531200000}`
    pub fn parse_funding_payments(response: &Value) -> ExchangeResult<Vec<FundingPayment>> {
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for income history".to_string()))?;

        let mut payments = Vec::with_capacity(arr.len());
        for item in arr {
            let symbol = Self::get_str(item, "symbol")
                .ok_or_else(|| ExchangeError::Parse("Missing 'symbol' in income record".to_string()))?
                .to_string();
            let payment: f64 = item.get("income")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| item.get("income").and_then(|v| v.as_f64()))
                .unwrap_or(0.0);
            let asset = Self::get_str(item, "asset").unwrap_or("USDT").to_string();
            let timestamp = item.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
            payments.push(FundingPayment {
                symbol,
                funding_rate: 0.0,
                position_size: 0.0,
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

    /// Parse ledger entries from `GET /fapi/v1/income` (all incomeTypes).
    ///
    /// Maps Binance `incomeType` to `LedgerEntryType`.
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for income history".to_string()))?;

        let mut entries = Vec::with_capacity(arr.len());
        for item in arr {
            let symbol = Self::get_str(item, "symbol").unwrap_or("").to_string();
            let income_type = Self::get_str(item, "incomeType").unwrap_or("OTHER");
            let amount: f64 = item.get("income")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| item.get("income").and_then(|v| v.as_f64()))
                .unwrap_or(0.0);
            let asset = Self::get_str(item, "asset").unwrap_or("USDT").to_string();
            let timestamp = item.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
            let entry_type = match income_type {
                "REALIZED_PNL" => LedgerEntryType::Trade,
                "FUNDING_FEE" => LedgerEntryType::Funding,
                "COMMISSION" => LedgerEntryType::Fee,
                "COMMISSION_REBATE" => LedgerEntryType::Rebate,
                "TRANSFER" => LedgerEntryType::Transfer,
                "LIQUIDATION_FEE" => LedgerEntryType::Liquidation,
                "DELIVERY_SETTLEMENT" => LedgerEntryType::Settlement,
                other => LedgerEntryType::Other(other.to_string()),
            };
            let id = format!("{}-{}-{}", timestamp, income_type, symbol);
            entries.push(LedgerEntry {
                id,
                asset,
                amount,
                balance: None,
                entry_type,
                description: format!("{} {}", income_type, symbol),
                ref_id: None,
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
    fn test_parse_price() {
        let response = json!({
            "symbol": "BTCUSDT",
            "price": "42000.50"
        });

        let price = BinanceParser::parse_price(&response).unwrap();
        assert!((price - 42000.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "lastUpdateId": 1027024,
            "bids": [["42000", "1.5"], ["41999", "2.0"]],
            "asks": [["42001", "1.0"], ["42002", "0.5"]]
        });

        let orderbook = BinanceParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 42000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "42000.50",
            "bidPrice": "42000.00",
            "askPrice": "42001.00",
            "highPrice": "43000.00",
            "lowPrice": "41000.00",
            "volume": "1000.0",
            "quoteVolume": "42000000.0",
            "priceChange": "100.5",
            "priceChangePercent": "0.24",
            "closeTime": 1499869899040i64
        });

        let ticker = BinanceParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert!((ticker.last_price - 42000.50).abs() < f64::EPSILON);
        assert!((ticker.bid_price.unwrap() - 42000.0).abs() < f64::EPSILON);
        assert!((ticker.ask_price.unwrap() - 42001.0).abs() < f64::EPSILON);
    }
}
