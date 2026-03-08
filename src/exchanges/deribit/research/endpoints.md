# Deribit API Endpoints

Complete endpoint reference for Deribit V5 connector implementation.

## Base URLs

### REST API (JSON-RPC over HTTP)
- **Production**: `https://www.deribit.com/api/v2`
- **Test**: `https://test.deribit.com/api/v2`

### WebSocket API (JSON-RPC over WebSocket)
- **Production**: `wss://www.deribit.com/ws/api/v2`
- **Test**: `wss://test.deribit.com/ws/api/v2`

## Protocol Notes

- All endpoints use **JSON-RPC 2.0** protocol
- HTTP Methods: GET and POST supported
- Method format: `{scope}/{method_name}` (e.g., `public/get_instruments`)
- Parameters must be named objects (no positional parameters)
- Request ID required for all requests

## MarketData Trait Endpoints

### Get Instruments
**Method**: `public/get_instruments`
**Scope**: Public
**Description**: Retrieves available trading instruments

**Parameters**:
- `currency` (required): string - Currency symbol: `BTC`, `ETH`, `USDC`, `USDT`, `EURR`, or `"any"`
- `kind` (optional): string - Instrument category: `future`, `option`, `spot`, `future_combo`, `option_combo`
- `expired` (optional): boolean - Default: `false`. Set `true` to retrieve recently expired instruments

**Response Fields**:
- `tick_size`: number
- `tick_size_steps`: array
- `taker_commission`: number
- `settlement_period`: string
- `settlement_currency`: string
- `quote_currency`: string
- `price_index`: string
- `min_trade_amount`: number
- `max_liquidation_commission`: number
- `max_leverage`: number
- `maker_commission`: number
- `kind`: string (future, option, perpetual, etc.)
- `is_active`: boolean
- `instrument_name`: string
- `instrument_id`: integer
- `instrument_type`: string
- `expiration_timestamp`: integer (milliseconds)
- `creation_timestamp`: integer (milliseconds)
- `counter_currency`: string
- `contract_size`: number
- `block_trade_tick_size`: number
- `block_trade_min_trade_amount`: number
- `block_trade_commission`: number
- `base_currency`: string
- `state`: string

**Rate Limit**: Sustained rate: 1 request/second

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/get_instruments",
  "params": {
    "currency": "BTC",
    "kind": "future"
  }
}
```

---

### Get Order Book
**Method**: `public/get_order_book`
**Scope**: Public
**Description**: Retrieves the order book for a given instrument

**Parameters**:
- `instrument_name` (required): string - Instrument name (e.g., "BTC-PERPETUAL")
- `depth` (optional): integer - Number of entries to return for bids and asks

**Best Practice**: Do NOT call this in a loop. Use WebSocket `book.*` channels for real-time updates.

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "public/get_order_book",
  "params": {
    "instrument_name": "BTC-PERPETUAL",
    "depth": 10
  }
}
```

---

### Get Ticker
**Method**: `public/ticker`
**Scope**: Public
**Description**: Gets ticker data for an instrument (best bid/ask, 24h volume, etc.)

**Parameters**:
- `instrument_name` (required): string - Instrument name (e.g., "BTC-PERPETUAL")

**Note**: This is the most "precise" endpoint for price data.

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 8106,
  "method": "public/ticker",
  "params": {
    "instrument_name": "BTC-PERPETUAL"
  }
}
```

---

### Get Book Summary by Currency
**Method**: `public/get_book_summary_by_currency`
**Scope**: Public
**Description**: Retrieves summary information (open interest, 24h volume, etc.) for all instruments for a currency

**Parameters**:
- `currency` (required): string - Currency (e.g., "BTC", "ETH", "USDT")
- `kind` (optional): string - Filter by instrument kind

**Best Practice**: Use WebSocket subscription to `ticker.{instrument_name}.{interval}` for real-time updates instead of polling.

---

### Get Last Trades by Instrument
**Method**: `public/get_last_trades_by_instrument`
**Scope**: Public
**Description**: Retrieves the latest trades for a specific instrument

**Parameters**:
- `instrument_name` (required): string - Instrument name
- `count` (optional): integer - Number of trades to retrieve
- `include_old` (optional): boolean - Include older trades

**Related Methods**:
- `public/get_last_trades_by_instrument_and_time` - Trades within a specific time range
- `public/get_last_trades_by_currency` - Trades by currency

---

### Get Order Book by Instrument ID
**Method**: `public/get_order_book_by_instrument_id`
**Scope**: Public
**Description**: Retrieves order book using instrument ID instead of name

**Parameters**:
- `instrument_id` (required): integer - Instrument ID
- `depth` (optional): integer - Number of entries for bids/asks

---

## Trading Trait Endpoints

### Place Buy Order
**Method**: `private/buy`
**Scope**: Private (requires authentication)
**Description**: Places a buy order

**Required Parameters**:
- `instrument_name`: string - Instrument name (e.g., "BTC-PERPETUAL", "ETH-19MAR21")
- `amount`: number - Amount in units of base currency
  - For perpetual and inverse futures: Amount is in USD units
  - For options: Amount is in underlying asset's base currency

**Optional Parameters**:
- `price`: number - Limit price (ignored for market orders)
- `type`: string - Order type: `market` or `limit` (default: `limit`)
- `time_in_force`: string - Options:
  - `good_til_cancelled` (GTC) - Default, remains open until filled/cancelled/expired
  - `good_til_day` (GTD) - Cancelled at 8 AM UTC
  - `immediate_or_cancel` (IOC) - Unfilled portion cancelled immediately
  - `fill_or_kill` (FOK) - Filled completely or not at all
- `post_only`: boolean - Ensures order is placed as maker
- `reduce_only`: boolean - Can only reduce position, not increase
- `label`: string - Custom label for the order
- `max_show`: number - For iceberg orders (min 100x instrument's min order size, min 1% of total)
- `trigger`: string - Trigger type: `index_price`, `mark_price`, or `last_price` (default)
- `trigger_price`: number - Price at which order is triggered
- `advanced`: string - Advanced order type (e.g., "usd")

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 5275,
  "method": "private/buy",
  "params": {
    "instrument_name": "BTC-PERPETUAL",
    "amount": 100,
    "type": "limit",
    "price": 50000
  }
}
```

---

### Place Sell Order
**Method**: `private/sell`
**Scope**: Private
**Description**: Places a sell order

**Parameters**: Same as `private/buy`

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 5276,
  "method": "private/sell",
  "params": {
    "instrument_name": "BTC-PERPETUAL",
    "amount": 100,
    "type": "limit",
    "price": 51000
  }
}
```

---

### Edit Order
**Method**: `private/edit`
**Scope**: Private
**Description**: Edits price and/or quantity of an existing order

**Parameters**:
- `order_id`: string (required) - Order ID to edit
- `amount`: number (optional) - New amount
- `price`: number (optional) - New price
- `trigger_price`: number (optional) - New trigger price

**Note**: Orders with invalid quantity or price parameters will be rejected (not truncated).

---

### Cancel Order
**Method**: `private/cancel`
**Scope**: Private
**Description**: Cancels a single order by order ID

**Parameters**:
- `order_id`: string (required) - Unique order identifier

---

### Cancel by Label
**Method**: `private/cancel_by_label`
**Scope**: Private
**Description**: Cancels all orders with a specific label

**Parameters**:
- `label`: string (required) - Label to match
- `detailed`: boolean (optional) - If true, returns execution reports instead of count

---

### Cancel All
**Method**: `private/cancel_all`
**Scope**: Private
**Description**: Cancels all open orders across all instruments and currencies

**Parameters**:
- `detailed`: boolean (optional) - If true, returns detailed execution reports

**Note**: Can cancel unprocessed buy/sell/edit requests in the queue.

---

### Cancel All by Currency
**Method**: `private/cancel_all_by_currency`
**Scope**: Private
**Description**: Cancels all orders for a specific currency

**Parameters**:
- `currency`: string (required) - Currency (e.g., "BTC", "ETH")
- `kind`: string (optional) - Filter by instrument kind
- `type`: string (optional) - Filter by order type
- `detailed`: boolean (optional) - If true, returns detailed execution reports

---

### Get Open Orders
**Method**: `private/get_open_orders`
**Scope**: Private
**Description**: Retrieves all open orders

**Scope Required**: `trade:read`

**Parameters**:
- `type`: string (optional) - Filter by order type: `algo_all`, `take_all`, `take_market`, `take_limit`

**Related Methods**:
- `private/get_open_orders_by_currency` - Filter by currency
- `private/get_open_orders_by_instrument` - Filter by instrument
- `private/get_open_orders_by_label` - Filter by label

---

### Get Order State
**Method**: `private/get_order_state`
**Scope**: Private
**Description**: Gets the current state of a specific order

**Parameters**:
- `order_id`: string (required) - Order ID

---

### Close Position
**Method**: `private/close_position`
**Scope**: Private
**Description**: Closes a position for a specific instrument

**Parameters**:
- `instrument_name`: string (required) - Instrument name
- `type`: string (required) - Order type: `market` or `limit`
- `price`: number (optional) - Price for limit orders

---

## Account Trait Endpoints

### Get Account Summary
**Method**: `private/get_account_summary`
**Scope**: Private
**Description**: Retrieves user account summary information

**Parameters**:
- `currency`: string (required) - Currency (e.g., "BTC", "ETH", "USDC")
- `extended`: boolean (optional) - Include extended information (e.g., MMP status)

**Example**:
```
GET /api/v2/private/get_account_summary?currency=BTC&extended=true
```

**Response includes**:
- Available balance
- Equity
- Margin balance
- Initial margin
- Maintenance margin
- Portfolio margining details
- Rate limits (if called)

---

### Get User Trades by Instrument
**Method**: `private/get_user_trades_by_instrument`
**Scope**: Private
**Description**: Retrieves user's trade history for a specific instrument

**Parameters**:
- `instrument_name`: string (required) - Instrument name
- `count`: integer (optional) - Number of trades to retrieve
- `start_timestamp`: integer (optional) - Start time (milliseconds)
- `end_timestamp`: integer (optional) - End time (milliseconds)
- `sorting`: string (optional) - Sort order: `asc` or `desc`

**Related Methods**:
- `private/get_user_trades_by_currency` - Trades by currency
- `private/get_user_trades_by_order` - Trades for specific order

---

### Get Settlement History by Instrument
**Method**: `private/get_settlement_history_by_instrument`
**Scope**: Private
**Description**: Retrieves settlement history for an instrument

**Parameters**:
- `instrument_name`: string (required) - Instrument name
- `type`: string (optional) - Settlement type
- `count`: integer (optional) - Number of records

---

## Positions Trait Endpoints

### Get Position
**Method**: `private/get_position`
**Scope**: Private
**Description**: Retrieves position information for a specific instrument

**Parameters**:
- `instrument_name`: string (required) - Instrument name

**Note**: For USDC instruments, delta is in base currency instead of USDC.

---

### Get Positions
**Method**: `private/get_positions`
**Scope**: Private
**Description**: Retrieves all positions for a currency

**Parameters**:
- `currency`: string (required) - Currency (e.g., "BTC", "ETH")
- `kind`: string (optional) - Filter by instrument kind

**Example**:
```
GET /api/v2/private/get_positions?currency=BTC
```

---

## Wallet/Transfer Endpoints (Optional)

### Get Transfers
**Method**: `private/get_transfers`
**Scope**: Private (requires `wallet:read_write`)
**Description**: Retrieves transfer information between accounts

**Parameters**:
- `currency`: string (optional) - Filter by currency
- `count`: integer (optional) - Number of transfers
- `offset`: integer (optional) - Pagination offset

---

### Withdraw
**Method**: `private/withdraw`
**Scope**: Private (requires `wallet:read_write`)
**Description**: Creates a withdrawal request

**Parameters**:
- `currency`: string (required) - Currency to withdraw
- `address`: string (required) - Destination address
- `amount`: number (required) - Amount to withdraw
- `priority`: string (optional) - Transaction priority

**Note**: Funds are deducted only after Deribit processes the withdrawal.

---

### Get Deposits
**Method**: `private/get_deposits`
**Scope**: Private (requires `wallet:read_write`)
**Description**: Retrieves deposit history

**Parameters**:
- `currency`: string (required) - Currency
- `count`: integer (optional) - Number of deposits
- `offset`: integer (optional) - Pagination offset

**Response Fields**:
- `count`: Total number of deposits
- `data`: Array of deposits
  - `confirmation_count`: Number of confirmations
  - `amount`: Deposit amount
  - `transaction_id`: Transaction hash
  - `state`: Deposit state
  - `updated_timestamp`: Last update time

---

## Best Practices

1. **Prefer WebSocket over REST**: WebSocket is faster and supports subscriptions
2. **Avoid Polling**: Use WebSocket subscriptions (`book.*`, `ticker.*`, `trades.*`) instead of polling REST endpoints
3. **Rate Limits**: Monitor credit consumption and implement exponential backoff for error 10028
4. **Order Validation**: Ensure quantity and price parameters are valid (orders with invalid params are rejected)
5. **Mass Operations**: Use batch methods (`cancel_all_by_currency`, etc.) for efficiency
6. **Subscriptions**: Batch subscriptions in one call (up to 500 channels)
7. **Connection Management**: Max 32 WebSocket connections per IP, 16 sessions per API key
8. **Error Handling**: Implement retry logic for transient errors with exponential backoff

---

## Additional Notes

- All timestamps are in milliseconds (UNIX epoch)
- Deribit uses cash settlement only (no physical delivery)
- Settlement occurs daily at 08:00 UTC
- Test and production environments require separate accounts and API keys
- Production and testnet have separate rate-limit pools
