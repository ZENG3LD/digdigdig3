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
use crate::core::types::{ExchangeResult, ExchangeError, CancelAllResponse, OrderResult};

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

        let symbol = data["symbol"].as_str()
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

        Ok(Ticker {
            symbol: symbol.to_string(),
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
        })
    }

    /// Parse orderbook from REST response
    ///
    /// Endpoint: GET /v5/market/orderbook
    /// Response: result = { s, b: [[price, size]], a: [[price, size]], ts, u }
    pub fn parse_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        let result = Self::extract_result(json)?;

        let bids = result["b"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing bids".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let asks = result["a"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing asks".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let timestamp = result["ts"].as_i64().unwrap_or(0);
        let sequence = result["u"].as_i64().map(|u| u.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
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
                let volume = arr.get(5)?.as_str()?.parse::<f64>().ok()?;
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
                })
            })
            .collect();

        // Bybit returns newest first, reverse to oldest first
        klines.reverse();

        Ok(klines)
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
            symbol,
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

        let symbol = data["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?;

        let rate = data["fundingRate"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let timestamp = data["fundingRateTimestamp"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        Ok(FundingRate {
            symbol: symbol.to_string(),
            rate,
            next_funding_time: None,
            timestamp,
        })
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
    pub fn parse_exchange_info(json: &Value) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let result = Self::extract_result(json)?;
        let list = result["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing result.list".into()))?;

        let symbols = list.iter()
            .filter_map(|item| {
                let symbol = item["symbol"].as_str()?.to_string();
                let base_asset = item["baseCoin"].as_str().unwrap_or("").to_string();
                let quote_asset = item["quoteCoin"].as_str().unwrap_or("").to_string();
                let status = item["status"].as_str().unwrap_or("").to_string();

                // Filter to active symbols only
                if status != "Trading" {
                    return None;
                }

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

                Some(crate::core::types::SymbolInfo {
                    symbol,
                    base_asset,
                    quote_asset,
                    status,
                    price_precision,
                    quantity_precision,
                    min_quantity,
                    max_quantity,
                    step_size,
                    min_notional: None,
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
}

// Balance type conversion helper - removed, use core Balance directly

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.last_price, 40000.0);
        assert_eq!(ticker.bid_price, Some(39999.0));
        assert_eq!(ticker.ask_price, Some(40001.0));
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
