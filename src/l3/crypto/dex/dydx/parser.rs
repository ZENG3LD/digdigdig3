//! # dYdX v4 Response Parser
//!
//! Парсинг JSON ответов от dYdX v4 Indexer API.

use serde_json::Value;
use chrono::DateTime;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, OrderBookLevel, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, StreamEvent, TradeSide,
    OrderbookDelta as OrderbookDeltaData,
    UserTrade, FundingPayment,
};

/// Парсер ответов dYdX v4 Indexer API
pub struct DydxParser;

impl DydxParser {
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

    /// Парсить ISO 8601 timestamp в Unix ms
    fn parse_iso_timestamp(iso: &str) -> Option<i64> {
        DateTime::parse_from_rfc3339(iso)
            .ok()
            .map(|dt| dt.timestamp_millis())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить price из ticker
    pub fn parse_price(response: &Value, symbol: &str) -> ExchangeResult<f64> {
        // dYdX doesn't have a single "price" endpoint
        // Use oraclePrice from markets response
        let markets = response.get("markets")
            .ok_or_else(|| ExchangeError::Parse("Missing 'markets' field".to_string()))?;

        let market = markets.get(symbol)
            .ok_or_else(|| ExchangeError::Parse(format!("Market '{}' not found", symbol)))?;

        Self::require_f64(market, "oraclePrice")
    }

    /// Парсить klines (candles)
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let candles = response.get("candles")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'candles' array".to_string()))?;

        let mut klines = Vec::with_capacity(candles.len());

        for candle in candles {
            let started_at = Self::get_str(candle, "startedAt")
                .and_then(Self::parse_iso_timestamp)
                .unwrap_or(0);

            klines.push(Kline {
                open_time: started_at,
                open: Self::get_f64(candle, "open").unwrap_or(0.0),
                close: Self::get_f64(candle, "close").unwrap_or(0.0),
                high: Self::get_f64(candle, "high").unwrap_or(0.0),
                low: Self::get_f64(candle, "low").unwrap_or(0.0),
                volume: Self::get_f64(candle, "baseTokenVolume").unwrap_or(0.0),
                quote_volume: Self::get_f64(candle, "usdVolume"),
                close_time: None,
                trades: candle.get("trades").and_then(|t| t.as_i64()).map(|t| t as u64),
            });
        }

        // dYdX returns candles newest-first; reverse to oldest-first for chart display
        klines.reverse();

        Ok(klines)
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            response.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let obj = level.as_object()?;
                            let price = obj.get("price").and_then(Self::parse_f64)?;
                            let size = obj.get("size").and_then(Self::parse_f64)?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: chrono::Utc::now().timestamp_millis(),
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

    /// Парсить ticker из markets response
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let markets = response.get("markets")
            .ok_or_else(|| ExchangeError::Parse("Missing 'markets' field".to_string()))?;

        let market = markets.get(symbol)
            .ok_or_else(|| ExchangeError::Parse(format!("Market '{}' not found", symbol)))?;

        Ok(Ticker {
            symbol: Self::get_str(market, "ticker").unwrap_or(symbol).to_string(),
            last_price: Self::get_f64(market, "oraclePrice").unwrap_or(0.0),
            bid_price: None, // Not provided in markets endpoint
            ask_price: None, // Not provided in markets endpoint
            high_24h: None,
            low_24h: None,
            volume_24h: Self::get_f64(market, "volume24H"),
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(market, "priceChange24H"),
            price_change_percent_24h: Self::get_f64(market, "priceChange24H")
                .and_then(|change| {
                    Self::get_f64(market, "oraclePrice")
                        .map(|price| (change / (price - change)) * 100.0)
                }),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let funding = response.get("historicalFunding")
            .and_then(|arr| arr.as_array()?.first())
            .ok_or_else(|| ExchangeError::Parse("Missing funding data".to_string()))?;

        let effective_at = Self::get_str(funding, "effectiveAt")
            .and_then(Self::parse_iso_timestamp)
            .unwrap_or(0);

        Ok(FundingRate {
            symbol: Self::get_str(funding, "ticker").unwrap_or("").to_string(),
            rate: Self::require_f64(funding, "rate")?,
            next_funding_time: None,
            timestamp: effective_at,
        })
    }

    /// Парсить public trades
    pub fn parse_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        let trades = response.get("trades")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'trades' array".to_string()))?;

        trades.iter()
            .map(|trade| {
                let side = match Self::get_str(trade, "side").unwrap_or("BUY") {
                    "SELL" => TradeSide::Sell,
                    _ => TradeSide::Buy,
                };

                let created_at = Self::get_str(trade, "createdAt")
                    .and_then(Self::parse_iso_timestamp)
                    .unwrap_or(0);

                Ok(PublicTrade {
                    id: Self::get_str(trade, "id").unwrap_or("").to_string(),
                    symbol: "".to_string(), // Market symbol not included in trade data
                    price: Self::require_f64(trade, "price")?,
                    quantity: Self::get_f64(trade, "size").unwrap_or(0.0),
                    side,
                    timestamp: created_at,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("BUY") {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("LIMIT") {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match Self::get_str(data, "status").unwrap_or("OPEN") {
            "FILLED" => OrderStatus::Filled,
            "CANCELED" | "BEST_EFFORT_CANCELED" => OrderStatus::Canceled,
            "OPEN" => {
                let total_filled = Self::get_f64(data, "totalFilled").unwrap_or(0.0);
                if total_filled > 0.0 {
                    OrderStatus::PartiallyFilled
                } else {
                    OrderStatus::New
                }
            },
            _ => OrderStatus::New,
        };

        let created_at = Self::get_str(data, "createdAt")
            .and_then(Self::parse_iso_timestamp)
            .unwrap_or(0);
        let updated_at = Self::get_str(data, "updatedAt")
            .and_then(Self::parse_iso_timestamp);

        Ok(Order {
            id: Self::get_str(data, "id").unwrap_or("").to_string(),
            client_order_id: data.get("clientId")
                .and_then(|v| v.as_u64())
                .map(|id| id.to_string()),
            symbol: Self::get_str(data, "ticker").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "triggerPrice"),
            quantity: Self::get_f64(data, "size").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "totalFilled").unwrap_or(0.0),
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at,
            updated_at,
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse fills (user trades) from a `GET /v4/fills` response.
    ///
    /// Expected response shape:
    /// ```json
    /// {"fills":[{"id":"123","orderId":"456","market":"BTC-USD","side":"BUY",
    ///   "price":"50000","size":"0.001","fee":"0.01","liquidity":"MAKER",
    ///   "createdAt":"2024-01-01T00:00:00Z"}]}
    /// ```
    /// `liquidity`: "MAKER" or "TAKER".
    pub fn parse_fills(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let fills = response.get("fills")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'fills' array".to_string()))?;

        fills.iter().map(|fill| {
            let id = Self::get_str(fill, "id").unwrap_or("").to_string();
            let order_id = Self::get_str(fill, "orderId").unwrap_or("").to_string();
            // market is "BTC-USD" — use as-is for the symbol
            let symbol = Self::get_str(fill, "market").unwrap_or("").to_string();
            let side_str = Self::get_str(fill, "side").unwrap_or("BUY");
            let side = if side_str.eq_ignore_ascii_case("SELL") {
                OrderSide::Sell
            } else {
                OrderSide::Buy
            };
            let price = Self::get_f64(fill, "price").unwrap_or(0.0);
            let quantity = Self::get_f64(fill, "size").unwrap_or(0.0);
            let commission = Self::get_f64(fill, "fee").unwrap_or(0.0).abs();
            // dYdX fees are always in USDC
            let commission_asset = "USDC".to_string();
            let liquidity = Self::get_str(fill, "liquidity").unwrap_or("TAKER");
            let is_maker = liquidity.eq_ignore_ascii_case("MAKER");
            let timestamp = Self::get_str(fill, "createdAt")
                .and_then(Self::parse_iso_timestamp)
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
        }).collect()
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let orders = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        orders.iter()
            .map(Self::parse_order)
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balances (asset positions в dYdX)
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let subaccount = response.get("subaccount")
            .ok_or_else(|| ExchangeError::Parse("Missing 'subaccount' field".to_string()))?;

        let asset_positions = subaccount.get("assetPositions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'assetPositions'".to_string()))?;

        let mut balances = Vec::new();

        for (asset, position) in asset_positions {
            let size = Self::get_f64(position, "size").unwrap_or(0.0);

            balances.push(Balance {
                asset: asset.clone(),
                free: size, // dYdX doesn't distinguish free/locked for USDC
                locked: 0.0,
                total: size,
            });
        }

        // Add equity and freeCollateral as meta information
        if let Some(equity) = Self::get_f64(subaccount, "equity") {
            if !balances.iter().any(|b| b.asset == "USDC") {
                balances.push(Balance {
                    asset: "USDC".to_string(),
                    free: Self::get_f64(subaccount, "freeCollateral").unwrap_or(equity),
                    locked: 0.0,
                    total: equity,
                });
            }
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let positions = response.get("positions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'positions' array".to_string()))?;

        Ok(positions.iter()
            .filter_map(|pos| Self::parse_position_data(pos).ok())
            .collect::<Vec<_>>())
    }

    fn parse_position_data(data: &Value) -> ExchangeResult<Position> {
        let size = Self::get_f64(data, "size").unwrap_or(0.0);

        // Skip closed positions
        if size.abs() < f64::EPSILON {
            return Err(ExchangeError::Parse("Position is closed".to_string()));
        }

        let side = match Self::get_str(data, "side").unwrap_or("LONG") {
            "SHORT" => PositionSide::Short,
            _ => PositionSide::Long,
        };

        Ok(Position {
            symbol: Self::get_str(data, "market").unwrap_or("").to_string(),
            side,
            quantity: size,
            entry_price: Self::get_f64(data, "entryPrice").unwrap_or(0.0),
            mark_price: None, // Not provided in position data
            unrealized_pnl: Self::get_f64(data, "unrealizedPnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realizedPnl"),
            leverage: 1, // dYdX uses margin fractions, not leverage
            liquidation_price: None,
            margin: None,
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker message from v4_markets channel.
    ///
    /// `target_symbol` — the dYdX market identifier we subscribed to (e.g. `"BTC-USD"`).
    /// The `v4_markets` channel is global and its snapshot contains ALL markets; we extract
    /// only the entry matching `target_symbol` instead of blindly picking the first entry.
    pub fn parse_ws_ticker(data: &Value, target_symbol: &str) -> ExchangeResult<Ticker> {
        let contents = data.get("contents")
            .ok_or_else(|| ExchangeError::Parse("Missing 'contents' field".to_string()))?;

        // Try v4_markets snapshot format first (contents.markets.{SYMBOL}),
        // then fall back to channel_data incremental format (contents IS the markets object).
        let markets = contents.get("markets")
            .and_then(|m| m.as_object())
            .or_else(|| contents.as_object())
            .ok_or_else(|| ExchangeError::Parse("'contents' has no usable market data".to_string()))?;

        // Always look up our specific symbol — ignore any `id` field and never fall back to
        // the first market.  This prevents global snapshots from emitting a ticker for a
        // random market that happens to be listed first in the response map.
        let market = markets.get(target_symbol)
            .ok_or_else(|| ExchangeError::Parse(format!(
                "Market '{}' not found in v4_markets contents", target_symbol
            )))?;

        Ok(Ticker {
            symbol: Self::get_str(market, "ticker").unwrap_or(target_symbol).to_string(),
            last_price: Self::get_f64(market, "oraclePrice").unwrap_or(0.0),
            bid_price: None, // Not provided in v4_markets
            ask_price: None, // Not provided in v4_markets
            high_24h: None,
            low_24h: None,
            volume_24h: Self::get_f64(market, "volume24H"),
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(market, "priceChange24H"),
            price_change_percent_24h: Self::get_f64(market, "priceChange24H")
                .and_then(|change| {
                    Self::get_f64(market, "oraclePrice")
                        .map(|price| (change / (price - change)) * 100.0)
                }),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let contents = data.get("contents")
            .ok_or_else(|| ExchangeError::Parse("Missing 'contents' field".to_string()))?;

        let side = match Self::get_str(contents, "side").unwrap_or("BUY") {
            "SELL" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        let created_at = Self::get_str(contents, "createdAt")
            .and_then(Self::parse_iso_timestamp)
            .unwrap_or(0);

        Ok(PublicTrade {
            id: Self::get_str(contents, "id").unwrap_or("").to_string(),
            symbol: data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            price: Self::require_f64(contents, "price")?,
            quantity: Self::get_f64(contents, "size").unwrap_or(0.0),
            side,
            timestamp: created_at,
        })
    }

    /// Parse WebSocket orderbook delta message
    pub fn parse_ws_orderbook_delta(data: &Value) -> ExchangeResult<StreamEvent> {
        let contents = data.get("contents")
            .ok_or_else(|| ExchangeError::Parse("Missing 'contents' field".to_string()))?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            contents.get(key)
                .and_then(|arr| arr.as_array())
                .map(|levels| {
                    levels.iter()
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

        Ok(StreamEvent::OrderbookDelta(OrderbookDeltaData {
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            timestamp: chrono::Utc::now().timestamp_millis(),
            first_update_id: None,
            last_update_id: None,
            prev_update_id: None,
            event_time: None,
            checksum: None,
        }))
    }

    /// Parse a `v4_candles` WebSocket message into a [`Kline`].
    ///
    /// ## Expected wire format
    ///
    /// ```json
    /// {
    ///   "type": "channel_data",
    ///   "channel": "v4_candles",
    ///   "id": "BTC-USD/1MIN",
    ///   "contents": {
    ///     "startedAt": "2024-01-01T00:00:00.000Z",
    ///     "open":  "42000.5",
    ///     "high":  "42100.0",
    ///     "low":   "41900.0",
    ///     "close": "42050.0",
    ///     "baseTokenVolume": "100.5",
    ///     "usdVolume": "4220025.0",
    ///     "trades": 150
    ///   }
    /// }
    /// ```
    ///
    /// The outer `"id"` field carries `"{SYMBOL}/{RESOLUTION}"` (e.g. `"BTC-USD/1MIN"`).
    ///
    /// ## Returns
    ///
    /// A `StreamEvent::Kline` wrapping the parsed [`Kline`], or an error if any
    /// required field is missing or un-parseable.
    pub fn parse_ws_candle(data: &Value) -> ExchangeResult<StreamEvent> {
        let contents = data
            .get("contents")
            .ok_or_else(|| ExchangeError::Parse(
                "v4_candles: missing 'contents' field".to_string()
            ))?;

        let open_time = Self::get_str(contents, "startedAt")
            .and_then(Self::parse_iso_timestamp)
            .unwrap_or(0);

        let open  = Self::require_f64(contents, "open")?;
        let high  = Self::require_f64(contents, "high")?;
        let low   = Self::require_f64(contents, "low")?;
        let close = Self::require_f64(contents, "close")?;

        let volume = Self::get_f64(contents, "baseTokenVolume").unwrap_or(0.0);
        let quote_volume = Self::get_f64(contents, "usdVolume");
        let trades = contents
            .get("trades")
            .and_then(|t| t.as_u64());

        let kline = Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume,
            close_time: None,
            trades,
        };

        Ok(StreamEvent::Kline(kline))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse historical funding payments from `GET /v4/fundingPayments`.
    ///
    /// Response:
    /// ```json
    /// {"fundingPayments":[
    ///   {"market":"BTC-USD","payment":"-0.01","rate":"0.0001",
    ///    "positionSize":"0.1","effectiveAt":"2024-01-01T00:00:00Z"}
    /// ]}
    /// ```
    pub fn parse_funding_payments(response: &Value) -> ExchangeResult<Vec<FundingPayment>> {
        let list = response.get("fundingPayments")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse(
                "Missing 'fundingPayments' in response".to_string(),
            ))?;

        let mut payments = Vec::with_capacity(list.len());
        for item in list {
            let symbol = item.get("market")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let funding_rate = item.get("rate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let position_size = item.get("positionSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let payment = item.get("payment")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // dYdX perpetuals always settle in USDC
            let asset = "USDC".to_string();

            // effectiveAt is ISO-8601 string → parse to unix ms
            let timestamp = item.get("effectiveAt")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);

            payments.push(FundingPayment {
                symbol,
                funding_rate,
                position_size,
                payment,
                asset,
                timestamp,
            });
        }
        Ok(payments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "bids": [
                {"price": "50000.0", "size": "1.5"},
                {"price": "49999.0", "size": "2.3"}
            ],
            "asks": [
                {"price": "50001.0", "size": "0.8"},
                {"price": "50002.0", "size": "1.2"}
            ]
        });

        let orderbook = DydxParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].price - 50000.0).abs() < f64::EPSILON);
        assert!((orderbook.bids[0].size - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "markets": {
                "BTC-USD": {
                    "ticker": "BTC-USD",
                    "oraclePrice": "50000.5",
                    "volume24H": "125000000.50",
                    "priceChange24H": "1250.75"
                }
            }
        });

        let ticker = DydxParser::parse_ticker(&response, "BTC-USD").unwrap();
        assert_eq!(ticker.symbol, "BTC-USD");
        assert!((ticker.last_price - 50000.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "candles": [
                {
                    "startedAt": "2026-01-20T12:00:00.000Z",
                    "ticker": "BTC-USD",
                    "resolution": "1MIN",
                    "low": "49950.0",
                    "high": "50100.0",
                    "open": "50000.0",
                    "close": "50050.0",
                    "baseTokenVolume": "125.5",
                    "usdVolume": "6277500.0",
                    "trades": 543
                }
            ]
        });

        let klines = DydxParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert!((klines[0].open - 50000.0).abs() < f64::EPSILON);
        assert!((klines[0].close - 50050.0).abs() < f64::EPSILON);
    }
}
