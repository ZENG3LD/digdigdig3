//! # Bybit Parser
//!
//! Parsing Bybit V5 API responses to internal types.
//!
//! ## Response Structure
//!
//! Success response:
//! ```json
//! {
//!   "retCode": 0,
//!   "retMsg": "OK",
//!   "result": { ... },
//!   "time": 1702617474601
//! }
//! ```
//!
//! ## Key Differences from KuCoin
//!
//! - Success code: `retCode: 0` (integer) vs KuCoin `code: "200000"` (string)
//! - Data wrapper: `result` vs `data`
//! - Most data in `result.list` arrays
//! - Kline order: [time, open, high, low, close, volume, turnover]
//! - All timestamps in milliseconds

use serde_json::Value;
use crate::core::types::*;
use crate::core::types::{ExchangeResult, ExchangeError, CancelAllResponse, OrderResult, PublicTrade, TradeSide};

pub struct BybitParser;

impl BybitParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RESPONSE WRAPPER
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Extract result from Bybit response
    ///
    /// Checks retCode == 0 for success
    fn extract_result(json: &Value) -> ExchangeResult<&Value> {
        let ret_code = json["retCode"].as_i64().unwrap_or(-1);

        if ret_code != 0 {
            let ret_msg = json["retMsg"].as_str().unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: ret_code as i32,
                message: ret_msg.to_string(),
            });
        }

        Ok(&json["result"])
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse ticker from REST response
    ///
    /// Endpoint: GET /v5/market/tickers
    /// Response: result.list[0] = { symbol, lastPrice, bid1Price, ask1Price, ... }
    pub fn parse_ticker(json: &Value) -> ExchangeResult<Ticker> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let data = list.first()
            .ok_or_else(|| ExchangeError::Parse("Empty result.list".into()))?;

        let _symbol = data["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?;

        let last_price = data["lastPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid lastPrice".into()))?;

        let bid_price = data["bid1Price"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let ask_price = data["ask1Price"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let high_24h = data["highPrice24h"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let low_24h = data["lowPrice24h"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let volume_24h = data["volume24h"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let quote_volume_24h = data["turnover24h"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let timestamp = json["time"].as_i64().unwrap_or(0);

        // Helper: parse non-empty string → f64, empty string → None
        let parse_ne = |key: &str| -> Option<f64> {
            data[key].as_str()
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<f64>().ok())
        };
        let parse_ne_i64 = |key: &str| -> Option<i64> {
            data[key].as_str()
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<i64>().ok())
        };

        Ok(Ticker {
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h,
            price_change_24h: {
                let last = data["lastPrice"].as_str().and_then(|s| s.parse::<f64>().ok());
                let prev = data["prevPrice24h"].as_str().and_then(|s| s.parse::<f64>().ok());
                match (last, prev) {
                    (Some(l), Some(p)) => Some(l - p),
                    _ => None,
                }
            },
            price_change_percent_24h: data["price24hPcnt"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v * 100.0),
            timestamp,
            // ── Top-of-book sizes ──
            bid_qty: parse_ne("bid1Size"),
            ask_qty: parse_ne("ask1Size"),
            // ── Extra price stats ──
            prev_price_24h: parse_ne("prevPrice24h"),
            prev_price_1h: parse_ne("prevPrice1h"),
            turnover_24h: parse_ne("turnover24h"),
            // ── Derivatives fields ──
            mark_price: parse_ne("markPrice"),
            index_price: parse_ne("indexPrice"),
            open_interest: parse_ne("openInterest"),
            open_interest_value: parse_ne("openInterestValue"),
            single_open_interest: parse_ne("singleOpenInterest"),
            funding_rate: parse_ne("fundingRate"),
            next_funding_time: parse_ne_i64("nextFundingTime"),
            funding_interval_hour: parse_ne("fundingIntervalHour"),
            funding_cap: parse_ne("fundingCap"),
            // basis: "" on spot and perps without calendar spread → None
            basis: parse_ne("basis"),
            // basisRate or basisRateYear: use basisRate first, fallback to basisRateYear
            basis_rate: parse_ne("basisRate").or_else(|| parse_ne("basisRateYear")),
            predicted_delivery_price: parse_ne("predictedDeliveryPrice"),
            // deliveryTime: "0" is a valid timestamp (no delivery) — keep as Some(0)
            delivery_time: data["deliveryTime"].as_str()
                .and_then(|s| s.parse::<i64>().ok()),
            ..Default::default()
        })
    }

    /// Parse orderbook from REST response
    ///
    /// Endpoint: GET /v5/market/orderbook
    /// Response: result = { s, b: [[price, size]], a: [[price, size]], ts, u }
    pub fn parse_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        let result = Self::extract_result(json)?;

        let bids: Vec<OrderBookLevel> = result["b"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing bids".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some(OrderBookLevel::new(price, size))
            })
            .collect();

        let asks: Vec<OrderBookLevel> = result["a"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing asks".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some(OrderBookLevel::new(price, size))
            })
            .collect();

        let timestamp = result["ts"].as_i64().unwrap_or(0);
        let last_update_id = result["u"].as_i64().map(|u| u as u64);
        let sequence = last_update_id.map(|u| u.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
            last_update_id,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
            ..Default::default()
        })
    }

    /// Parse klines from REST response
    ///
    /// Endpoint: GET /v5/market/kline
    /// Response: result.list = [[time, open, high, low, close, volume, turnover], ...]
    ///
    /// CRITICAL: Array order is [time, open, high, low, close, volume, turnover]
    /// This differs from KuCoin: [time, open, close, high, low, volume, turnover]
    /// HIGH and CLOSE positions are SWAPPED!
    pub fn parse_klines(json: &Value) -> ExchangeResult<Vec<Kline>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let mut klines: Vec<Kline> = list.iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;

                // Bybit order: [time, open, high, low, close, volume, turnover]
                let open_time = arr.first()?.as_str()?.parse::<i64>().ok()?;
                let open = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                let high = arr.get(2)?.as_str()?.parse::<f64>().ok()?;
                let low = arr.get(3)?.as_str()?.parse::<f64>().ok()?;
                let close = arr.get(4)?.as_str()?.parse::<f64>().ok()?;
                // mark/index/premium-index klines return only [t,o,h,l,c] (no volume/turnover);
                // regular klines add volume at idx 5. Default missing volume to 0.0 instead of
                // dropping the row via `?`.
                let volume = arr.get(5).and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let quote_volume = arr.get(6).and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());

                Some(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume,
                    close_time: None,
                    trades: None,
                    ..Default::default()
                })
            })
            .collect();

        // Bybit returns newest first, reverse to oldest first
        klines.reverse();

        Ok(klines)
    }

    /// Parse recent public trades from REST response.
    ///
    /// Endpoint: GET /v5/market/recent-trade
    /// Response: result.list = [{ execId, symbol, price, size, side("Buy"/"Sell"), time(string ms), isBlockTrade }]
    pub fn parse_recent_trades(json: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let trades = list.iter()
            .filter_map(|item| {
                let id = item["execId"].as_str()?.to_string();
                let price = item["price"].as_str()?.parse::<f64>().ok()?;
                let quantity = item["size"].as_str()?.parse::<f64>().ok()?;
                let side = match item["side"].as_str()? {
                    "Buy" => TradeSide::Buy,
                    _ => TradeSide::Sell,
                };
                let timestamp = item["time"].as_str()?.parse::<i64>().ok()?;
                let is_block_trade = item["isBlockTrade"].as_bool();
                let is_rpi_trade = item["isRPITrade"].as_bool();
                let seq = item["seq"].as_str().and_then(|s| s.parse::<i64>().ok());
                Some(PublicTrade { id, price, quantity, side, timestamp, is_block_trade, is_rpi_trade, seq, ..Default::default() })
            })
            .collect();

        Ok(trades)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse balance from REST response
    ///
    /// Endpoint: GET /v5/account/wallet-balance
    /// Response: result.list[0].coin = [{ coin, walletBalance, locked, ... }]
    pub fn parse_balance(json: &Value) -> ExchangeResult<Vec<crate::core::Balance>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let account = list.first()
            .ok_or_else(|| ExchangeError::Parse("Empty result.list".into()))?;

        let coins = account["coin"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing coin array".into()))?;

        let balances = coins.iter()
            .filter_map(|coin_data| {
                let asset = coin_data["coin"].as_str()?.to_string();
                let free = coin_data["walletBalance"].as_str()?.parse::<f64>().ok()?;
                let locked = coin_data["locked"].as_str()?.parse::<f64>().ok().unwrap_or(0.0);
                let total = free + locked;

                Some(crate::core::Balance {
                    asset,
                    free,
                    locked,
                    total,
                })
            })
            .collect();

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRADING PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse order from REST response
    ///
    /// Endpoint: POST /v5/order/create OR GET /v5/order/realtime
    /// Response: result = { orderId, symbol, side, orderType, ... } OR result.list[0]
    pub fn parse_order(json: &Value) -> ExchangeResult<Order> {
        let result = Self::extract_result(json)?;

        // Handle both direct result and result.list[0]
        let data = if result.is_array() || result.get("list").is_some() {
            result["list"].as_array()
                .and_then(|list| list.first())
                .ok_or_else(|| ExchangeError::Parse("Empty order list".into()))?
        } else {
            result
        };

        let id = data["orderId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
            .to_string();

        let symbol = data["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?
            .to_string();

        let side = match data["side"].as_str() {
            Some("Buy") => OrderSide::Buy,
            Some("Sell") => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse("Invalid side".into())),
        };

        let order_type = match data["orderType"].as_str() {
            Some("Market") => OrderType::Market,
            Some("Limit") => OrderType::Limit { price: 0.0 },
            _ => OrderType::Limit { price: 0.0 }, // default
        };

        let status = Self::parse_order_status(data["orderStatus"].as_str().unwrap_or(""));

        let price = data["price"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let quantity = data["qty"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let filled_quantity = data["cumExecQty"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let average_price = data["avgPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let created_at = data["createdTime"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let updated_at = data["updatedTime"].as_str()
            .and_then(|s| s.parse::<i64>().ok());

        Ok(Order {
            id,
            client_order_id: data["orderLinkId"].as_str().map(String::from),
            symbol: Some(symbol),
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity,
            filled_quantity,
            average_price,
            time_in_force: TimeInForce::Gtc,
            commission: None,
            commission_asset: None,
            created_at,
            updated_at,
        })
    }

    /// Parse order status from string
    fn parse_order_status(status: &str) -> OrderStatus {
        match status {
            "Created" | "New" => OrderStatus::New,
            "PartiallyFilled" => OrderStatus::PartiallyFilled,
            "Filled" => OrderStatus::Filled,
            "Cancelled" => OrderStatus::Canceled,
            "Rejected" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        }
    }

    /// Parse funding rate
    ///
    /// Endpoint: GET /v5/market/funding/history
    /// Response: result.list[0] = { symbol, fundingRate, fundingRateTimestamp }
    pub fn parse_funding_rate(json: &Value) -> ExchangeResult<FundingRate> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let data = list.first()
            .ok_or_else(|| ExchangeError::Parse("Empty result.list".into()))?;

        let rate = data["fundingRate"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let timestamp = data["fundingRateTimestamp"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let symbol = data["symbol"].as_str()
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(FundingRate {
            rate,
            next_funding_time: None,
            timestamp,
            symbol,
            ..Default::default()
        })
    }

    /// Parse an array of funding rate records (`/v5/market/funding/history`).
    ///
    /// Response: `result.list = [{ symbol, fundingRate, fundingRateTimestamp }]`.
    pub fn parse_funding_rates(json: &Value) -> ExchangeResult<Vec<FundingRate>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;
        let rates = list.iter().map(|item| FundingRate {
            rate: item["fundingRate"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0),
            next_funding_time: None,
            timestamp: item["fundingRateTimestamp"].as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
            symbol: item["symbol"].as_str().filter(|s| !s.is_empty()).map(String::from),
            ..Default::default()
        }).collect();
        Ok(rates)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse exchange info (symbol list) from Bybit response
    ///
    /// Endpoint: GET /v5/market/instruments-info
    /// Response: result.list = [{ symbol, baseCoin, quoteCoin, status, lotSizeFilter, priceFilter }]
    ///
    /// Filters to active/trading symbols only (status == "Trading").
    pub fn parse_exchange_info(json: &Value, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let symbols = list.iter()
            .filter_map(|item| {
                let symbol = item["symbol"].as_str()?.to_string();
                let base_asset = item["baseCoin"].as_str().unwrap_or("").to_string();
                let quote_asset = item["quoteCoin"].as_str().unwrap_or("").to_string();
                // RAW: keep native status verbatim, no filter
                let status = item["status"].as_str().unwrap_or("").to_string();

                // Parse lot size filter
                let lot_filter = item.get("lotSizeFilter");
                let min_quantity = lot_filter
                    .and_then(|f| f["minOrderQty"].as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let max_quantity = lot_filter
                    .and_then(|f| f["maxOrderQty"].as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let step_size = lot_filter
                    .and_then(|f| f["qtyStep"].as_str())
                    .and_then(|s| s.parse::<f64>().ok());

                // Parse price filter for precision
                let price_filter = item.get("priceFilter");
                let tick_size = price_filter
                    .and_then(|f| f["tickSize"].as_str())
                    .and_then(|s| s.parse::<f64>().ok());

                // Derive price precision from tick size (e.g. "0.01" -> 2)
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

                // Derive quantity precision from step size
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

                // RAW native contract type (e.g. "LinearPerpetual", "InversePerpetual", absent on spot)
                let instrument_type = item["contractType"].as_str().map(|v| v.to_string());

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
                    instrument_type,
                    extra: item.clone(),
                })
            })
            .collect();

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // OPTIONAL TRAIT PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse cancel-all response.
    ///
    /// Bybit returns: `result.list = [{ orderId, orderLinkId }, ...]`
    /// Each item is a successfully cancelled order.
    pub fn parse_cancel_all_response(json: &Value) -> ExchangeResult<CancelAllResponse> {
        let result = Self::extract_result(json)?;

        let list = result["list"].as_array()
            .cloned()
            .unwrap_or_default();

        let details: Vec<OrderResult> = list.iter()
            .map(|item| OrderResult {
                order: None,
                client_order_id: item["orderLinkId"].as_str().map(String::from),
                success: true,
                error: None,
                error_code: None,
            })
            .collect();

        let cancelled_count = details.len() as u32;

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count: 0,
            details,
        })
    }

    /// Parse amend order response.
    ///
    /// Bybit returns: `result = { orderId, orderLinkId }`
    /// The full updated order is not returned — re-use `parse_order` to wrap it.
    pub fn parse_amend_order_response(json: &Value) -> ExchangeResult<Order> {
        let result = Self::extract_result(json)?;

        // Bybit amend returns minimal info: orderId + orderLinkId
        // Build a minimal Order object from the available data.
        let id = result["orderId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing orderId in amend response".to_string()))?
            .to_string();

        Ok(Order {
            id,
            client_order_id: result["orderLinkId"].as_str().map(String::from),
            symbol: None,
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

    /// Parse batch orders response.
    ///
    /// Bybit batch response: `result.list = [{ orderId, orderLinkId, code, msg }, ...]`
    /// `code == "0"` indicates success for each item.
    pub fn parse_batch_orders_response(json: &Value) -> ExchangeResult<Vec<OrderResult>> {
        let result = Self::extract_result(json)?;

        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list in batch response".to_string()))?;

        let results = list.iter()
            .map(|item| {
                let code = item["code"].as_str().unwrap_or("0");
                let success = code == "0";
                if success {
                    OrderResult {
                        order: None,
                        client_order_id: item["orderLinkId"].as_str().map(String::from),
                        success: true,
                        error: None,
                        error_code: None,
                    }
                } else {
                    let msg = item["msg"].as_str().unwrap_or("Unknown error").to_string();
                    OrderResult {
                        order: None,
                        client_order_id: item["orderLinkId"].as_str().map(String::from),
                        success: false,
                        error: Some(msg),
                        error_code: code.parse().ok(),
                    }
                }
            })
            .collect();

        Ok(results)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT TRANSFERS PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse inter-transfer response.
    ///
    /// Bybit returns: `result = { transferId }`
    pub fn parse_transfer_response(json: &Value) -> ExchangeResult<TransferResponse> {
        let result = Self::extract_result(json)?;

        let transfer_id = result["transferId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing transferId".to_string()))?
            .to_string();

        Ok(TransferResponse {
            transfer_id,
            status: "SUCCESS".to_string(),
            asset: String::new(),
            amount: 0.0,
            timestamp: None,
        })
    }

    /// Parse transfer history response.
    ///
    /// Bybit returns: `result.list = [{ transferId, coin, amount, fromAccountType, toAccountType, status, timestamp }]`
    pub fn parse_transfer_history(json: &Value) -> ExchangeResult<Vec<TransferResponse>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .cloned()
            .unwrap_or_default();

        let records = list.iter()
            .map(|item| {
                let transfer_id = item["transferId"].as_str()
                    .unwrap_or("")
                    .to_string();
                let asset = item["coin"].as_str()
                    .unwrap_or("")
                    .to_string();
                let amount = item["amount"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let status = item["status"].as_str()
                    .unwrap_or("UNKNOWN")
                    .to_string();
                let timestamp = item["timestamp"].as_str()
                    .and_then(|s| s.parse::<i64>().ok());

                TransferResponse {
                    transfer_id,
                    status,
                    asset,
                    amount,
                    timestamp,
                }
            })
            .collect();

        Ok(records)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // CUSTODIAL FUNDS PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse deposit address response.
    ///
    /// Bybit returns: `result = { coin, chains: [{ chainType, addressDeposit, tagDeposit, ... }] }`
    pub fn parse_deposit_address(json: &Value, asset: &str, network: Option<&str>) -> ExchangeResult<DepositAddress> {
        let result = Self::extract_result(json)?;

        let coin = result["coin"].as_str().unwrap_or(asset);

        let chains = result["chains"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing chains array in deposit address".to_string()))?;

        // Pick the chain matching `network`, or the first one if network is None.
        let chain_data = if let Some(net) = network {
            chains.iter()
                .find(|c| {
                    c["chainType"].as_str().map(|s| s.eq_ignore_ascii_case(net)).unwrap_or(false)
                })
                .ok_or_else(|| ExchangeError::Parse(
                    format!("Network '{}' not found in deposit address chains", net)
                ))?
        } else {
            chains.first()
                .ok_or_else(|| ExchangeError::Parse("No chains in deposit address response".to_string()))?
        };

        let address = chain_data["addressDeposit"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing addressDeposit".to_string()))?
            .to_string();

        let tag = chain_data["tagDeposit"].as_str()
            .filter(|s| !s.is_empty())
            .map(String::from);

        let chain_type = chain_data["chainType"].as_str().map(String::from);

        Ok(DepositAddress {
            address,
            tag,
            network: chain_type,
            asset: coin.to_string(),
            created_at: None,
        })
    }

    /// Parse withdrawal response.
    ///
    /// Bybit returns: `result = { id }`
    pub fn parse_withdraw_response(json: &Value) -> ExchangeResult<WithdrawResponse> {
        let result = Self::extract_result(json)?;

        let withdraw_id = result["id"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing withdrawal id".to_string()))?
            .to_string();

        Ok(WithdrawResponse {
            withdraw_id,
            status: "PENDING".to_string(),
            tx_hash: None,
        })
    }

    /// Parse deposit history response.
    ///
    /// Bybit returns: `result.rows = [{ txID, coin, amount, chain, status, successAt }]`
    pub fn parse_deposit_history(json: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        let result = Self::extract_result(json)?;

        // deposit history uses "rows" not "list"
        let rows = result["rows"].as_array()
            .cloned()
            .unwrap_or_default();

        let records = rows.iter()
            .map(|item| {
                let id = item["txID"].as_str().unwrap_or("").to_string();
                let asset = item["coin"].as_str().unwrap_or("").to_string();
                let amount = item["amount"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let tx_hash = item["txID"].as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                let network = item["chain"].as_str().map(String::from);
                let status = item["status"].as_str().unwrap_or("UNKNOWN").to_string();
                let timestamp = item["successAt"].as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);

                FundsRecord::Deposit {
                    id,
                    asset,
                    amount,
                    tx_hash,
                    network,
                    status,
                    timestamp,
                }
            })
            .collect();

        Ok(records)
    }

    /// Parse withdrawal history response.
    ///
    /// Bybit returns: `result.rows = [{ withdrawId, coin, amount, chain, address, tag, txID, status, createTime }]`
    pub fn parse_withdrawal_history(json: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        let result = Self::extract_result(json)?;

        let rows = result["rows"].as_array()
            .cloned()
            .unwrap_or_default();

        let records = rows.iter()
            .map(|item| {
                let id = item["withdrawId"].as_str().unwrap_or("").to_string();
                let asset = item["coin"].as_str().unwrap_or("").to_string();
                let amount = item["amount"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let fee = item["withdrawFee"].as_str()
                    .and_then(|s| s.parse::<f64>().ok());
                let address = item["address"].as_str().unwrap_or("").to_string();
                let tag = item["tag"].as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                let tx_hash = item["txID"].as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                let network = item["chain"].as_str().map(String::from);
                let status = item["status"].as_str().unwrap_or("UNKNOWN").to_string();
                let timestamp = item["createTime"].as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);

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
            })
            .collect();

        Ok(records)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SUB-ACCOUNT PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse create sub-account response.
    ///
    /// Bybit returns: `result = { uid, username, memberType, status, remark }`
    pub fn parse_create_sub_member(json: &Value) -> ExchangeResult<SubAccountResult> {
        let result = Self::extract_result(json)?;

        let id = result["uid"].as_u64()
            .map(|u| u.to_string())
            .or_else(|| result["uid"].as_str().map(String::from));

        let name = result["username"].as_str().map(String::from);

        Ok(SubAccountResult {
            id,
            name,
            accounts: vec![],
            transaction_id: None,
        })
    }

    /// Parse list sub-members response.
    ///
    /// Bybit returns: `result.subMembers = [{ uid, username, memberType, status, remark }]`
    pub fn parse_list_sub_members(json: &Value) -> ExchangeResult<SubAccountResult> {
        let result = Self::extract_result(json)?;

        let sub_members = result["subMembers"].as_array()
            .cloned()
            .unwrap_or_default();

        let accounts: Vec<SubAccount> = sub_members.iter()
            .map(|item| {
                let id = item["uid"].as_u64()
                    .map(|u| u.to_string())
                    .or_else(|| item["uid"].as_str().map(String::from))
                    .unwrap_or_default();
                let name = item["username"].as_str().unwrap_or("").to_string();
                let status = match item["status"].as_u64() {
                    Some(1) => "Normal",
                    Some(2) => "Login Banned",
                    Some(4) => "Frozen",
                    _ => "Unknown",
                }.to_string();

                SubAccount { id, name, status }
            })
            .collect();

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts,
            transaction_id: None,
        })
    }

    /// Parse universal transfer response.
    ///
    /// Bybit returns: `result = { transferId }`
    pub fn parse_universal_transfer(json: &Value) -> ExchangeResult<SubAccountResult> {
        let result = Self::extract_result(json)?;

        let transfer_id = result["transferId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing transferId in universal transfer response".to_string()))?
            .to_string();

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts: vec![],
            transaction_id: Some(transfer_id),
        })
    }

    /// Parse sub-account balance response.
    ///
    /// Bybit returns: `result = { memberId, accountType, balance: [{ coin, walletBalance, ... }] }`
    /// The balance list is returned as `transaction_id` is not applicable here.
    /// We store the coin balances as a JSON summary in `transaction_id`.
    pub fn parse_sub_account_balance(json: &Value) -> ExchangeResult<SubAccountResult> {
        let result = Self::extract_result(json)?;

        let member_id = result["memberId"].as_str()
            .or_else(|| result["memberId"].as_u64().map(|_| "").filter(|_| false))
            .map(String::from);

        // Summarize balance as "COIN:amount,COIN:amount,..."
        let balance_summary = result["balance"].as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let coin = item["coin"].as_str()?;
                        let amount = item["walletBalance"].as_str().unwrap_or("0");
                        Some(format!("{}:{}", coin, amount))
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            });

        Ok(SubAccountResult {
            id: member_id,
            name: None,
            accounts: vec![],
            transaction_id: balance_summary,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // USER TRADE PARSER
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse a single execution (fill) from `result.list[]` of GET /v5/execution/list
    ///
    /// Each list item:
    /// ```json
    /// {
    ///   "execId": "...",
    ///   "orderId": "...",
    ///   "symbol": "BTCUSDT",
    ///   "side": "Buy",
    ///   "execPrice": "50000",
    ///   "execQty": "0.001",
    ///   "execFee": "0.00001",
    ///   "feeCurrency": "USDT",
    ///   "isMaker": false,
    ///   "execTime": "1672531200000"
    /// }
    /// ```
    pub fn parse_user_trade(item: &Value) -> ExchangeResult<UserTrade> {
        let id = item["execId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing execId".to_string()))?
            .to_string();

        let order_id = item["orderId"].as_str()
            .unwrap_or("")
            .to_string();

        let symbol = item["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?
            .to_string();

        let side = match item["side"].as_str().unwrap_or("Buy") {
            "Sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let price = item["execPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let quantity = item["execQty"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let commission = item["execFee"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let commission_asset = item["feeCurrency"].as_str()
            .unwrap_or("")
            .to_string();

        let is_maker = item["isMaker"].as_bool().unwrap_or(false);

        // execTime is a string in milliseconds
        let timestamp = item["execTime"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
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
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket message
    ///
    /// Format: { "topic": "...", "type": "snapshot|delta", "ts": ..., "data": {...} }
    pub fn parse_ws_message(json: &Value) -> ExchangeResult<(String, String, &Value)> {
        let topic = json["topic"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing topic".into()))?;

        let msg_type = json["type"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing type".into()))?;

        let data = &json["data"];

        Ok((topic.to_string(), msg_type.to_string(), data))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding payments from `GET /v5/account/transaction-log?type=SETTLEMENT`
    pub fn parse_funding_payments(response: &Value) -> ExchangeResult<Vec<FundingPayment>> {
        let result = Self::extract_result(response)?;
        let list = result.get("list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'list' in transaction-log result".to_string()))?;

        let mut payments = Vec::with_capacity(list.len());
        for item in list {
            let symbol = item.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let payment: f64 = item.get("cashFlow")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let position_size: f64 = item.get("qty")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let asset = item.get("currency").and_then(|v| v.as_str()).unwrap_or("USDT").to_string();
            let timestamp: i64 = item.get("transactionTime")
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
    // MARKET DATA EXTENSIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse open interest list from `GET /v5/market/open-interest`.
    ///
    /// Response: `result.list = [{ openInterest, openInterestValue, timestamp }, ...]`
    pub fn parse_open_interest_list(json: &Value) -> ExchangeResult<Vec<crate::core::types::OpenInterest>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list in open-interest".to_string()))?;

        let records = list.iter()
            .map(|item| {
                let open_interest = item["openInterest"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let open_interest_value = item["openInterestValue"].as_str()
                    .and_then(|s| s.parse::<f64>().ok());
                let timestamp = item["timestamp"].as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                let single_open_interest = item["singleOpenInterest"].as_str()
                    .filter(|s| !s.is_empty())
                    .and_then(|s| s.parse::<f64>().ok());
                crate::core::types::OpenInterest {
                    open_interest,
                    open_interest_value,
                    timestamp,
                    single_open_interest,
                    ..Default::default()
                }
            })
            .collect();

        Ok(records)
    }

    /// Parse long/short ratio list from `GET /v5/market/account-ratio`.
    ///
    /// Response: `result.list = [{ buyRatio, sellRatio, timestamp }, ...]`
    pub fn parse_long_short_ratios(json: &Value, symbol: &str, ratio_type: &str) -> ExchangeResult<Vec<LongShortRatio>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list in account-ratio".to_string()))?;

        let records = list.iter()
            .map(|item| {
                let long_ratio = item["buyRatio"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let short_ratio = item["sellRatio"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let ratio = if short_ratio != 0.0 { Some(long_ratio / short_ratio) } else { None };
                let timestamp = item["timestamp"].as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                LongShortRatio {
                    symbol: symbol.to_string(),
                    ratio_type: ratio_type.to_string(),
                    long_ratio,
                    short_ratio,
                    ratio,
                    timestamp, ..Default::default() 
                }
            })
            .collect();

        Ok(records)
    }

    /// Parse mark/index/premium kline list.
    ///
    /// All three endpoints share the same response list format as `/v5/market/kline`:
    /// `result.list = [[timestamp, open, high, low, close, volume, turnover], ...]`
    pub fn parse_mark_price_kline(json: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::parse_klines(json)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT LEDGER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ledger from `GET /v5/account/transaction-log` (all types).
    ///
    /// Maps Bybit `type` field to `LedgerEntryType`.
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        let result = Self::extract_result(response)?;
        let list = result.get("list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'list' in transaction-log result".to_string()))?;

        let mut entries = Vec::with_capacity(list.len());
        for item in list {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let symbol = item.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tx_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("OTHER");
            let amount: f64 = item.get("cashFlow")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let balance: Option<f64> = item.get("cashBalance")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
            let asset = item.get("currency").and_then(|v| v.as_str()).unwrap_or("USDT").to_string();
            let timestamp: i64 = item.get("transactionTime")
                .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
            let entry_type = match tx_type {
                "TRADE" => LedgerEntryType::Trade,
                "SETTLEMENT" => LedgerEntryType::Funding,
                "DELIVERY" => LedgerEntryType::Settlement,
                "TRANSFER" | "AIRDROP" => LedgerEntryType::Transfer,
                "CASHBACK" | "REBATE" => LedgerEntryType::Rebate,
                "LIQUIDATION" => LedgerEntryType::Liquidation,
                "DEPOSIT" => LedgerEntryType::Deposit,
                "WITHDRAWAL" => LedgerEntryType::Withdrawal,
                other => LedgerEntryType::Other(other.to_string()),
            };
            entries.push(LedgerEntry {
                id,
                asset,
                amount,
                balance,
                entry_type,
                description: format!("{} {}", tx_type, symbol),
                ref_id: None,
                timestamp,
            });
        }
        Ok(entries)
    }
}

// Balance type conversion helper - removed, use core Balance directly

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_funding_rates() {
        let json = json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "category": "linear",
                "list": [
                    {"symbol":"BTCUSDT","fundingRate":"0.00010000","fundingRateTimestamp":"1672531200000"},
                    {"symbol":"BTCUSDT","fundingRate":"-0.00005000","fundingRateTimestamp":"1672560000000"}
                ]
            }
        });

        let rates = BybitParser::parse_funding_rates(&json).unwrap();
        assert_eq!(rates.len(), 2);
        assert!((rates[0].rate - 0.0001).abs() < 1e-9);
        assert_eq!(rates[0].timestamp, 1672531200000);
        assert!((rates[1].rate - (-0.00005)).abs() < 1e-9);
        assert_eq!(rates[1].timestamp, 1672560000000);
    }

    #[test]
    fn test_parse_ticker() {
        let json = json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "list": [{
                    "symbol": "BTCUSDT",
                    "lastPrice": "40000.00",
                    "bid1Price": "39999.00",
                    "ask1Price": "40001.00",
                    "highPrice24h": "41000.00",
                    "lowPrice24h": "39000.00",
                    "volume24h": "1234.56",
                    "turnover24h": "49382000.00"
                }]
            },
            "time": 1702617474601i64
        });

        let ticker = BybitParser::parse_ticker(&json).unwrap();
        assert_eq!(ticker.last_price, 40000.0);
        assert_eq!(ticker.bid_price, Some(39999.0));
        assert_eq!(ticker.ask_price, Some(40001.0));
    }

    #[test]
    fn test_parse_recent_trades() {
        // Live payload shape (probed 2026-06-14): spot category
        let json = json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "category": "spot",
                "list": [
                    {
                        "execId": "abc123",
                        "symbol": "BTCUSDT",
                        "price": "64126.7",
                        "size": "0.002701",
                        "side": "Sell",
                        "time": "1781450145888",
                        "isBlockTrade": false,
                        "seq": "0"
                    },
                    {
                        "execId": "def456",
                        "symbol": "BTCUSDT",
                        "price": "64130.0",
                        "size": "0.010000",
                        "side": "Buy",
                        "time": "1781450145900",
                        "isBlockTrade": false,
                        "seq": "1"
                    }
                ]
            }
        });

        let trades = BybitParser::parse_recent_trades(&json).unwrap();
        assert_eq!(trades.len(), 2);

        assert_eq!(trades[0].id, "abc123");
        assert!((trades[0].price - 64126.7).abs() < 1e-6);
        assert!((trades[0].quantity - 0.002701).abs() < 1e-8);
        assert_eq!(trades[0].side, TradeSide::Sell);
        assert_eq!(trades[0].timestamp, 1781450145888);

        assert_eq!(trades[1].id, "def456");
        assert_eq!(trades[1].side, TradeSide::Buy);
        assert_eq!(trades[1].timestamp, 1781450145900);
    }

    #[test]
    fn test_parse_klines() {
        let json = json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "list": [
                    ["1670608800000", "40000.00", "40500.00", "39900.00", "40200.00", "123.456", "4960000.00"]
                ]
            },
            "time": 1702617474601i64
        });

        let klines = BybitParser::parse_klines(&json).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open_time, 1670608800000);
        assert_eq!(klines[0].open, 40000.0);
        assert_eq!(klines[0].high, 40500.0);
        assert_eq!(klines[0].low, 39900.0);
        assert_eq!(klines[0].close, 40200.0);
    }
}
