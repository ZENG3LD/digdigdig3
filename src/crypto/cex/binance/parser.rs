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
    pub fn parse_exchange_info(response: &serde_json::Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let symbols = response["symbols"].as_array()
            .ok_or_else(|| ExchangeError::Parse("missing symbols array".into()))?;

        let mut result = Vec::with_capacity(symbols.len());
        for s in symbols {
            let status = s["status"].as_str().unwrap_or("").to_string();
            if status != "TRADING" { continue; }

            result.push(SymbolInfo {
                symbol: s["symbol"].as_str().unwrap_or("").to_string(),
                base_asset: s["baseAsset"].as_str().unwrap_or("").to_string(),
                quote_asset: s["quoteAsset"].as_str().unwrap_or("").to_string(),
                status,
                price_precision: s["pricePrecision"].as_u64().unwrap_or(8) as u8,
                quantity_precision: s["quantityPrecision"].as_u64().unwrap_or(8) as u8,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
                min_notional: None,
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

    /// Парсить fee info из /api/v3/account или /sapi/v1/asset/tradeFee
    pub fn parse_fee_info(response: &Value, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Self::check_error(response)?;

        // Trade fee endpoint returns array of {symbol, makerCommission, takerCommission}
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

        // Account endpoint: commissionRates object
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

        Err(ExchangeError::Parse("Cannot extract fee info from response".to_string()))
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
