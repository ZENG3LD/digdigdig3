# Gate.io API v4 Endpoints Research

**Research Date**: 2026-01-21
**API Version**: v4
**Documentation**: https://www.gate.com/docs/developers/apiv4/en/

---

## Table of Contents

- [Base URLs](#base-urls)
- [SPOT Endpoints](#spot-endpoints)
  - [Market Data](#spot-market-data)
  - [Trading](#spot-trading)
  - [Account](#spot-account)
- [FUTURES Endpoints](#futures-endpoints)
  - [Market Data](#futures-market-data)
  - [Trading](#futures-trading)
  - [Account](#futures-account)

---

## Base URLs

### Production (Mainnet)

| Environment | Type | Base URL |
|------------|------|----------|
| **Spot & Margin** | REST | `https://api.gateio.ws/api/v4` |
| **Futures USDT** | REST | `https://fx-api.gateio.ws/api/v4` |
| **Futures BTC** | REST | `https://fx-api.gateio.ws/api/v4` |
| **Alternative Spot** | REST | `https://api.gate.com/api/v4` |

**Note**: `api.gateio.ws` and `fx-api.gateio.ws` are the primary production URLs. `api.gate.com` redirects to `api.gateio.ws`.

### TestNet

| Type | Base URL |
|------|----------|
| **Spot REST** | `https://api-testnet.gateapi.io/api/v4` |
| **Futures REST** | `https://fx-api-testnet.gateio.ws/api/v4` |

---

## SPOT Endpoints

All endpoints are relative to: `https://api.gateio.ws/api/v4`

### SPOT Market Data

#### 1. Get Server Time

- **Endpoint**: `GET /spot/time`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/time`
- **Auth Required**: No (Public)
- **Parameters**: None
- **Description**: Get API server time in seconds
- **Response Example**:
  ```json
  {
    "server_time": 1729100692
  }
  ```
- **Notes**: Returns Unix timestamp in **seconds** (not milliseconds). Use for time synchronization.

---

#### 2. Get Ticker (Single or All Symbols)

- **Endpoint**: `GET /spot/tickers`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/tickers?currency_pair=BTC_USDT`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `currency_pair` (optional, string): Trading pair (e.g., "BTC_USDT"). If omitted, returns all tickers.
  - `timezone` (optional, string): Timezone for time range (default: "utc0")
- **Description**: Market ticker(s) with 24h statistics
- **Response Example** (single ticker):
  ```json
  [
    {
      "currency_pair": "BTC_USDT",
      "last": "48600.5",
      "lowest_ask": "48601.0",
      "highest_bid": "48600.0",
      "change_percentage": "2.5",
      "base_volume": "1234.567",
      "quote_volume": "60000000.00",
      "high_24h": "49000.0",
      "low_24h": "47500.0",
      "etf_net_value": null,
      "etf_pre_net_value": null,
      "etf_pre_timestamp": null,
      "etf_leverage": null
    }
  ]
  ```
- **Response Fields**:
  - `currency_pair`: Trading pair identifier
  - `last`: Last traded price
  - `lowest_ask`: Current best ask price
  - `highest_bid`: Current best bid price
  - `change_percentage`: 24h price change percentage (as string, e.g. "2.5" = 2.5%)
  - `base_volume`: 24h volume in base currency
  - `quote_volume`: 24h volume in quote currency
  - `high_24h`: Highest price in last 24 hours
  - `low_24h`: Lowest price in last 24 hours

**Note**: All numeric values are returned as **strings**.

---

#### 3. Get Orderbook

- **Endpoint**: `GET /spot/order_book`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/order_book?currency_pair=BTC_USDT&limit=100`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `currency_pair` (required, string): Trading pair (e.g., "BTC_USDT")
  - `interval` (optional, string): Price aggregation (0 = no aggregation, default: "0")
  - `limit` (optional, integer): Maximum depth levels (default: 10, max: 100)
  - `with_id` (optional, boolean): Return order book ID (default: false)
- **Description**: Market depth information
- **Response Example**:
  ```json
  {
    "id": 123456789,
    "current": 1623898993123,
    "update": 1623898993121,
    "asks": [
      ["48610.0", "0.5"],
      ["48615.0", "1.2"]
    ],
    "bids": [
      ["48600.0", "0.8"],
      ["48595.0", "2.1"]
    ]
  }
  ```
- **Response Fields**:
  - `id`: Order book update ID (optional, returned when `with_id=true`)
  - `current`: Timestamp when response was generated (milliseconds)
  - `update`: Timestamp when order book last changed (milliseconds)
  - `asks`: Array of [price, quantity] arrays (sell orders, sorted low to high)
  - `bids`: Array of [price, quantity] arrays (buy orders, sorted high to low)

**Note**: Top 1000 asks and bids are returned at maximum. Price and quantity are strings.

---

#### 4. Get Klines (Candlestick Data)

- **Endpoint**: `GET /spot/candlesticks`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/candlesticks?currency_pair=BTC_USDT&interval=1h&from=1566703297&to=1566789757`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `currency_pair` (required, string): Trading pair
  - `limit` (optional, integer): Number of data points (default: 100, max: 1000). **Conflicts with `from`/`to`**.
  - `from` (optional, integer): Start time (Unix timestamp in **seconds**)
  - `to` (optional, integer): End time (Unix timestamp in **seconds**, default: current time)
  - `interval` (optional, string): Candle interval (default: "30m")
- **Supported Intervals**:
  - `10s`, `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `8h`, `1d`, `7d`, `30d`
  - Note: `30d` = 1 natural month
- **Max Records**: 1000 per request
- **Response Example**:
  ```json
  [
    ["1566703320", "8533.02", "8553.74", "8550.24", "8527.17", "8553.74", "123.456"],
    ["1566703260", "8533.02", "8553.74", "8550.24", "8527.17", "8553.74", "789.012"]
  ]
  ```
- **Array Structure**: `[time, volume, close, high, low, open, quote_volume]`

| Index | Field | Type | Description |
|-------|-------|------|-------------|
| 0 | time | string | Unix timestamp in **seconds** |
| 1 | volume | string | Trading volume (base currency) |
| 2 | close | string | Closing price |
| 3 | high | string | Highest price |
| 4 | low | string | Lowest price |
| 5 | open | string | Opening price |
| 6 | quote_volume | string | Quote currency volume (optional) |

**CRITICAL**: Array order is **different** from most exchanges. Gate.io uses: `[time, volume, close, high, low, open, quote_volume]`

**Notes**:
- Time is in **seconds**, not milliseconds
- Most recent candle is first (descending order)
- All values are strings

---

#### 5. Get Currency Pairs (Symbols List)

- **Endpoint**: `GET /spot/currency_pairs`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/currency_pairs`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `currency_pair` (optional, string): Filter by specific pair
- **Description**: List of available trading pairs with trading rules
- **Response Example**:
  ```json
  [
    {
      "id": "BTC_USDT",
      "base": "BTC",
      "quote": "USDT",
      "fee": "0.2",
      "min_base_amount": "0.0001",
      "min_quote_amount": "1.0",
      "amount_precision": 4,
      "precision": 2,
      "trade_status": "tradable",
      "sell_start": 0,
      "buy_start": 0
    }
  ]
  ```
- **Response Fields**:
  - `id`: Currency pair ID (e.g., "BTC_USDT")
  - `base`: Base currency
  - `quote`: Quote currency
  - `fee`: Trading fee rate (percentage)
  - `min_base_amount`: Minimum order amount (base currency)
  - `min_quote_amount`: Minimum order amount (quote currency)
  - `amount_precision`: Amount decimal places
  - `precision`: Price decimal places
  - `trade_status`: Trading status ("tradable", "untradable")

---

### SPOT Trading

#### 1. Create Order (Place Order)

- **Endpoint**: `POST /spot/orders`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/orders`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Required Parameters** (JSON body):
  - `currency_pair` (string): Trading pair (e.g., "BTC_USDT")
  - `side` (enum): "buy" or "sell"
  - `amount` (string): Order amount (base currency)
  - `price` (optional, string): Order price (required for limit orders)
  - `type` (optional, enum): "limit" (default) or "market"
  - `account` (optional, enum): "spot" (default), "margin", "cross_margin"
  - `time_in_force` (optional, enum): "gtc" (good till cancelled, default), "ioc" (immediate or cancel), "poc" (pending or cancelled), "fok" (fill or kill)

- **Optional Parameters**:
  - `text` (string): User-defined order label (max 28 characters)
  - `iceberg` (string): Iceberg order amount (0 = not iceberg)
  - `auto_borrow` (boolean): Auto borrow for margin trading
  - `auto_repay` (boolean): Auto repay for margin trading

- **Example Request**:
  ```json
  {
    "currency_pair": "BTC_USDT",
    "side": "buy",
    "amount": "0.01",
    "price": "48000",
    "type": "limit",
    "time_in_force": "gtc",
    "text": "my-order-123"
  }
  ```

- **Response**:
  ```json
  {
    "id": "123456789",
    "text": "my-order-123",
    "create_time": "1729100692",
    "update_time": "1729100692",
    "create_time_ms": "1729100692123",
    "update_time_ms": "1729100692123",
    "currency_pair": "BTC_USDT",
    "status": "open",
    "type": "limit",
    "account": "spot",
    "side": "buy",
    "iceberg": "0",
    "amount": "0.01",
    "price": "48000",
    "time_in_force": "gtc",
    "left": "0.01",
    "filled_total": "0",
    "fee": "0",
    "fee_currency": "USDT",
    "point_fee": "0",
    "gt_fee": "0",
    "gt_discount": false,
    "rebated_fee": "0",
    "rebated_fee_currency": "USDT"
  }
  ```

---

#### 2. Cancel Order by ID

- **Endpoint**: `DELETE /spot/orders/{order_id}`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/orders/123456789?currency_pair=BTC_USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: DELETE
- **Path Parameters**:
  - `order_id` (string): Order ID to cancel
- **Query Parameters**:
  - `currency_pair` (required, string): Trading pair
  - `account` (optional, string): Account type (default: "spot")
- **Response**: Returns the cancelled order object (same structure as create order response)

---

#### 3. Get Order by ID

- **Endpoint**: `GET /spot/orders/{order_id}`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/orders/123456789?currency_pair=BTC_USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Path Parameters**:
  - `order_id` (string): Order ID to query
- **Query Parameters**:
  - `currency_pair` (required, string): Trading pair
  - `account` (optional, string): Account type (default: "spot")
- **Response**: Order object with full details (same structure as create order response)

---

#### 4. Get Open Orders (List Orders)

- **Endpoint**: `GET /spot/orders`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/orders?currency_pair=BTC_USDT&status=open`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `currency_pair` (required, string): Trading pair
  - `status` (required, enum): "open" (active orders) or "finished" (completed orders)
  - `page` (optional, integer): Page number (default: 1)
  - `limit` (optional, integer): Records per page (default: 100, max: 100)
  - `account` (optional, string): Account type (default: "spot")
  - `from` (optional, integer): Start timestamp (seconds)
  - `to` (optional, integer): End timestamp (seconds)
  - `side` (optional, enum): "buy" or "sell"

- **Response**: Array of order objects
  ```json
  [
    {
      "id": "123456789",
      "currency_pair": "BTC_USDT",
      "status": "open",
      ...
    }
  ]
  ```

**Notes**:
- For `status=open`: Returns all active orders (no time limit)
- For `status=finished`: Time range required, max 7 days

---

#### 5. Cancel All Orders

- **Endpoint**: `DELETE /spot/orders`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/orders?currency_pair=BTC_USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: DELETE
- **Query Parameters**:
  - `currency_pair` (required, string): Trading pair
  - `side` (optional, enum): "buy" or "sell" (if omitted, cancels both)
  - `account` (optional, string): Account type (default: "spot")

- **Response**: Array of cancelled order objects

---

### SPOT Account

#### 1. Get Accounts (List Balances)

- **Endpoint**: `GET /spot/accounts`
- **Full URL**: `https://api.gateio.ws/api/v4/spot/accounts?currency=BTC`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `currency` (optional, string): Filter by currency (e.g., "BTC")

- **Response Example**:
  ```json
  [
    {
      "currency": "BTC",
      "available": "1.2345",
      "locked": "0.0100"
    },
    {
      "currency": "USDT",
      "available": "10000.50",
      "locked": "500.00"
    }
  ]
  ```

- **Response Fields**:
  - `currency`: Currency code
  - `available`: Available balance for trading/withdrawal
  - `locked`: Balance locked in orders

- **Total Balance Formula**: `total = available + locked`

---

## FUTURES Endpoints

All futures endpoints are relative to: `https://fx-api.gateio.ws/api/v4`

The `{settle}` parameter in URLs can be:
- `usdt` - for USDT-margined perpetual contracts
- `btc` - for BTC-margined perpetual contracts

### FUTURES Market Data

#### 1. Get Server Time

- Same as Spot: `GET /spot/time` (works on futures base URL too)
- Base URL: `https://fx-api.gateio.ws/api/v4/spot/time`

---

#### 2. Get Ticker (Single or All Symbols)

- **Endpoint**: `GET /futures/{settle}/tickers`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/tickers?contract=BTC_USDT`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `contract` (optional, string): Contract name (e.g., "BTC_USDT"). If omitted, returns all tickers.
- **Description**: Futures market ticker(s)
- **Response Example**:
  ```json
  [
    {
      "contract": "BTC_USDT",
      "last": "48600.5",
      "change_percentage": "2.5",
      "volume_24h": "1234567",
      "volume_24h_btc": "25.5",
      "volume_24h_usd": "1200000",
      "volume_24h_base": "1234567",
      "volume_24h_quote": "60000000",
      "volume_24h_settle": "60000000",
      "mark_price": "48601.2",
      "funding_rate": "0.0001",
      "funding_rate_indicative": "0.00012",
      "index_price": "48599.8",
      "lowest_ask": "48601.0",
      "highest_bid": "48600.0",
      "high_24h": "49000.0",
      "low_24h": "47500.0"
    }
  ]
  ```

- **Additional Futures Fields**:
  - `mark_price`: Current mark price (for liquidation calculations)
  - `funding_rate`: Current funding rate
  - `funding_rate_indicative`: Predicted next funding rate
  - `index_price`: Index price from spot markets
  - `volume_24h_base`, `volume_24h_quote`, `volume_24h_settle`: Various volume measures

---

#### 3. Get Orderbook

- **Endpoint**: `GET /futures/{settle}/order_book`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/order_book?contract=BTC_USDT&limit=100`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `contract` (required, string): Contract name
  - `interval` (optional, string): Price aggregation (0 = no aggregation)
  - `limit` (optional, integer): Maximum depth levels (default: 10, max: 100)
  - `with_id` (optional, boolean): Return order book ID

- **Response**: Same structure as spot orderbook
  ```json
  {
    "id": 123456789,
    "current": 1623898993123,
    "update": 1623898993121,
    "asks": [["48610.0", 1000], ["48615.0", 1500]],
    "bids": [["48600.0", 800], ["48595.0", 2100]]
  }
  ```

**Note**: Futures orderbook quantity is in **number of contracts**, not base currency amount.

---

#### 4. Get Klines (Candlestick Data)

- **Endpoint**: `GET /futures/{settle}/candlesticks`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/candlesticks?contract=BTC_USDT&interval=1h&from=1566703297&to=1566789757`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `contract` (required, string): Contract name
  - `from` (optional, integer): Start time (Unix timestamp in **seconds**)
  - `to` (optional, integer): End time (Unix timestamp in **seconds**)
  - `limit` (optional, integer): Number of data points (default: 100, max: 2000)
  - `interval` (optional, string): Candle interval (default: "5m")

- **Supported Intervals**: Same as spot (`10s`, `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `8h`, `1d`, `7d`, `30d`)
- **Max Records**: 2000 per request (vs 1000 for spot)

- **Special Candle Types** (prefix to `contract` parameter):
  - `mark_` - Mark price candles (e.g., `mark_BTC_USDT`)
  - `index_` - Index price candles (e.g., `index_BTC_USDT`)

- **Response**: Same structure as spot klines `[time, volume, close, high, low, open]`

---

#### 5. Get Contracts Info (Symbols List)

- **Endpoint**: `GET /futures/{settle}/contracts`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/contracts`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `contract` (optional, string): Filter by specific contract
  - `limit` (optional, integer): Max contracts to return
  - `offset` (optional, integer): Offset for pagination

- **Response Example**:
  ```json
  [
    {
      "name": "BTC_USDT",
      "type": "direct",
      "quanto_multiplier": "0.0001",
      "leverage_min": "1",
      "leverage_max": "100",
      "maintenance_rate": "0.005",
      "mark_type": "index",
      "mark_price": "48600.5",
      "index_price": "48599.8",
      "last_price": "48600.0",
      "maker_fee_rate": "0.0002",
      "taker_fee_rate": "0.0006",
      "order_price_round": "0.1",
      "mark_price_round": "0.1",
      "funding_rate": "0.0001",
      "funding_interval": 28800,
      "funding_next_apply": 1729129200,
      "risk_limit_base": "1000000",
      "risk_limit_step": "500000",
      "risk_limit_max": "8000000",
      "order_size_min": 1,
      "order_size_max": 1000000,
      "order_price_deviate": "0.5",
      "ref_discount_rate": "0",
      "ref_rebate_rate": "0.2",
      "orderbook_id": 123456789,
      "trade_id": 987654321,
      "trade_size": 100,
      "position_size": 5000,
      "orders_limit": 50,
      "enable_bonus": true,
      "enable_credit": true,
      "create_time": 1546905600,
      "funding_cap_ratio": "0.005",
      "in_delisting": false
    }
  ]
  ```

- **Key Fields**:
  - `name`: Contract symbol (e.g., "BTC_USDT")
  - `type`: "direct" (linear) or "inverse"
  - `quanto_multiplier`: Contract size multiplier
  - `leverage_min`, `leverage_max`: Leverage range
  - `maintenance_rate`: Maintenance margin rate
  - `funding_rate`: Current funding rate
  - `funding_interval`: Funding interval in seconds (typically 28800 = 8 hours)
  - `order_size_min`, `order_size_max`: Order size limits
  - `maker_fee_rate`, `taker_fee_rate`: Trading fees

---

#### 6. Get Funding Rate

- **Endpoint**: `GET /futures/{settle}/funding_rate`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/funding_rate?contract=BTC_USDT&limit=100`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `contract` (required, string): Contract name
  - `limit` (optional, integer): Max records to return (default: 100, max: 1000)

- **Response Example**:
  ```json
  [
    {
      "t": 1729129200,
      "r": "0.0001"
    },
    {
      "t": 1729140800,
      "r": "0.00012"
    }
  ]
  ```

- **Response Fields**:
  - `t`: Funding time (Unix timestamp in **seconds**)
  - `r`: Funding rate (decimal, e.g., 0.0001 = 0.01%)

**Note**: Returns historical funding rates in descending order (newest first).

---

### FUTURES Trading

#### 1. Create Order (Place Order)

- **Endpoint**: `POST /futures/{settle}/orders`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/orders`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Required Parameters** (JSON body):
  - `contract` (string): Contract name (e.g., "BTC_USDT")
  - `size` (integer): Order size (number of contracts, positive for long, negative for short)
  - `price` (optional, string): Order price (required for limit orders, "0" for market orders)
  - `tif` (optional, enum): Time in force - "gtc" (default), "ioc", "poc", "fok"
  - `text` (optional, string): User-defined order label (max 28 characters)
  - `iceberg` (optional, integer): Iceberg order visible size (0 = not iceberg)
  - `reduce_only` (optional, boolean): Reduce-only order (default: false)
  - `close` (optional, boolean): Close position order (default: false)
  - `auto_size` (optional, string): "close_long", "close_short" - auto calculate size to close position

- **Example Request (Limit Order)**:
  ```json
  {
    "contract": "BTC_USDT",
    "size": 10,
    "price": "48000",
    "tif": "gtc",
    "text": "my-order-123",
    "reduce_only": false
  }
  ```

- **Example Request (Market Order)**:
  ```json
  {
    "contract": "BTC_USDT",
    "size": -5,
    "price": "0",
    "tif": "ioc"
  }
  ```

- **Response**: Same structure as spot order response
  ```json
  {
    "id": 123456789,
    "user": 123456,
    "create_time": 1729100692.123,
    "finish_time": 0,
    "finish_as": "",
    "status": "open",
    "contract": "BTC_USDT",
    "size": 10,
    "price": "48000",
    "fill_price": "0",
    "left": 10,
    "text": "my-order-123",
    "tkfr": "0.0006",
    "mkfr": "0.0002",
    "refu": 0,
    "is_reduce_only": false,
    "is_close": false,
    "is_liq": false,
    "tif": "gtc",
    "iceberg": 0
  }
  ```

**Order Status Values**:
- `open` - Active, waiting to be matched
- `finished` - Completed (filled or cancelled)
- `cancelled` - Cancelled by user
- `liquidating` - Being liquidated
- `liquidated` - Liquidated

---

#### 2. Cancel Order by ID

- **Endpoint**: `DELETE /futures/{settle}/orders/{order_id}`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/orders/123456789`
- **Auth Required**: Yes (Private)
- **HTTP Method**: DELETE
- **Path Parameters**:
  - `order_id` (integer): Order ID to cancel

- **Response**: Returns the cancelled order object

---

#### 3. Get Order by ID

- **Endpoint**: `GET /futures/{settle}/orders/{order_id}`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/orders/123456789`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Path Parameters**:
  - `order_id` (integer): Order ID to query

- **Response**: Order object with full details

---

#### 4. Get Open Orders (List Orders)

- **Endpoint**: `GET /futures/{settle}/orders`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/orders?contract=BTC_USDT&status=open`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `contract` (required, string): Contract name
  - `status` (required, enum): "open" or "finished"
  - `limit` (optional, integer): Max records (default: 100, max: 100)
  - `offset` (optional, integer): Pagination offset (default: 0)
  - `last_id` (optional, string): Last order ID for pagination
  - `count_total` (optional, integer): Whether to return total count (0 or 1)

- **Response**: Array of order objects
  ```json
  [
    {
      "id": 123456789,
      "contract": "BTC_USDT",
      "status": "open",
      ...
    }
  ]
  ```

---

#### 5. Cancel All Orders

- **Endpoint**: `DELETE /futures/{settle}/orders`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/orders?contract=BTC_USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: DELETE
- **Query Parameters**:
  - `contract` (required, string): Contract name
  - `side` (optional, enum): "ask" (sell) or "bid" (buy) - if omitted, cancels both

- **Response**: Array of cancelled order objects

---

### FUTURES Account

#### 1. Get Account Balance

- **Endpoint**: `GET /futures/{settle}/accounts`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/accounts`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Parameters**: None

- **Response Example**:
  ```json
  {
    "user": 123456,
    "currency": "USDT",
    "total": "10000.5",
    "unrealised_pnl": "150.25",
    "position_margin": "500.0",
    "order_margin": "200.0",
    "available": "9150.25",
    "point": "0",
    "bonus": "0",
    "in_dual_mode": false,
    "enable_credit": true,
    "position_initial_margin": "0",
    "maintenance_margin": "250.0",
    "enable_evolved_classic": true,
    "history": {
      "dnw": "0",
      "pnl": "150.25",
      "fee": "5.50",
      "refr": "0",
      "fund": "2.10",
      "point_dnw": "0",
      "point_fee": "0",
      "point_refr": "0",
      "bonus_dnw": "0",
      "bonus_offset": "0"
    }
  }
  ```

- **Response Fields**:
  - `currency`: Settlement currency (e.g., "USDT")
  - `total`: Total account balance
  - `unrealised_pnl`: Unrealized profit/loss from all positions
  - `position_margin`: Margin used by positions
  - `order_margin`: Margin used by open orders
  - `available`: Available balance for trading
  - `maintenance_margin`: Required maintenance margin
  - `in_dual_mode`: Whether dual mode (hedge mode) is enabled

**Balance Formula**: `total = available + position_margin + order_margin - unrealised_pnl`

---

#### 2. Get Positions List

- **Endpoint**: `GET /futures/{settle}/positions`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/positions?contract=BTC_USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `contract` (optional, string): Filter by specific contract
  - `holding` (optional, boolean): Only return non-zero positions (default: false)

- **Response Example**:
  ```json
  [
    {
      "user": 123456,
      "contract": "BTC_USDT",
      "size": 10,
      "leverage": "10",
      "risk_limit": "1000000",
      "leverage_max": "100",
      "maintenance_rate": "0.005",
      "value": "4860.5",
      "margin": "486.05",
      "entry_price": "48600.5",
      "liq_price": "43740.45",
      "mark_price": "48605.2",
      "unrealised_pnl": "0.47",
      "realised_pnl": "-1.20",
      "history_pnl": "-1.20",
      "last_close_pnl": "0",
      "realised_point": "0",
      "history_point": "0",
      "adl_ranking": 3,
      "pending_orders": 2,
      "close_order": null,
      "mode": "single",
      "cross_leverage_limit": "0",
      "update_time": 1729100692
    }
  ]
  ```

- **Response Fields**:
  - `contract`: Contract symbol
  - `size`: Position size (positive = long, negative = short, 0 = no position)
  - `leverage`: Current leverage
  - `value`: Position value in quote currency
  - `margin`: Position margin
  - `entry_price`: Average entry price
  - `liq_price`: Liquidation price
  - `mark_price`: Current mark price
  - `unrealised_pnl`: Unrealized profit/loss
  - `realised_pnl`: Realized profit/loss from current position
  - `history_pnl`: Historical realized PnL
  - `adl_ranking`: Auto-deleveraging queue ranking (1-5, 5 = highest risk)
  - `mode`: Position mode ("single" for one-way, "dual" for hedge mode)

---

#### 3. Get Single Position

- **Endpoint**: `GET /futures/{settle}/positions/{contract}`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/positions/BTC_USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Path Parameters**:
  - `contract` (string): Contract name

- **Response**: Single position object (same structure as positions list item)

---

#### 4. Update Position Leverage

- **Endpoint**: `POST /futures/{settle}/positions/{contract}/leverage`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/positions/BTC_USDT/leverage`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Path Parameters**:
  - `contract` (string): Contract name
- **Request Body**:
  - `leverage` (string): New leverage value (e.g., "10")
  - `cross_leverage_limit` (optional, string): Cross margin leverage limit

- **Example Request**:
  ```json
  {
    "leverage": "20"
  }
  ```

- **Response**: Updated position object

**Notes**:
- Leverage can only be changed when position size is 0
- Different leverage modes: isolated margin vs cross margin

---

#### 5. Update Position Margin

- **Endpoint**: `POST /futures/{settle}/positions/{contract}/margin`
- **Full URL**: `https://fx-api.gateio.ws/api/v4/futures/usdt/positions/BTC_USDT/margin`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Path Parameters**:
  - `contract` (string): Contract name
- **Request Body**:
  - `change` (string): Margin change amount (positive to add, negative to remove)

- **Example Request**:
  ```json
  {
    "change": "100.5"
  }
  ```

- **Response**: Updated position object

---

## Summary

### Key Takeaways

1. **Base URLs**:
   - Spot: `https://api.gateio.ws/api/v4`
   - Futures: `https://fx-api.gateio.ws/api/v4`

2. **Symbol Format**:
   - Spot: `BASE_QUOTE` (e.g., "BTC_USDT") with underscore
   - Futures: `BASE_QUOTE` (e.g., "BTC_USDT") with underscore - same format!

3. **Timestamps**:
   - Server time: **seconds**
   - Klines time: **seconds**
   - Order timestamps: **seconds** (with milliseconds available in `*_ms` fields)
   - Orderbook timestamps: **milliseconds**

4. **Klines Array Order** (CRITICAL):
   - Gate.io: `[time, volume, close, high, low, open, quote_volume]`
   - Most exchanges: `[time, open, high, low, close, volume]`
   - **Must handle this difference!**

5. **Numeric Values**:
   - Most fields returned as **strings**
   - Need to parse to float/int

6. **Futures Specifics**:
   - `{settle}` can be `usdt` or `btc`
   - Order size in **number of contracts**, not base currency amount
   - Position size sign indicates direction (+ = long, - = short)

7. **Rate Limits**:
   - Public endpoints: By IP
   - Private endpoints: By UID
   - Order placement: 10 requests/second for spot, 100 requests/second for futures

---

## Sources

- [Gate.io API Documentation](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Spot Trading REST API](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Futures Trading REST API](https://www.gate.com/docs/futures/api/index.html)
- [GitHub - gateio/gateapi-python](https://github.com/gateio/gateapi-python)
- [GitHub - gateio/gateapi-go SpotApi](https://github.com/gateio/gateapi-go/blob/master/docs/SpotApi.md)
- [GitHub - gateio/gateapi-java FuturesApi](https://github.com/gateio/gateapi-java/blob/master/docs/FuturesApi.md)

---

**Research completed**: 2026-01-21
**Next steps**: Implement endpoints in `endpoints.rs` following KuCoin reference structure
