# Crypto.com Exchange API v1 - Endpoints Reference

## Base URLs

**Production:**
- REST API: `https://api.crypto.com/exchange/v1/{method}`
- WebSocket User API: `wss://stream.crypto.com/exchange/v1/user`
- WebSocket Market Data: `wss://stream.crypto.com/exchange/v1/market`

**UAT Sandbox:**
- REST API: `https://uat-api.3ona.co/exchange/v1/{method}`
- WebSocket User API: `wss://uat-stream.3ona.co/exchange/v1/user`
- WebSocket Market Data: `wss://uat-stream.3ona.co/exchange/v1/market`

**Headers:** All REST requests require `Content-Type: application/json`

---

## MarketData Trait Endpoints

### 1. Get Instruments
**Endpoint:** `public/get-instruments`
**Method:** GET/POST
**Auth:** No
**Description:** List all tradable instruments with specifications

**Request Parameters:**
```json
{
  "id": 1,
  "method": "public/get-instruments",
  "nonce": 1587523073344
}
```

**Response Fields:**
- `instrument_name` - Symbol identifier
- `quote_currency` - Quote asset
- `base_currency` - Base asset
- `price_decimals` - Price precision
- `quantity_decimals` - Quantity precision
- `margin_trading_enabled` - Margin support flag
- `max_quantity` - Maximum order size
- `min_quantity` - Minimum order size

---

### 2. Get Order Book
**Endpoint:** `public/get-book`
**Method:** GET/POST
**Auth:** No
**Description:** Fetch order book snapshot

**Request Parameters:**
```json
{
  "id": 1,
  "method": "public/get-book",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "depth": 10
  },
  "nonce": 1587523073344
}
```

**Parameters:**
- `instrument_name` (required) - Trading pair symbol
- `depth` (optional) - Number of levels per side (default: 50, max: 50)

**Response:**
```json
{
  "code": 0,
  "method": "public/get-book",
  "result": {
    "depth": 10,
    "instrument_name": "BTCUSD-PERP",
    "data": [{
      "asks": [["50126.000000", "0.400000", "0"]],
      "bids": [["50113.500000", "0.400000", "0"]]
    }]
  }
}
```

**Order Book Entry Format:** `[price, quantity, order_count]`

---

### 3. Get Candlestick
**Endpoint:** `public/get-candlestick`
**Method:** GET/POST
**Auth:** No
**Description:** Historical OHLCV candlestick data

**Request Parameters:**
```json
{
  "id": 1,
  "method": "public/get-candlestick",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "timeframe": "1h",
    "start_ts": 1613580710000,
    "end_ts": 1613667110000
  },
  "nonce": 1587523073344
}
```

**Parameters:**
- `instrument_name` (required) - Trading pair
- `timeframe` (optional) - Candle interval (default: `M1`)
- `start_ts` (optional) - Start timestamp (milliseconds)
- `end_ts` (optional) - End timestamp (milliseconds)

**Supported Timeframes:**
| Modern | Legacy | Duration |
|--------|--------|----------|
| `1m` | `M1` | 1 minute |
| `5m` | `M5` | 5 minutes |
| `15m` | `M15` | 15 minutes |
| `30m` | `M30` | 30 minutes |
| `1h` | `H1` | 1 hour |
| `2h` | `H2` | 2 hours |
| `4h` | `H4` | 4 hours |
| `12h` | `H12` | 12 hours |
| `1D` | `D1`/`1d` | 1 day |
| `7D` | — | 1 week |
| `14D` | — | 2 weeks |
| `1M` | — | 1 month |

**Response Fields:**
- `t` - Timestamp
- `o` - Open price
- `h` - High price
- `l` - Low price
- `c` - Close price
- `v` - Volume

---

### 4. Get Recent Trades
**Endpoint:** `public/get-trades`
**Method:** GET/POST
**Auth:** No
**Description:** Recent public trades

**Request Parameters:**
```json
{
  "id": 1,
  "method": "public/get-trades",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "count": 100
  },
  "nonce": 1587523073344
}
```

**Parameters:**
- `instrument_name` (required) - Trading pair
- `count` (optional) - Number of trades to return

**Response Fields:**
- `trade_id` - Unique trade identifier
- `side` - Trade side (BUY/SELL)
- `price` - Execution price
- `quantity` - Trade quantity
- `timestamp` - Execution time

---

### 5. Get Tickers
**Endpoint:** `public/get-tickers`
**Method:** GET/POST
**Auth:** No
**Description:** Market ticker data for all or specific instruments

**Request Parameters:**
```json
{
  "id": 1,
  "method": "public/get-tickers",
  "params": {
    "instrument_name": "BTCUSD-PERP"
  },
  "nonce": 1587523073344
}
```

**Parameters:**
- `instrument_name` (optional) - Specific instrument (omit for all)

**Response:**
```json
{
  "id": -1,
  "method": "public/get-tickers",
  "code": 0,
  "result": {
    "data": [{
      "h": "51790.00",
      "l": "47895.50",
      "a": "51174.500000",
      "i": "BTCUSD-PERP",
      "v": "879.5024",
      "vv": "26370000.12",
      "oi": "12345.12",
      "c": "0.03955106",
      "b": "51170.000000",
      "k": "51180.000000",
      "t": 1613580710768
    }]
  }
}
```

**Response Fields:**
- `i` - Instrument name
- `b` - Best bid price
- `k` - Best ask price
- `a` - Last traded price
- `c` - 24h price change
- `h` - 24h high
- `l` - 24h low
- `v` - 24h volume (base currency)
- `vv` - 24h volume (quote currency)
- `oi` - Open interest
- `t` - Timestamp

---

### 6. Get Valuations
**Endpoint:** `public/get-valuations`
**Method:** GET/POST
**Auth:** No
**Description:** Index price, mark price, and funding rate data

**Response Fields:**
- `instrument_name` - Trading pair
- `index_price` - Index price
- `mark_price` - Mark price used for unrealized PnL
- `funding_rate` - Current funding rate
- `next_funding_time` - Next funding timestamp

---

### 7. Get Expired Settlement Price
**Endpoint:** `public/get-expired-settlement-price`
**Method:** GET/POST
**Auth:** No
**Description:** Settlement prices for expired futures contracts

---

### 8. Get Insurance
**Endpoint:** `public/get-insurance`
**Method:** GET/POST
**Auth:** No
**Description:** Insurance fund balance

---

### 9. Get Announcements
**Endpoint:** `public/get-announcements`
**Method:** GET/POST
**Auth:** No
**Description:** System announcements and maintenance notices

---

### 10. Get Risk Parameters
**Endpoint:** `public/get-risk-parameters`
**Method:** GET/POST
**Auth:** No
**Description:** Risk settings, leverage limits, and margin requirements

---

## Trading Trait Endpoints

### 1. Create Order
**Endpoint:** `private/create-order`
**Method:** POST
**Auth:** Yes
**Description:** Place new order

**Request:**
```json
{
  "id": 1,
  "method": "private/create-order",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "side": "BUY",
    "type": "LIMIT",
    "price": "50000.00",
    "quantity": "0.5",
    "time_in_force": "GOOD_TILL_CANCEL",
    "client_oid": "client_order_id_123"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Order Types:**
- `LIMIT` - Requires `price` and `quantity`
- `MARKET` - Requires `quantity` or `notional` (for BUY)
- `STOP_LOSS` - Requires `ref_price`, optional `notional`
- `STOP_LIMIT` - Requires `price` and `ref_price`
- `TAKE_PROFIT` - Requires `ref_price`, optional `notional`
- `TAKE_PROFIT_LIMIT` - Requires `price` and `ref_price`

**Order Sides:**
- `BUY`
- `SELL`

**Time in Force:**
- `GOOD_TILL_CANCEL` - Remains until canceled
- `IMMEDIATE_OR_CANCEL` - Fill immediately or cancel
- `FILL_OR_KILL` - Fill entire order or cancel

**Execution Instructions (`exec_inst` array):**
- `POST_ONLY` - Only add liquidity (must use `GOOD_TILL_CANCEL`)
- `SMART_POST_ONLY` - Smart post-only logic
- `ISOLATED_MARGIN` - Use isolated margin

**Reference Price Type (`ref_price_type`):**
- `MARK_PRICE` (default)
- `INDEX_PRICE`
- `LAST_PRICE`

**Response:**
```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 0,
  "result": {
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02",
    "order_id": "18342311"
  }
}
```

---

### 2. Amend Order
**Endpoint:** `private/amend-order`
**Method:** POST
**Auth:** Yes
**Description:** Modify existing order price/quantity

**Request:**
```json
{
  "id": 1,
  "method": "private/amend-order",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "order_id": "18342311",
    "price": "50100.00",
    "quantity": "0.6"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 3. Cancel Order
**Endpoint:** `private/cancel-order`
**Method:** POST
**Auth:** Yes
**Description:** Cancel single order

**Request:**
```json
{
  "id": 1,
  "method": "private/cancel-order",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "order_id": "18342311"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Response:**
```json
{
  "id": 1,
  "method": "private/cancel-order",
  "code": 0,
  "message": "NO_ERROR",
  "result": {
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02",
    "order_id": "18342311"
  }
}
```

---

### 4. Cancel All Orders
**Endpoint:** `private/cancel-all-orders`
**Method:** POST
**Auth:** Yes
**Description:** Cancel all orders for specific instrument

**Request:**
```json
{
  "id": 1,
  "method": "private/cancel-all-orders",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 5. Get Open Orders
**Endpoint:** `private/get-open-orders`
**Method:** POST
**Auth:** Yes
**Description:** List active orders

**Request:**
```json
{
  "id": 1,
  "method": "private/get-open-orders",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "page": 0,
    "page_size": 100
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Parameters:**
- `instrument_name` (optional) - Filter by instrument
- `page` (optional) - Page number (default: 0)
- `page_size` (optional) - Results per page (default: 100)

**Response:**
```json
{
  "id": 1,
  "method": "private/get-open-orders",
  "code": 0,
  "result": {
    "count": 1,
    "order_list": [{
      "order_id": "18342311",
      "client_oid": "client_order_id_123",
      "instrument_name": "BTCUSD-PERP",
      "side": "BUY",
      "type": "LIMIT",
      "price": "50000.00",
      "quantity": "0.5",
      "cumulative_quantity": "0.0",
      "avg_price": "0.0",
      "status": "ACTIVE",
      "time_in_force": "GOOD_TILL_CANCEL",
      "exec_inst": [],
      "create_time": 1587523073344,
      "update_time": 1587523073344,
      "isolation_id": "",
      "isolation_type": ""
    }]
  }
}
```

---

### 6. Get Order Detail
**Endpoint:** `private/get-order-detail`
**Method:** POST
**Auth:** Yes
**Description:** Fetch specific order details

**Request:**
```json
{
  "id": 1,
  "method": "private/get-order-detail",
  "api_key": "your_api_key",
  "params": {
    "order_id": "18342311"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 7. Get Order History
**Endpoint:** `private/get-order-history`
**Method:** POST
**Auth:** Yes
**Description:** Historical orders (6-month window)

**Request:**
```json
{
  "id": 1,
  "method": "private/get-order-history",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "start_ts": 1613580710000,
    "end_ts": 1613667110000,
    "page": 0,
    "page_size": 100
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Parameters:**
- `instrument_name` (optional)
- `start_ts` (optional) - Start timestamp (ms)
- `end_ts` (optional) - End timestamp (ms)
- `page` (optional) - Default: 0
- `page_size` (optional) - Default: 100

---

### 8. Get Trades
**Endpoint:** `private/get-trades`
**Method:** POST
**Auth:** Yes
**Description:** User's trade execution history

**Request:**
```json
{
  "id": 1,
  "method": "private/get-trades",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "start_ts": 1613580710000,
    "end_ts": 1613667110000,
    "page": 0,
    "page_size": 100
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

## Account Trait Endpoints

### 1. Get User Balance
**Endpoint:** `private/user-balance`
**Method:** POST
**Auth:** Yes
**Description:** Wallet balance and margin information

**Request:**
```json
{
  "id": 1,
  "method": "private/user-balance",
  "api_key": "your_api_key",
  "params": {},
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Response Fields:**
- `total_available_balance` - Available for trading
- `total_margin_balance` - Total margin (no haircut)
- `total_initial_margin` - Total IM requirement
- `total_maintenance_margin` - Total MM requirement
- `total_position_cost` - Total position notional
- `total_cash_balance` - Total cash
- `total_collateral_value` - Total collateral
- `total_session_unrealized_pnl` - Session unrealized PnL
- `total_isolated_cash_balance` - Isolated margin cash
- `instrument_collateral_list` - Per-asset balances
- `isolated_positions` - Isolated margin positions

**Per-Asset Balance Fields:**
- `instrument_name` - Asset symbol
- `quantity` - Available quantity
- `reserved_qty` - Reserved in orders
- `locked_qty` - Locked quantity
- `last_price` - Latest price
- `collateral_weight` - Haircut coefficient

---

### 2. Get User Balance History
**Endpoint:** `private/user-balance-history`
**Method:** POST
**Auth:** Yes
**Description:** Historical balance snapshots

**Request:**
```json
{
  "id": 1,
  "method": "private/user-balance-history",
  "api_key": "your_api_key",
  "params": {
    "timeframe": "D1",
    "end_ts": 1613667110000,
    "limit": 100
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 3. Get Accounts
**Endpoint:** `private/get-accounts`
**Method:** POST
**Auth:** Yes
**Description:** Master and sub-account details

---

### 4. Get Fee Rate
**Endpoint:** `private/get-fee-rate`
**Method:** POST
**Auth:** Yes
**Description:** Account-level fee rates

**Response Fields:**
- `maker_fee_rate` - Maker fee percentage
- `taker_fee_rate` - Taker fee percentage
- `volume_tier` - VIP tier level

---

### 5. Get Instrument Fee Rate
**Endpoint:** `private/get-instrument-fee-rate`
**Method:** POST
**Auth:** Yes
**Description:** Instrument-specific fees

**Request:**
```json
{
  "id": 1,
  "method": "private/get-instrument-fee-rate",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 6. Get Transactions
**Endpoint:** `private/get-transactions`
**Method:** POST
**Auth:** Yes
**Description:** All account transactions (deposits, withdrawals, trades, fees)

**Request:**
```json
{
  "id": 1,
  "method": "private/get-transactions",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "start_ts": 1613580710000,
    "end_ts": 1613667110000,
    "page": 0,
    "page_size": 100
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

## Positions Trait Endpoints

### 1. Get Positions
**Endpoint:** `private/get-positions`
**Method:** POST
**Auth:** Yes
**Description:** Current open positions

**Request:**
```json
{
  "id": 1,
  "method": "private/get-positions",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Response Fields:**
- `instrument_name` - Trading pair
- `quantity` - Position size (positive = long, negative = short)
- `cost` - Position cost
- `open_position_pnl` - Unrealized PnL
- `session_pnl` - Session PnL
- `entry_price` - Average entry price
- `mark_price` - Current mark price
- `initial_margin` - IM requirement
- `maintenance_margin` - MM requirement
- `leverage` - Position leverage
- `type` - Position type (ISOLATED/CROSS)

---

### 2. Close Position
**Endpoint:** `private/close-position`
**Method:** POST
**Auth:** Yes
**Description:** Close open position at market

**Request:**
```json
{
  "id": 1,
  "method": "private/close-position",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "type": "MARKET"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

## Margin & Leverage Management

### 1. Change Account Leverage
**Endpoint:** `private/change-account-leverage`
**Method:** POST
**Auth:** Yes
**Description:** Set account-level leverage

**Request:**
```json
{
  "id": 1,
  "method": "private/change-account-leverage",
  "api_key": "your_api_key",
  "params": {
    "leverage": "10"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 2. Change Isolated Margin Leverage
**Endpoint:** `private/change-isolated-margin-leverage`
**Method:** POST
**Auth:** Yes
**Description:** Adjust isolated position leverage

**Request:**
```json
{
  "id": 1,
  "method": "private/change-isolated-margin-leverage",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "leverage": "20"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 3. Create Isolated Margin Transfer
**Endpoint:** `private/create-isolated-margin-transfer`
**Method:** POST
**Auth:** Yes
**Description:** Transfer funds to/from isolated margin position

**Request:**
```json
{
  "id": 1,
  "method": "private/create-isolated-margin-transfer",
  "api_key": "your_api_key",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "currency": "USDT",
    "amount": "100.00",
    "direction": "IN"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

**Direction:**
- `IN` - Transfer to isolated margin
- `OUT` - Transfer from isolated margin

---

## Sub-Account Management

### 1. Create Sub-Account Transfer
**Endpoint:** `private/create-subaccount-transfer`
**Method:** POST
**Auth:** Yes
**Description:** Transfer between master and sub-accounts

---

### 2. Get Sub-Account Balances
**Endpoint:** `private/get-subaccount-balances`
**Method:** POST
**Auth:** Yes
**Description:** All sub-account balances

---

## Wallet Endpoints

### 1. Get Deposit Address
**Endpoint:** `private/get-deposit-address`
**Method:** POST
**Auth:** Yes
**Description:** Crypto deposit addresses

**Request:**
```json
{
  "id": 1,
  "method": "private/get-deposit-address",
  "api_key": "your_api_key",
  "params": {
    "currency": "BTC"
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 2. Get Deposit History
**Endpoint:** `private/get-deposit-history`
**Method:** POST
**Auth:** Yes
**Description:** Deposit transaction records

---

### 3. Create Withdrawal
**Endpoint:** `private/create-withdrawal`
**Method:** POST
**Auth:** Yes
**Description:** Initiate crypto withdrawal

**Request:**
```json
{
  "id": 1,
  "method": "private/create-withdrawal",
  "api_key": "your_api_key",
  "params": {
    "currency": "BTC",
    "amount": "0.5",
    "address": "bc1q...",
    "address_tag": ""
  },
  "sig": "signature_here",
  "nonce": 1587523073344
}
```

---

### 4. Get Withdrawal History
**Endpoint:** `private/get-withdrawal-history`
**Method:** POST
**Auth:** Yes
**Description:** Withdrawal transaction records

---

### 5. Get Currency Networks
**Endpoint:** `private/get-currency-networks`
**Method:** POST
**Auth:** Yes
**Description:** Available withdrawal networks per currency

---

## Important Notes

1. **Max Open Orders:** 200 per instrument, 1,000 total per account
2. **Numeric Format:** All numbers must be strings wrapped in double quotes (e.g., `"12.34"`)
3. **Error Codes:** `code: 0` means success, non-zero indicates error
4. **Pagination:** Most list endpoints support `page` and `page_size` parameters
5. **Time Window:** Order history limited to 6-month windows
6. **Nonce:** Must be unique and incrementing (milliseconds recommended)
