# Lighter Exchange Response Formats

## Standard Response Structure

All Lighter API responses follow a consistent structure with status codes and messages.

---

## Success Response Format

### Standard Success Response

```json
{
  "code": 200,
  "message": "string",
  "data": {}
}
```

**Fields**:
- `code` (integer): HTTP status code (200 for success)
- `message` (string): Human-readable status message
- `data` (object/array): Response payload (varies by endpoint)

---

## Error Response Format

### Standard Error Response

```json
{
  "code": 400,
  "message": "Error description"
}
```

**Common Error Codes**:
- `400` - Bad Request (invalid parameters, malformed request)
- `401` - Unauthorized (invalid or expired auth token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found (resource doesn't exist)
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error
- `503` - Service Unavailable

**Error Message Examples**:
- `"Invalid signature"`
- `"Invalid nonce"`
- `"Token expired"`
- `"Insufficient balance"`
- `"Market not found"`
- `"Order not found"`

---

## Market Data Responses

### Order Books Response

**Endpoint**: `GET /api/v1/orderBooks`

```json
{
  "code": 200,
  "message": "string",
  "order_books": [
    {
      "symbol": "ETH",
      "market_id": 0,
      "market_type": "perp",
      "base_asset_id": 0,
      "quote_asset_id": 0,
      "status": "active",
      "taker_fee": "0.0001",
      "maker_fee": "0.0000",
      "liquidation_fee": "0.01",
      "min_base_amount": "0.01",
      "min_quote_amount": "0.1",
      "supported_size_decimals": 4,
      "supported_price_decimals": 4,
      "supported_quote_decimals": 4,
      "order_quote_limit": "281474976.710655"
    }
  ]
}
```

**Field Types**:
- `symbol` (string): Market symbol (e.g., "ETH", "BTC")
- `market_id` (integer): Unique market identifier
- `market_type` (string): "perp" or "spot"
- `base_asset_id` (integer): Base asset identifier
- `quote_asset_id` (integer): Quote asset identifier (typically USDC)
- `status` (string): "active" or "inactive"
- `taker_fee` (string): Taker fee as decimal (e.g., "0.0001" = 0.01%)
- `maker_fee` (string): Maker fee as decimal (e.g., "0.0000" = 0%)
- `liquidation_fee` (string): Liquidation fee as decimal
- `min_base_amount` (string): Minimum order size in base asset
- `min_quote_amount` (string): Minimum order value in quote asset
- `supported_size_decimals` (integer): Decimal places for size
- `supported_price_decimals` (integer): Decimal places for price
- `supported_quote_decimals` (integer): Decimal places for quote
- `order_quote_limit` (string): Maximum order value

---

### Order Book Details Response

**Endpoint**: `GET /api/v1/orderBookDetails`

```json
{
  "code": 200,
  "message": "string",
  "order_book_details": [
    {
      "symbol": "ETH",
      "market_id": 0,
      "market_type": "perp",
      "base_asset_id": 0,
      "quote_asset_id": 0,
      "status": "active",
      "taker_fee": "0.0001",
      "maker_fee": "0.0000",
      "liquidation_fee": "0.01",
      "min_base_amount": "0.01",
      "min_quote_amount": "0.1",
      "supported_size_decimals": 4,
      "supported_price_decimals": 4,
      "supported_quote_decimals": 4,
      "order_quote_limit": "281474976.710655",
      "size_decimals": 4,
      "price_decimals": 4,
      "quote_multiplier": 10000,
      "default_initial_margin_fraction": 100,
      "min_initial_margin_fraction": 100,
      "maintenance_margin_fraction": 50,
      "closeout_margin_fraction": 100,
      "last_trade_price": 3024.66,
      "daily_trades_count": 68,
      "daily_base_token_volume": 235.25,
      "daily_quote_token_volume": 93566.25,
      "daily_price_low": 3014.66,
      "daily_price_high": 3024.66,
      "daily_price_change": 3.66,
      "open_interest": 93.0,
      "daily_chart": {
        "1640995200": 3024.66
      },
      "market_config": {
        "market_margin_mode": 0,
        "insurance_fund_account_index": 281474976710655,
        "liquidation_mode": 0,
        "force_reduce_only": false,
        "trading_hours": ""
      }
    }
  ],
  "spot_order_book_details": [
    {
      "symbol": "ETH/USDC",
      "market_id": 2048,
      "market_type": "spot",
      "base_asset_id": 1,
      "quote_asset_id": 3,
      "status": "active",
      "last_trade_price": 2731.79,
      "daily_price_change": -10.24
    }
  ]
}
```

**Additional Fields** (vs orderBooks):
- `quote_multiplier` (integer): Multiplier for quote calculations
- `default_initial_margin_fraction` (integer): Default margin requirement (basis points)
- `min_initial_margin_fraction` (integer): Minimum margin requirement (basis points)
- `maintenance_margin_fraction` (integer): Maintenance margin level (basis points)
- `closeout_margin_fraction` (integer): Closeout threshold (basis points)
- `last_trade_price` (number): Most recent trade price
- `daily_trades_count` (integer): Number of trades in last 24h
- `daily_base_token_volume` (number): 24h volume in base asset
- `daily_quote_token_volume` (number): 24h volume in quote asset
- `daily_price_low` (number): 24h low price
- `daily_price_high` (number): 24h high price
- `daily_price_change` (number): 24h price change
- `open_interest` (number): Current open interest
- `daily_chart` (object): Map of timestamp to price for daily chart
- `market_config` (object): Market configuration settings

---

### Trades Response

**Endpoint**: `GET /api/v1/recentTrades` or `GET /api/v1/trades`

```json
{
  "code": 200,
  "message": "string",
  "trades": [
    {
      "trade_id": 12345,
      "tx_hash": "0xabc123...",
      "type": "string",
      "market_id": 0,
      "size": "1.5",
      "price": "3024.66",
      "usd_amount": "4536.99",
      "ask_id": 100,
      "bid_id": 101,
      "ask_account_id": 1000,
      "bid_account_id": 1001,
      "is_maker_ask": true,
      "block_height": 123456,
      "timestamp": 1640995200,
      "taker_fee": 1000,
      "taker_position_size_before": "10.0",
      "taker_entry_quote_before": "30000.0",
      "taker_initial_margin_fraction_before": 100,
      "taker_position_sign_changed": false,
      "maker_fee": 0,
      "maker_position_size_before": "5.0",
      "maker_entry_quote_before": "15000.0",
      "maker_initial_margin_fraction_before": 100,
      "maker_position_sign_changed": false,
      "transaction_time": 1640995200
    }
  ]
}
```

**Field Types**:
- `trade_id` (integer): Unique trade identifier
- `tx_hash` (string): Transaction hash
- `type` (string): Trade type
- `market_id` (integer): Market identifier
- `size` (string): Trade size in base asset
- `price` (string): Trade price
- `usd_amount` (string): Trade value in USD
- `ask_id` (integer): Ask order ID
- `bid_id` (integer): Bid order ID
- `ask_account_id` (integer): Seller account ID
- `bid_account_id` (integer): Buyer account ID
- `is_maker_ask` (boolean): True if ask was maker side
- `block_height` (integer): Block number
- `timestamp` (integer): Unix timestamp
- `taker_fee` (integer): Taker fee paid (omitted if zero)
- `taker_position_size_before` (string): Taker's position before trade (omitted if empty)
- `taker_entry_quote_before` (string): Taker's entry quote before trade (omitted if empty)
- `taker_initial_margin_fraction_before` (integer): Taker's margin before (omitted if zero)
- `taker_position_sign_changed` (boolean): True if position flipped (omitted if false)
- `maker_fee` (integer): Maker fee paid (omitted if zero)
- `maker_position_size_before` (string): Maker's position before trade (omitted if empty)
- `maker_entry_quote_before` (string): Maker's entry quote before trade (omitted if empty)
- `maker_initial_margin_fraction_before` (integer): Maker's margin before (omitted if zero)
- `maker_position_sign_changed` (boolean): True if position flipped (omitted if false)
- `transaction_time` (integer): Transaction timestamp

**Note**: Optional fields are omitted when they have default/zero values.

---

### Candlesticks (OHLCV) Response

**Endpoint**: `GET /api/v1/candlesticks`

```json
{
  "code": 200,
  "message": "string",
  "candlesticks": [
    {
      "timestamp": 1640995200,
      "open": "3020.00",
      "high": "3030.00",
      "low": "3015.00",
      "close": "3024.66",
      "volume": "235.25",
      "quote_volume": "93566.25"
    }
  ]
}
```

**Field Types**:
- `timestamp` (integer): Unix timestamp for candle start
- `open` (string): Opening price
- `high` (string): High price
- `low` (string): Low price
- `close` (string): Closing price
- `volume` (string): Volume in base asset
- `quote_volume` (string): Volume in quote asset

**Supported Resolutions**: "1m", "5m", "15m", "1h", "4h", "1d"

---

### Funding Rates Response

**Endpoint**: `GET /api/v1/fundings`

```json
{
  "code": 200,
  "message": "string",
  "fundings": [
    {
      "market_id": 0,
      "funding_rate": "0.0001",
      "timestamp": 1640995200
    }
  ]
}
```

**Field Types**:
- `market_id` (integer): Market identifier
- `funding_rate` (string): Current funding rate as decimal
- `timestamp` (integer): Unix timestamp

---

### Exchange Statistics Response

**Endpoint**: `GET /api/v1/exchangeStats`

```json
{
  "code": 200,
  "message": "string",
  "stats": {
    "total_volume_24h": "1000000.0",
    "total_trades_24h": 10000,
    "active_markets": 50
  }
}
```

---

## Account Data Responses

### Account Information Response

**Endpoint**: `GET /api/v1/account`

```json
{
  "code": 200,
  "message": "string",
  "total": 1,
  "accounts": [
    {
      "code": 200,
      "account_type": 1,
      "index": 1,
      "l1_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "status": 1,
      "collateral": "46342",
      "available_balance": "19995",
      "total_order_count": 100,
      "pending_order_count": 100,
      "positions": [
        {
          "market_id": 1,
          "symbol": "ETH",
          "open_order_count": 3,
          "sign": 1,
          "position": "3.6956",
          "avg_entry_price": "3024.66",
          "unrealized_pnl": "17.521309",
          "realized_pnl": "2.000000"
        }
      ],
      "assets": [
        {
          "symbol": "USDC",
          "asset_id": 1,
          "balance": "1000",
          "locked_balance": "1000"
        }
      ]
    }
  ]
}
```

**Account Fields**:
- `code` (integer): Status code for this account
- `account_type` (integer): Account type (0 = Standard, 1 = Premium)
- `index` (integer): Account index
- `l1_address` (string): Layer 1 wallet address
- `status` (integer): 1 = active, 0 = inactive
- `collateral` (string): Total collateral in USDC
- `available_balance` (string): Available balance for trading
- `total_order_count` (integer): Total orders placed (all time)
- `pending_order_count` (integer): Currently active orders

**Position Fields**:
- `market_id` (integer): Market identifier
- `symbol` (string): Market symbol
- `open_order_count` (integer): Active orders for this market
- `sign` (integer): 1 = Long, -1 = Short, 0 = No position
- `position` (string): Position size in base asset
- `avg_entry_price` (string): Average entry price
- `unrealized_pnl` (string): Unsettled profit/loss
- `realized_pnl` (string): Settled profit/loss

**Asset Fields**:
- `symbol` (string): Asset symbol (e.g., "USDC")
- `asset_id` (integer): Asset identifier
- `balance` (string): Total balance
- `locked_balance` (string): Balance locked in orders/positions

---

### API Keys Response

**Endpoint**: `GET /api/v1/apikeys`

```json
{
  "code": 200,
  "message": "string",
  "api_keys": [
    {
      "api_key_index": 3,
      "public_key": "0xabc123...",
      "status": "active",
      "created_at": 1640995200
    }
  ]
}
```

**Field Types**:
- `api_key_index` (integer): API key index (3-254)
- `public_key` (string): Public key for signature verification
- `status` (string): "active" or "inactive"
- `created_at` (integer): Unix timestamp of creation

---

### Account Inactive Orders Response

**Endpoint**: `GET /api/v1/accountInactiveOrders`

```json
{
  "code": 200,
  "message": "string",
  "orders": [
    {
      "order_index": 12345,
      "client_order_index": 100,
      "market_id": 0,
      "side": "buy",
      "order_type": "limit",
      "base_amount": "1.5",
      "price": "3024.66",
      "status": "filled",
      "created_at": 1640995200,
      "updated_at": 1640995300
    }
  ]
}
```

**Field Types**:
- `order_index` (integer): Order identifier
- `client_order_index` (integer): Client-provided order ID
- `market_id` (integer): Market identifier
- `side` (string): "buy" or "sell"
- `order_type` (string): "limit" or "market"
- `base_amount` (string): Order size
- `price` (string): Order price (for limit orders)
- `status` (string): "filled", "cancelled", "expired"
- `created_at` (integer): Order creation timestamp
- `updated_at` (integer): Last update timestamp

---

### PnL Response

**Endpoint**: `GET /api/v1/pnl`

```json
{
  "code": 200,
  "message": "string",
  "pnl": [
    {
      "market_id": 0,
      "realized_pnl": "100.50",
      "unrealized_pnl": "50.25",
      "total_pnl": "150.75"
    }
  ]
}
```

**Field Types**:
- `market_id` (integer): Market identifier
- `realized_pnl` (string): Settled profit/loss
- `unrealized_pnl` (string): Unsettled profit/loss on open positions
- `total_pnl` (string): Total PnL (realized + unrealized)

---

## Transaction Responses

### Send Transaction Response

**Endpoint**: `POST /api/v1/sendTx`

```json
{
  "code": 200,
  "message": "Transaction accepted",
  "tx_hash": "0xabc123..."
}
```

**Field Types**:
- `tx_hash` (string): Transaction hash for tracking

---

### Send Transaction Batch Response

**Endpoint**: `POST /api/v1/sendTxBatch`

```json
{
  "code": 200,
  "message": "Batch accepted",
  "tx_hashes": [
    "0xabc123...",
    "0xdef456..."
  ]
}
```

**Field Types**:
- `tx_hashes` (array of strings): Transaction hashes for each transaction in batch

---

### Next Nonce Response

**Endpoint**: `GET /api/v1/nextNonce`

```json
{
  "code": 200,
  "message": "string",
  "nonce": 42
}
```

**Field Types**:
- `nonce` (integer): Next nonce value to use

---

### Account Transactions Response

**Endpoint**: `GET /api/v1/accountTxs`

```json
{
  "code": 200,
  "message": "string",
  "transactions": [
    {
      "hash": "0xabc123...",
      "type": 14,
      "info": "{...}",
      "event_info": "{...}",
      "status": 1,
      "transaction_index": 100,
      "l1_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "account_index": 1,
      "nonce": 1,
      "expire_at": 1640999999,
      "block_height": 123456,
      "queued_at": 1640995200,
      "executed_at": 1640995201,
      "sequence_index": 1,
      "parent_hash": "0xparent123...",
      "transaction_time": 1640995201
    }
  ]
}
```

**Field Types**:
- `hash` (string): Transaction hash
- `type` (integer): Transaction type code (see authentication.md)
- `info` (string): JSON object as string with transaction details
- `event_info` (string): JSON object as string with event details
- `status` (integer): Transaction status (1 = success, 0 = failed/pending)
- `transaction_index` (integer): Index within block
- `l1_address` (string): Layer 1 address
- `account_index` (integer): Account identifier
- `nonce` (integer): Transaction nonce
- `expire_at` (integer): Expiry timestamp
- `block_height` (integer): Block number
- `queued_at` (integer): Queue timestamp
- `executed_at` (integer): Execution timestamp
- `sequence_index` (integer): Sequence number
- `parent_hash` (string): Parent transaction hash
- `transaction_time` (integer): Transaction timestamp

---

### Deposit History Response

**Endpoint**: `GET /api/v1/deposit/history`

```json
{
  "code": 200,
  "message": "string",
  "deposits": [
    {
      "tx_hash": "0xabc123...",
      "l1_tx_hash": "0xl1abc123...",
      "amount": "1000.0",
      "asset_id": 1,
      "status": "completed",
      "timestamp": 1640995200
    }
  ]
}
```

**Field Types**:
- `tx_hash` (string): L2 transaction hash
- `l1_tx_hash` (string): L1 transaction hash
- `amount` (string): Deposit amount
- `asset_id` (integer): Asset identifier
- `status` (string): "completed", "pending", "failed"
- `timestamp` (integer): Deposit timestamp

---

### Withdrawal History Response

**Endpoint**: `GET /api/v1/withdraw/history`

```json
{
  "code": 200,
  "message": "string",
  "withdrawals": [
    {
      "tx_hash": "0xabc123...",
      "l1_tx_hash": "0xl1abc123...",
      "amount": "1000.0",
      "asset_id": 1,
      "status": "completed",
      "timestamp": 1640995200
    }
  ]
}
```

**Field Types**: Same as deposit history

---

## Block and Blockchain Responses

### Current Height Response

**Endpoint**: `GET /api/v1/currentHeight`

```json
{
  "code": 200,
  "message": "string",
  "height": 123456
}
```

**Field Types**:
- `height` (integer): Current blockchain height

---

## Data Type Notes

### Numeric Values

**Strings for Precision**:
Most numeric values (prices, amounts, balances) are returned as **strings** to preserve precision and avoid floating-point errors.

**Examples**:
- `"3024.66"` - Price as string
- `"1.5"` - Amount as string
- `"0.0001"` - Fee as string

**Integer Representation**:
When submitting transactions, amounts and prices must be sent as **integers** by multiplying by the appropriate decimal precision.

**Example**:
- Price: `3024.66` → `30246600` (multiply by 10000 if 4 decimal places)
- Amount: `1.5` → `15000` (multiply by 10000 if 4 decimal places)

### Boolean Values

**Representation**: Standard JSON booleans
- `true`
- `false`

**Omission**: Some boolean fields are omitted when `false` to reduce response size.

### Timestamps

**Format**: Unix timestamp (seconds since epoch)
**Type**: Integer

**Example**: `1640995200` = 2021-12-31 16:00:00 UTC

### Addresses

**Format**: Hexadecimal string with `0x` prefix
**Type**: String

**Example**: `"0x70997970C51812dc3A010C7d01b50e0d17dc79C8"`

### Hashes

**Format**: Hexadecimal string with `0x` prefix
**Type**: String

**Example**: `"0xabc123def456..."`

---

## Response Size Optimization

### Omitted Fields

Lighter API responses omit certain fields when they have default/zero values:

**Commonly Omitted**:
- Fee fields when zero: `taker_fee`, `maker_fee`
- Position fields when empty: `taker_position_size_before`, `maker_position_size_before`
- Boolean fields when false: `taker_position_sign_changed`, `maker_position_sign_changed`
- Margin fields when zero: `taker_initial_margin_fraction_before`

**Implication for Parsing**:
- Use optional/nullable types in data structures
- Provide default values when fields are missing
- Don't assume all fields are always present

---

## Pagination

### Pattern

Lighter API uses limit-based pagination for list endpoints.

**Common Parameters**:
- `limit` (integer): Number of items to return
- `start_timestamp` (integer): Start time filter
- `end_timestamp` (integer): End time filter

**Example**:
```
GET /api/v1/trades?market_id=0&limit=100&start_timestamp=1640995200
```

**Note**: No cursor-based pagination; use timestamp ranges for large datasets.

---

## WebSocket Response Formats

See `websocket.md` for detailed WebSocket message formats.

**General Structure**:
```json
{
  "channel": "channel_name",
  "type": "update/[type]",
  "timestamp": 1640995200,
  "data": {}
}
```

---

## Implementation Notes for V5 Connector

### Parsing Considerations

1. **String to Number Conversion**:
   - Parse price/amount strings to `f64` or `Decimal` for calculations
   - Preserve string format when forwarding data
   - Handle scientific notation if present

2. **Optional Fields**:
   - Use `Option<T>` for fields that may be omitted
   - Provide sensible defaults (e.g., `false` for omitted booleans)
   - Use `#[serde(default)]` attribute in Rust

3. **Timestamp Conversion**:
   - Convert Unix timestamps to `DateTime<Utc>` if needed
   - Store as integers for efficiency
   - Handle millisecond vs second precision

4. **Error Handling**:
   - Parse `code` field to determine success/failure
   - Extract `message` field for error details
   - Map error codes to specific error types

### Example Rust Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct Trade {
    pub trade_id: u64,
    pub tx_hash: String,
    #[serde(rename = "type")]
    pub trade_type: String,
    pub market_id: u16,
    pub size: String,
    pub price: String,
    pub usd_amount: String,
    pub ask_id: u64,
    pub bid_id: u64,
    pub ask_account_id: u64,
    pub bid_account_id: u64,
    pub is_maker_ask: bool,
    pub block_height: u64,
    pub timestamp: i64,
    #[serde(default)]
    pub taker_fee: Option<u64>,
    #[serde(default)]
    pub maker_fee: Option<u64>,
    pub transaction_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct Candlestick {
    pub timestamp: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub quote_volume: String,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    pub market_id: u16,
    pub symbol: String,
    pub open_order_count: u32,
    pub sign: i8,  // 1 = Long, -1 = Short, 0 = None
    pub position: String,
    pub avg_entry_price: String,
    pub unrealized_pnl: String,
    pub realized_pnl: String,
}
```

---

## Testing Response Parsing

### Test Cases

1. **Success Responses**:
   - Valid data with all fields present
   - Valid data with optional fields omitted
   - Empty arrays/lists

2. **Error Responses**:
   - Various error codes (400, 401, 429, 500)
   - Different error messages

3. **Edge Cases**:
   - Very large numbers (test string precision)
   - Negative values (PnL, price changes)
   - Zero values
   - Empty strings vs null

4. **Malformed Responses**:
   - Missing required fields
   - Wrong data types
   - Invalid JSON

### Example Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trade_response() {
        let json = r#"{
            "code": 200,
            "message": "success",
            "trades": [{
                "trade_id": 12345,
                "tx_hash": "0xabc",
                "type": "trade",
                "market_id": 0,
                "size": "1.5",
                "price": "3024.66",
                "usd_amount": "4536.99",
                "ask_id": 100,
                "bid_id": 101,
                "ask_account_id": 1000,
                "bid_account_id": 1001,
                "is_maker_ask": true,
                "block_height": 123456,
                "timestamp": 1640995200,
                "transaction_time": 1640995200
            }]
        }"#;

        let response: ApiResponse<TradesData> =
            serde_json::from_str(json).unwrap();

        assert_eq!(response.code, 200);
        assert_eq!(response.data.unwrap().trades.len(), 1);
    }
}
```
