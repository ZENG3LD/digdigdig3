# Crypto.com Exchange API v1 - Response Formats

## Standard Response Structure

All API responses follow a consistent JSON format:

```json
{
  "id": 1,
  "method": "endpoint_method",
  "code": 0,
  "message": "success_or_error_message",
  "result": {
    // Endpoint-specific data
  }
}
```

**Common Fields:**
- `id` - Request ID (matches request)
- `method` - Endpoint method called
- `code` - Status code (0 = success, non-zero = error)
- `message` - Optional message (present on errors)
- `result` - Response data (varies by endpoint)

---

## Success Response (Code 0)

```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 0,
  "result": {
    "order_id": "18342311",
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02"
  }
}
```

---

## Error Response (Code Non-Zero)

```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 10003,
  "message": "INVALID_SIGNATURE",
  "original": "original_request_payload"
}
```

---

## Market Data Responses

### Get Instruments

```json
{
  "id": 1,
  "method": "public/get-instruments",
  "code": 0,
  "result": {
    "data": [
      {
        "instrument_name": "BTCUSD-PERP",
        "quote_currency": "USD",
        "base_currency": "BTC",
        "price_decimals": 1,
        "quantity_decimals": 4,
        "margin_trading_enabled": true,
        "max_quantity": "100000.0000",
        "min_quantity": "0.0001",
        "max_price": "1000000.0",
        "min_price": "0.1",
        "instrument_type": "PERPETUAL_SWAP",
        "contract_size": "1",
        "tradable": true
      }
    ]
  }
}
```

**Field Descriptions:**
- `instrument_name` - Trading pair identifier
- `quote_currency` - Quote asset (e.g., USD, USDT)
- `base_currency` - Base asset (e.g., BTC, ETH)
- `price_decimals` - Price precision
- `quantity_decimals` - Quantity precision
- `margin_trading_enabled` - Margin support flag
- `max_quantity` / `min_quantity` - Order size limits
- `max_price` / `min_price` - Price bounds
- `instrument_type` - Type (SPOT, PERPETUAL_SWAP, FUTURES)
- `contract_size` - Contract multiplier
- `tradable` - Trading enabled flag

---

### Get Order Book

```json
{
  "id": 1,
  "method": "public/get-book",
  "code": 0,
  "result": {
    "depth": 10,
    "instrument_name": "BTCUSD-PERP",
    "data": [
      {
        "asks": [
          ["50126.000000", "0.400000", "0"],
          ["50130.000000", "1.279000", "0"]
        ],
        "bids": [
          ["50113.500000", "0.400000", "0"],
          ["50113.000000", "0.051800", "0"]
        ],
        "t": 1613580710768
      }
    ]
  }
}
```

**Entry Format:** `[price, quantity, order_count]`
- `price` - Limit price (string)
- `quantity` - Total size at price level (string)
- `order_count` - Number of orders (string, often "0")

**Additional Fields:**
- `depth` - Number of levels per side
- `instrument_name` - Trading pair
- `t` - Timestamp (milliseconds)

---

### Get Candlestick

```json
{
  "id": 1,
  "method": "public/get-candlestick",
  "code": 0,
  "result": {
    "instrument_name": "BTCUSD-PERP",
    "interval": "1h",
    "data": [
      {
        "t": 1613577600000,
        "o": "50100.00",
        "h": "50500.00",
        "l": "49800.00",
        "c": "50200.00",
        "v": "123.4567"
      }
    ]
  }
}
```

**OHLCV Fields:**
- `t` - Candle open time (milliseconds)
- `o` - Open price (string)
- `h` - High price (string)
- `l` - Low price (string)
- `c` - Close price (string)
- `v` - Volume (base currency, string)

---

### Get Recent Trades

```json
{
  "id": 1,
  "method": "public/get-trades",
  "code": 0,
  "result": {
    "instrument_name": "BTCUSD-PERP",
    "data": [
      {
        "dataTime": 1613580710768,
        "d": "18342311001",
        "s": "BUY",
        "p": "50100.00",
        "q": "0.5000",
        "t": 1613580710768,
        "i": "BTCUSD-PERP"
      }
    ]
  }
}
```

**Trade Fields:**
- `dataTime` - Trade timestamp (milliseconds)
- `d` - Trade ID (string)
- `s` - Side (BUY/SELL)
- `p` - Trade price (string)
- `q` - Trade quantity (string)
- `t` - Trade time (milliseconds)
- `i` - Instrument name

---

### Get Tickers

```json
{
  "id": 1,
  "method": "public/get-tickers",
  "code": 0,
  "result": {
    "data": [
      {
        "i": "BTCUSD-PERP",
        "b": "51170.000000",
        "k": "51180.000000",
        "a": "51174.500000",
        "c": "0.03955106",
        "h": "51790.00",
        "l": "47895.50",
        "v": "879.5024",
        "vv": "26370000.12",
        "oi": "12345.12",
        "t": 1613580710768
      }
    ]
  }
}
```

**Ticker Fields:**
- `i` - Instrument name
- `b` - Best bid price
- `k` - Best ask price
- `a` - Last traded price
- `c` - 24h change (decimal)
- `h` - 24h high
- `l` - 24h low
- `v` - 24h volume (base currency)
- `vv` - 24h volume (quote currency)
- `oi` - Open interest (derivatives only)
- `t` - Timestamp (milliseconds)

---

### Get Valuations

```json
{
  "id": 1,
  "method": "public/get-valuations",
  "code": 0,
  "result": {
    "data": [
      {
        "instrument_name": "BTCUSD-PERP",
        "index_price": "51000.00",
        "mark_price": "51005.00",
        "last_price": "51010.00",
        "funding_rate": "0.0001",
        "next_funding_time": 1613584800000
      }
    ]
  }
}
```

**Valuation Fields:**
- `instrument_name` - Trading pair
- `index_price` - Index price
- `mark_price` - Mark price (unrealized PnL calculation)
- `last_price` - Latest trade price
- `funding_rate` - Current funding rate
- `next_funding_time` - Next funding timestamp (ms)

---

## Trading Responses

### Create Order

```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 0,
  "result": {
    "order_id": "18342311",
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02"
  }
}
```

**Fields:**
- `order_id` - Exchange-assigned order ID (string)
- `client_oid` - Client-provided order ID (optional, string)

---

### Cancel Order

```json
{
  "id": 1,
  "method": "private/cancel-order",
  "code": 0,
  "message": "NO_ERROR",
  "result": {
    "order_id": "18342311",
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02"
  }
}
```

---

### Get Open Orders

```json
{
  "id": 1,
  "method": "private/get-open-orders",
  "code": 0,
  "result": {
    "count": 2,
    "order_list": [
      {
        "order_id": "18342311",
        "client_oid": "client_order_123",
        "instrument_name": "BTCUSD-PERP",
        "side": "BUY",
        "type": "LIMIT",
        "price": "50000.00",
        "quantity": "0.5000",
        "cumulative_quantity": "0.0000",
        "cumulative_value": "0.00",
        "avg_price": "0.00",
        "fee_currency": "USD",
        "time_in_force": "GOOD_TILL_CANCEL",
        "exec_inst": [],
        "trigger_price": "",
        "ref_price_type": "",
        "status": "ACTIVE",
        "reason": "",
        "create_time": 1587523073344,
        "update_time": 1587523073344,
        "isolation_id": "",
        "isolation_type": ""
      }
    ]
  }
}
```

**Order Object Fields:**
- `order_id` - Unique order identifier
- `client_oid` - Client order ID (optional)
- `instrument_name` - Trading pair
- `side` - BUY or SELL
- `type` - Order type (LIMIT, MARKET, STOP_LOSS, etc.)
- `price` - Limit price (string)
- `quantity` - Total order size
- `cumulative_quantity` - Filled quantity
- `cumulative_value` - Filled notional value
- `avg_price` - Average fill price
- `fee_currency` - Fee denomination
- `time_in_force` - TIF (GOOD_TILL_CANCEL, IOC, FOK)
- `exec_inst` - Execution instructions array
- `trigger_price` - Stop/trigger price (conditional orders)
- `ref_price_type` - Reference price type (MARK_PRICE, INDEX_PRICE, LAST_PRICE)
- `status` - Order status (ACTIVE, FILLED, CANCELED, REJECTED, EXPIRED)
- `reason` - Cancel/reject reason
- `create_time` - Creation timestamp (ms)
- `update_time` - Last update timestamp (ms)
- `isolation_id` - Isolated margin ID
- `isolation_type` - Isolated margin type

**Order Status Values:**
- `ACTIVE` - Open and active
- `FILLED` - Completely filled
- `CANCELED` - Canceled by user/system
- `REJECTED` - Rejected (invalid params, risk)
- `EXPIRED` - Expired (GTD orders)
- `PENDING` - Pending activation (stop orders)

---

### Get Order Detail

```json
{
  "id": 1,
  "method": "private/get-order-detail",
  "code": 0,
  "result": {
    "order_info": {
      "order_id": "18342311",
      "client_oid": "client_order_123",
      "instrument_name": "BTCUSD-PERP",
      "side": "BUY",
      "type": "LIMIT",
      "price": "50000.00",
      "quantity": "0.5000",
      "cumulative_quantity": "0.2500",
      "avg_price": "49950.00",
      "status": "ACTIVE",
      "create_time": 1587523073344,
      "update_time": 1587523180000
    },
    "trade_list": [
      {
        "trade_id": "183423110001",
        "order_id": "18342311",
        "instrument_name": "BTCUSD-PERP",
        "side": "BUY",
        "fee": "0.25",
        "fee_currency": "USD",
        "create_time": 1587523100000,
        "traded_price": "49950.00",
        "traded_quantity": "0.2500",
        "liquidity_indicator": "MAKER"
      }
    ]
  }
}
```

**Trade Object Fields:**
- `trade_id` - Unique trade identifier
- `order_id` - Parent order ID
- `instrument_name` - Trading pair
- `side` - BUY or SELL
- `fee` - Trading fee amount
- `fee_currency` - Fee denomination
- `create_time` - Execution timestamp (ms)
- `traded_price` - Fill price
- `traded_quantity` - Fill quantity
- `liquidity_indicator` - MAKER or TAKER

---

### Get Trades (User Fills)

```json
{
  "id": 1,
  "method": "private/get-trades",
  "code": 0,
  "result": {
    "trade_list": [
      {
        "account_id": "account_123",
        "event_date": "2021-02-17",
        "journal_type": "TRADING",
        "trade_id": "183423110001",
        "order_id": "18342311",
        "client_oid": "client_order_123",
        "instrument_name": "BTCUSD-PERP",
        "side": "BUY",
        "fee": "0.25",
        "fee_instrument": "USD",
        "trade_match_id": "match_123",
        "create_time": 1613580710768,
        "traded_price": "49950.00",
        "traded_quantity": "0.2500",
        "liquidity_indicator": "MAKER",
        "taker_side": "SELL"
      }
    ]
  }
}
```

**Additional Trade Fields:**
- `account_id` - User account identifier
- `event_date` - Trade date (YYYY-MM-DD)
- `journal_type` - Transaction type (TRADING)
- `trade_match_id` - Match identifier
- `taker_side` - Taker's side (opposite of maker)

---

## Account Responses

### Get User Balance

```json
{
  "id": 1,
  "method": "private/user-balance",
  "code": 0,
  "result": {
    "accounts": [
      {
        "balance": "10000.00",
        "available": "9500.00",
        "order": "500.00",
        "stake": "0.00",
        "currency": "USDT"
      }
    ],
    "total_available_balance": "9500.00",
    "total_margin_balance": "10200.00",
    "total_initial_margin": "500.00",
    "total_maintenance_margin": "250.00",
    "total_position_cost": "5000.00",
    "total_cash_balance": "10000.00",
    "total_collateral_value": "10200.00",
    "total_session_unrealized_pnl": "200.00",
    "total_isolated_cash_balance": "0.00",
    "instrument_collateral_list": [
      {
        "instrument_name": "USDT",
        "quantity": "10000.00",
        "reserved_qty": "500.00",
        "locked_qty": "0.00",
        "last_price": "1.00",
        "collateral_weight": "1.0",
        "haircut": "0.0"
      }
    ],
    "isolated_positions": []
  }
}
```

**Balance Fields:**
- `balance` - Total balance
- `available` - Available for trading
- `order` - Reserved in open orders
- `stake` - Staked amount
- `currency` - Asset symbol

**Account Summary Fields:**
- `total_available_balance` - Total available across all assets
- `total_margin_balance` - Total margin (no haircut)
- `total_initial_margin` - IM requirement for positions + orders
- `total_maintenance_margin` - MM requirement
- `total_position_cost` - Total position notional
- `total_cash_balance` - Total cash
- `total_collateral_value` - Collateral value (after haircut)
- `total_session_unrealized_pnl` - Session unrealized PnL
- `total_isolated_cash_balance` - Isolated margin cash

**Instrument Collateral Fields:**
- `instrument_name` - Asset symbol
- `quantity` - Total quantity
- `reserved_qty` - Reserved in orders
- `locked_qty` - Locked (withdrawals, etc.)
- `last_price` - Latest price
- `collateral_weight` - Collateral coefficient
- `haircut` - Haircut percentage

---

### Get Fee Rate

```json
{
  "id": 1,
  "method": "private/get-fee-rate",
  "code": 0,
  "result": {
    "maker_fee_rate": "0.0004",
    "taker_fee_rate": "0.0007",
    "volume_tier": "VIP_0"
  }
}
```

**Fee Fields:**
- `maker_fee_rate` - Maker fee percentage (0.0004 = 0.04%)
- `taker_fee_rate` - Taker fee percentage (0.0007 = 0.07%)
- `volume_tier` - VIP tier (VIP_0, VIP_1, etc.)

---

### Get Instrument Fee Rate

```json
{
  "id": 1,
  "method": "private/get-instrument-fee-rate",
  "code": 0,
  "result": {
    "instrument_name": "BTCUSD-PERP",
    "maker_fee_rate": "0.0004",
    "taker_fee_rate": "0.0007"
  }
}
```

---

## Position Responses

### Get Positions

```json
{
  "id": 1,
  "method": "private/get-positions",
  "code": 0,
  "result": {
    "position_list": [
      {
        "account_id": "account_123",
        "instrument_name": "BTCUSD-PERP",
        "quantity": "1.5000",
        "cost": "75000.00",
        "open_position_pnl": "1500.00",
        "session_pnl": "300.00",
        "entry_price": "50000.00",
        "mark_price": "51000.00",
        "initial_margin": "7500.00",
        "maintenance_margin": "3750.00",
        "leverage": "10",
        "type": "CROSS",
        "create_time": 1587523073344,
        "update_time": 1613580710768
      }
    ]
  }
}
```

**Position Fields:**
- `account_id` - User account
- `instrument_name` - Trading pair
- `quantity` - Position size (positive = long, negative = short)
- `cost` - Position cost basis
- `open_position_pnl` - Unrealized PnL
- `session_pnl` - Session PnL
- `entry_price` - Average entry price
- `mark_price` - Current mark price
- `initial_margin` - IM requirement
- `maintenance_margin` - MM requirement
- `leverage` - Position leverage
- `type` - CROSS or ISOLATED
- `create_time` - Position open time
- `update_time` - Last update time

---

## Common Field Types

### Numeric Strings

All numeric values are strings wrapped in double quotes:

```json
{
  "price": "50000.00",
  "quantity": "0.5000",
  "fee": "0.25"
}
```

**Never:**
```json
{
  "price": 50000.00  // WRONG
}
```

---

### Timestamps

All timestamps are integers in milliseconds since Unix epoch:

```json
{
  "create_time": 1613580710768,
  "t": 1613580710768
}
```

---

### Empty Values

Empty strings for optional fields:

```json
{
  "client_oid": "",
  "trigger_price": "",
  "isolation_id": ""
}
```

---

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 0 | Success | Request successful |
| 10001 | INVALID_REQUEST | Malformed request |
| 10002 | INVALID_PARAMETERS | Invalid parameters |
| 10003 | INVALID_SIGNATURE | Authentication failed |
| 10004 | INVALID_NONCE | Duplicate/old nonce |
| 10005 | INVALID_API_KEY | API key not found |
| 10006 | IP_NOT_WHITELISTED | IP not allowed |
| 10007 | THROTTLE_REACHED | Rate limit exceeded |
| 10008 | PERMISSION_DENIED | Insufficient permissions |
| 20001 | DUPLICATE_ORDER | Duplicate client_oid |
| 20002 | INSUFFICIENT_BALANCE | Not enough balance |
| 20003 | INVALID_ORDER_PRICE | Price out of bounds |
| 20004 | INVALID_ORDER_QUANTITY | Quantity invalid |
| 20005 | ORDER_NOT_FOUND | Order does not exist |
| 20006 | MAX_ORDERS_EXCEEDED | Too many open orders |
| 30001 | MARKET_UNAVAILABLE | Market halted/suspended |
| 30002 | INSTRUMENT_NOT_FOUND | Invalid instrument |

---

## Response Size Limits

- **Order Book:** Max 50 levels per side
- **Candlesticks:** Max 1000 candles per request
- **Trades:** Max 100 trades per request (use pagination)
- **Order History:** Max 100 orders per page
- **User Trades:** Max 100 trades per page

---

## Pagination

List endpoints support pagination:

```json
{
  "params": {
    "page": 0,
    "page_size": 100
  }
}
```

**Response:**
```json
{
  "result": {
    "count": 250,
    "order_list": [...]
  }
}
```

Calculate total pages: `total_pages = ceil(count / page_size)`
