# Coinbase Advanced Trade API Endpoints Research

**Research Date**: 2026-01-20

This document contains comprehensive research on Coinbase's Advanced Trade API endpoints for Spot trading (note: Coinbase does not offer traditional futures contracts like other exchanges).

---

## Table of Contents

- [Base URLs](#base-urls)
- [SPOT Endpoints](#spot-endpoints)
  - [Market Data](#spot-market-data)
  - [Trading](#spot-trading)
  - [Account](#spot-account)
- [Public vs Private Endpoints](#public-vs-private-endpoints)
- [Summary](#summary)

---

## Base URLs

### Production (Mainnet)

| Environment | Type | Base URL |
|------------|------|----------|
| **Advanced Trade API** | REST | `https://api.coinbase.com/api/v3/brokerage` |
| **WebSocket Market Data** | WebSocket | `wss://advanced-trade-ws.coinbase.com` |
| **WebSocket User Data** | WebSocket | `wss://advanced-trade-ws-user.coinbase.com` |

### Sandbox (Testnet)

Coinbase does not provide a separate sandbox environment for Advanced Trade API. Testing must be done with small amounts on production or using the older Exchange API sandbox.

**Note**: All endpoints use the `/api/v3/brokerage/` prefix for REST calls.

---

## SPOT Endpoints

### SPOT Market Data

#### 1. Get Server Time

- **Endpoint**: `GET /time`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/time`
- **Auth Required**: No (Public)
- **Parameters**: None
- **Description**: Get API server time in RFC3339 format
- **Response Example**:
  ```json
  {
    "iso": "2023-10-26T10:05:30.123Z",
    "epochSeconds": "1698315930",
    "epochMillis": "1698315930123"
  }
  ```
- **Notes**: Recommended for time sync. Server-client time difference must be within 30 seconds for JWT validation.

---

#### 2. List Products (Trading Pairs)

- **Endpoint**: `GET /products`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/products`
- **Auth Required**: Yes (Private)
- **Query Parameters**:
  - `limit` (optional, integer): Max 1000
  - `offset` (optional, integer): Pagination offset
  - `product_type` (optional, enum): "SPOT", "FUTURE" (filter by product type)
  - `product_ids` (optional, array): Filter by specific product IDs
- **Description**: Get list of available trading pairs
- **Response Fields**:
  - `product_id`: Trading pair (e.g., "BTC-USD")
  - `price`: Current price
  - `price_percentage_change_24h`: 24h change percentage
  - `volume_24h`: 24h trading volume
  - `volume_percentage_change_24h`: 24h volume change
  - `base_increment`: Minimum order size increment
  - `quote_increment`: Minimum price increment
  - `quote_min_size`: Minimum order value (quote currency)
  - `quote_max_size`: Maximum order value (quote currency)
  - `base_min_size`: Minimum order size (base currency)
  - `base_max_size`: Maximum order size (base currency)
  - `base_name`: Base currency name
  - `quote_name`: Quote currency name
  - `is_disabled`: Trading disabled flag
  - `new`: New product flag
  - `status`: Product status
  - `cancel_only`: Cancel only mode

**Public Alternative**: `GET /market/products` (no authentication required)

---

#### 3. Get Product Details (Single Product)

- **Endpoint**: `GET /products/{product_id}`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/products/BTC-USD`
- **Auth Required**: Yes (Private)
- **Path Parameters**:
  - `product_id` (required, string): Trading pair (e.g., "BTC-USD")
- **Description**: Get details for a single product
- **Response**: Same fields as List Products

**Public Alternative**: `GET /market/products/{product_id}` (no authentication required)

---

#### 4. Get Product Book (Order Book)

- **Endpoint**: `GET /product_book`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/product_book?product_id=BTC-USD&limit=10`
- **Auth Required**: Yes (Private)
- **Query Parameters**:
  - `product_id` (required, string): Trading pair
  - `limit` (optional, integer): Number of levels per side (default: 50, max: 500)
  - `aggregation_price_increment` (optional, string): Price aggregation level
- **Description**: Level 2 order book data
- **Response Fields**:
  - `product_id`: Trading pair
  - `bids`: Array of bid levels `[price, size, num_orders]`
  - `asks`: Array of ask levels `[price, size, num_orders]`
  - `time`: Timestamp (RFC3339)

**Public Alternative**: `GET /market/product_book` (no authentication required)

---

#### 5. Get Best Bid/Ask

- **Endpoint**: `GET /best_bid_ask`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/best_bid_ask?product_ids=BTC-USD,ETH-USD`
- **Auth Required**: Yes (Private)
- **Query Parameters**:
  - `product_ids` (optional, array): List of product IDs (comma-separated)
- **Description**: Get best bid/ask prices for products
- **Response Fields**:
  - `pricebooks`: Array of price book entries
    - `product_id`: Trading pair
    - `bids`: Best bid `[price, size]`
    - `asks`: Best ask `[price, size]`
    - `time`: Timestamp (RFC3339)

---

#### 6. Get Product Candles (Klines)

- **Endpoint**: `GET /products/{product_id}/candles`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/products/BTC-USD/candles?start=1609459200&end=1609545600&granularity=ONE_HOUR`
- **Auth Required**: Yes (Private)
- **Path Parameters**:
  - `product_id` (required, string): Trading pair
- **Query Parameters**:
  - `start` (required, string): Start time (Unix timestamp in seconds)
  - `end` (required, string): End time (Unix timestamp in seconds)
  - `granularity` (required, enum): Candle interval
- **Supported Granularities**:
  - `UNKNOWN_GRANULARITY` (default)
  - `ONE_MINUTE`
  - `FIVE_MINUTE`
  - `FIFTEEN_MINUTE`
  - `THIRTY_MINUTE`
  - `ONE_HOUR`
  - `TWO_HOUR`
  - `SIX_HOUR`
  - `ONE_DAY`
- **Max Records**: 300 per request
- **Response Array Format**:
  ```json
  {
    "candles": [
      {
        "start": "1639508050",
        "low": "48000.00",
        "high": "49000.00",
        "open": "48500.00",
        "close": "48800.00",
        "volume": "123.45"
      }
    ]
  }
  ```
- **Notes**:
  - `start` is Unix timestamp in seconds (string)
  - All price/volume fields are strings
  - Data sorted by time descending (newest first)
  - Request range limited to 300 candles

**Public Alternative**: `GET /market/products/{product_id}/candles` (no authentication required)

---

#### 7. Get Market Trades

- **Endpoint**: `GET /products/{product_id}/ticker`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/products/BTC-USD/ticker?limit=100`
- **Auth Required**: Yes (Private)
- **Path Parameters**:
  - `product_id` (required, string): Trading pair
- **Query Parameters**:
  - `limit` (required, integer): Number of trades (1-1000)
  - `start` (optional, string): Start time (RFC3339)
  - `end` (optional, string): End time (RFC3339)
- **Description**: Get recent market trades
- **Response Fields**:
  - `trades`: Array of trade objects
    - `trade_id`: Trade ID
    - `product_id`: Trading pair
    - `price`: Trade price
    - `size`: Trade size
    - `time`: Trade time (RFC3339)
    - `side`: Trade side ("BUY" or "SELL")
    - `bid`: Bid price at time of trade
    - `ask`: Ask price at time of trade

**Public Alternative**: `GET /market/products/{product_id}/ticker` (no authentication required)

---

### SPOT Trading

#### 1. Create Order

- **Endpoint**: `POST /orders`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders`
- **Auth Required**: Yes (Private) - Requires "trade" permission
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Request Body**:
  ```json
  {
    "client_order_id": "0000-00000-000000",
    "product_id": "BTC-USD",
    "side": "BUY",
    "order_configuration": {
      "market_market_ioc": {
        "quote_size": "1000.00"
      }
    }
  }
  ```
- **Required Parameters**:
  - `client_order_id` (string): Unique client-provided order ID
  - `product_id` (string): Trading pair
  - `side` (enum): "BUY" or "SELL"
  - `order_configuration` (object): Order configuration (one of):
    - `market_market_ioc`: Market order IOC
      - `quote_size` (string): Amount in quote currency (for BUY)
      - `base_size` (string): Amount in base currency (for SELL)
    - `limit_limit_gtc`: Limit order GTC
      - `base_size` (string): Order size
      - `limit_price` (string): Limit price
      - `post_only` (boolean): Post-only flag
    - `limit_limit_gtd`: Limit order GTD
      - `base_size`, `limit_price`, `end_time`, `post_only`
    - `limit_limit_fok`: Limit order FOK
      - `base_size`, `limit_price`
    - `stop_limit_stop_limit_gtc`: Stop-limit order
      - `base_size`, `limit_price`, `stop_price`, `stop_direction`
    - `stop_limit_stop_limit_gtd`: Stop-limit GTD
- **Optional Parameters**:
  - `retail_portfolio_id` (string): Portfolio ID
  - `leverage` (string): Leverage (for margin trading)
  - `margin_type` (enum): "CROSS" or "ISOLATED"
  - `self_trade_prevention_id` (string): STP ID
  - `preview_id` (string): Preview ID from preview endpoint
- **Response**:
  ```json
  {
    "success": true,
    "success_response": {
      "order_id": "11111-00000-000000",
      "product_id": "BTC-USD",
      "side": "BUY",
      "client_order_id": "0000-00000-000000"
    },
    "order_configuration": { ... }
  }
  ```

---

#### 2. Cancel Orders

- **Endpoint**: `POST /orders/batch_cancel`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders/batch_cancel`
- **Auth Required**: Yes (Private) - Requires "trade" permission
- **HTTP Method**: POST
- **Request Body**:
  ```json
  {
    "order_ids": [
      "11111-00000-000000",
      "11111-00000-000001"
    ]
  }
  ```
- **Response**:
  ```json
  {
    "results": [
      {
        "success": true,
        "order_id": "11111-00000-000000"
      }
    ]
  }
  ```

---

#### 3. Edit Order

- **Endpoint**: `POST /orders/edit`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders/edit`
- **Auth Required**: Yes (Private) - Requires "trade" permission
- **HTTP Method**: POST
- **Request Body**:
  ```json
  {
    "order_id": "11111-00000-000000",
    "price": "50000.00",
    "size": "0.5"
  }
  ```

---

#### 4. Get Order by ID

- **Endpoint**: `GET /orders/historical/{order_id}`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders/historical/11111-00000-000000`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Path Parameters**:
  - `order_id` (string): Order ID
- **Query Parameters**:
  - `client_order_id` (optional, string): Alternative lookup by client ID
  - `user_native_currency` (optional, string): Currency for total value
- **Response Fields**:
  - `order`: Order object
    - `order_id`: Server order ID
    - `product_id`: Trading pair
    - `user_id`: User ID
    - `order_configuration`: Order configuration object
    - `side`: "BUY" or "SELL"
    - `client_order_id`: Client order ID
    - `status`: Order status
    - `time_in_force`: Time in force
    - `created_time`: Creation time (RFC3339)
    - `completion_percentage`: Fill percentage
    - `filled_size`: Filled size
    - `average_filled_price`: Average fill price
    - `fee`: Trading fee
    - `number_of_fills`: Number of fills
    - `filled_value`: Total filled value
    - `pending_cancel`: Pending cancel flag
    - `size_in_quote`: Size in quote currency
    - `total_fees`: Total fees
    - `size_inclusive_of_fees`: Size including fees
    - `total_value_after_fees`: Total value after fees
    - `trigger_status`: Trigger status
    - `order_type`: Order type
    - `reject_reason`: Rejection reason
    - `settled`: Settlement status
    - `product_type`: Product type
    - `reject_message`: Rejection message
    - `cancel_message`: Cancellation message

---

#### 5. List Orders

- **Endpoint**: `GET /orders/historical/batch`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders/historical/batch?product_id=BTC-USD&order_status=OPEN`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `product_id` (optional, string): Filter by trading pair
  - `order_status` (optional, array): Filter by status ("OPEN", "FILLED", "CANCELLED", "EXPIRED", "FAILED")
  - `limit` (optional, integer): Max 1000
  - `start_date` (optional, string): Start date (RFC3339)
  - `end_date` (optional, string): End date (RFC3339)
  - `user_native_currency` (optional, string): Currency for values
  - `order_type` (optional, enum): "MARKET", "LIMIT", "STOP", "STOP_LIMIT"
  - `order_side` (optional, enum): "BUY", "SELL"
  - `cursor` (optional, string): Pagination cursor
  - `product_type` (optional, enum): "SPOT", "FUTURE"
  - `order_placement_source` (optional, enum): Source filter
  - `contract_expiry_type` (optional, enum): Contract expiry type
- **Response**:
  ```json
  {
    "orders": [ ... ],
    "sequence": "0",
    "has_next": true,
    "cursor": "next_page_cursor"
  }
  ```

---

#### 6. List Fills

- **Endpoint**: `GET /orders/historical/fills`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders/historical/fills?order_id=11111-00000-000000`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `order_id` (optional, string): Filter by order ID
  - `product_id` (optional, string): Filter by product
  - `start_sequence_timestamp` (optional, string): Start time (RFC3339)
  - `end_sequence_timestamp` (optional, string): End time (RFC3339)
  - `limit` (optional, integer): Max 1000
  - `cursor` (optional, string): Pagination cursor
- **Response Fields**:
  - `fills`: Array of fill objects
    - `entry_id`: Fill ID
    - `trade_id`: Trade ID
    - `order_id`: Order ID
    - `trade_time`: Trade time (RFC3339)
    - `trade_type`: Trade type
    - `price`: Fill price
    - `size`: Fill size
    - `commission`: Commission fee
    - `product_id`: Trading pair
    - `sequence_timestamp`: Sequence timestamp
    - `liquidity_indicator`: "MAKER" or "TAKER"
    - `size_in_quote`: Size in quote currency
    - `user_id`: User ID
    - `side`: "BUY" or "SELL"

---

#### 7. Preview Order

- **Endpoint**: `POST /orders/preview`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/orders/preview`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Description**: Preview order without placing it
- **Request Body**: Same as Create Order
- **Response**: Estimated order details including fees

---

### SPOT Account

#### 1. List Accounts

- **Endpoint**: `GET /accounts`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/accounts?limit=250`
- **Auth Required**: Yes (Private) - Requires "view" permission
- **HTTP Method**: GET
- **Query Parameters**:
  - `limit` (optional, integer): Max 250
  - `cursor` (optional, string): Pagination cursor
  - `retail_portfolio_id` (optional, string): Filter by portfolio
- **Response Example**:
  ```json
  {
    "accounts": [
      {
        "uuid": "8bfc20d7-f7c6-4422-bf07-8243ca4169fe",
        "name": "BTC Wallet",
        "currency": "BTC",
        "available_balance": {
          "value": "1.23",
          "currency": "BTC"
        },
        "default": true,
        "active": true,
        "created_at": "2021-05-31T09:59:59Z",
        "updated_at": "2021-05-31T09:59:59Z",
        "deleted_at": null,
        "type": "ACCOUNT_TYPE_CRYPTO",
        "ready": true,
        "hold": {
          "value": "0.1",
          "currency": "BTC"
        }
      }
    ],
    "has_next": true,
    "cursor": "next_page_cursor",
    "size": 1
  }
  ```
- **Account Fields**:
  - `uuid`: Account ID
  - `name`: Account name
  - `currency`: Currency code
  - `available_balance`: Available balance object
    - `value`: Balance value (string)
    - `currency`: Currency code
  - `default`: Default account flag
  - `active`: Active status
  - `created_at`: Creation time (RFC3339)
  - `updated_at`: Update time (RFC3339)
  - `deleted_at`: Deletion time (RFC3339, null if not deleted)
  - `type`: Account type
  - `ready`: Ready status
  - `hold`: Held/frozen balance object

---

#### 2. Get Account by ID

- **Endpoint**: `GET /accounts/{account_uuid}`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/accounts/8bfc20d7-f7c6-4422-bf07-8243ca4169fe`
- **Auth Required**: Yes (Private) - Requires "view" permission
- **HTTP Method**: GET
- **Path Parameters**:
  - `account_uuid` (string): Account UUID
- **Response**: Same as account object in List Accounts

---

#### 3. Get Transaction Summary

- **Endpoint**: `GET /transaction_summary`
- **Full URL**: `https://api.coinbase.com/api/v3/brokerage/transaction_summary?start_date=2021-01-01T00:00:00Z&end_date=2021-12-31T23:59:59Z`
- **Auth Required**: Yes (Private) - Requires "view" permission
- **HTTP Method**: GET
- **Query Parameters**:
  - `start_date` (optional, string): Start date (RFC3339)
  - `end_date` (optional, string): End date (RFC3339)
  - `user_native_currency` (optional, string): Currency for summary
  - `product_type` (optional, enum): "SPOT", "FUTURE"
  - `contract_expiry_type` (optional, enum): Contract expiry filter
- **Description**: Get fee tier and transaction summary
- **Response Fields**:
  - `total_volume`: Total trading volume
  - `total_fees`: Total fees paid
  - `fee_tier`: Current fee tier object
    - `pricing_tier`: Tier name
    - `usd_from`: Volume tier start
    - `usd_to`: Volume tier end
    - `taker_fee_rate`: Taker fee rate
    - `maker_fee_rate`: Maker fee rate
  - `margin_rate`: Margin rate object
  - `goods_and_services_tax`: Tax object
  - `advanced_trade_only_volume`: Advanced Trade volume
  - `advanced_trade_only_fees`: Advanced Trade fees
  - `coinbase_pro_volume`: Pro volume (deprecated)
  - `coinbase_pro_fees`: Pro fees (deprecated)

---

## Public vs Private Endpoints

### Public Endpoints (No Authentication)

Public endpoints are available under `/market/` prefix and do not require authentication:

- `GET /market/products` - List all products
- `GET /market/products/{product_id}` - Get product details
- `GET /market/product_book` - Get order book
- `GET /market/products/{product_id}/candles` - Get candles
- `GET /market/products/{product_id}/ticker` - Get market trades

### Private Endpoints (Authentication Required)

All other endpoints require JWT authentication and appropriate API key permissions:

- **"view" permission**: Account, portfolio, transaction data
- **"trade" permission**: Create, cancel, edit orders
- **"transfer" permission**: Portfolio transfers (not covered here)

---

## Summary

### Overall Status

Coinbase Advanced Trade API provides comprehensive REST endpoints for spot trading with modern authentication (JWT). Key characteristics:

1. **Base URL**: `https://api.coinbase.com/api/v3/brokerage`
2. **Authentication**: JWT (ES256) with 2-minute expiration
3. **Symbol Format**: `BASE-QUOTE` (e.g., "BTC-USD")
4. **Timestamp Format**: RFC3339 for most fields, Unix seconds for some query params
5. **Rate Limits**: 30 req/s (private), 10 req/s (public)
6. **Pagination**: Cursor-based
7. **Max Records**: 300 candles, 1000 orders/fills, 250 accounts

### Key Findings

1. **No Futures Trading**: Coinbase Advanced Trade API does not offer traditional perpetual or futures contracts (different from KuCoin/Binance)
2. **Order Configuration**: Complex order types use nested `order_configuration` objects
3. **RFC3339 Timestamps**: Most timestamps use RFC3339 format (not milliseconds)
4. **Public/Private Split**: Market data available via both public and private endpoints
5. **Granularity Enums**: Uses string enums (`ONE_MINUTE`, `FIVE_MINUTE`) not integers
6. **Response Structure**: No generic wrapper like KuCoin's `{code, data}` - direct response objects

### Differences from KuCoin

| Feature | KuCoin | Coinbase |
|---------|--------|----------|
| Futures Support | Yes (USDT, USD perpetuals) | No |
| Auth Method | HMAC-SHA256 + Base64 | JWT (ES256) |
| Timestamp Format | Milliseconds | RFC3339 + Unix seconds |
| Symbol Format | `BTC-USDT` (spot), `XBTUSDTM` (futures) | `BTC-USD` (spot only) |
| Rate Limits | 16,000/30s (VIP 5) | 30/s private, 10/s public |
| Kline Intervals | String (`1min`) or int (futures) | Enum (`ONE_MINUTE`) |
| Response Wrapper | `{code: "200000", data: {...}}` | Direct object |
| Error Codes | String codes (`429000`) | HTTP status codes |

---

## Sources

Research compiled from the following official sources:

- [Coinbase Advanced Trade API - Welcome](https://docs.cdp.coinbase.com/advanced-trade/docs/welcome)
- [Coinbase Advanced Trade API - Overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview)
- [Advanced Trade REST API Endpoints](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/rest-api)
- [Advanced Trade API - API Overview](https://docs.cdp.coinbase.com/advanced-trade/docs/api-overview/)
- [Coinbase API Key Authentication](https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication)
- [Create Order - Advanced Trade](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
- [List Orders - Advanced Trade](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/list-orders)
- [Get Order - Advanced Trade](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/get-order)
- [List Fills - Advanced Trade](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/list-fills)
- [Coinbase Advanced Python SDK](https://github.com/coinbase/coinbase-advanced-py)
- [Coinbase Advanced Python SDK Documentation](https://coinbase.github.io/coinbase-advanced-py/)
- [Coinbase API Cheat Sheet - Vezgo](https://vezgo.com/blog/coinbase-api-cheat-sheet-for-developers/)
- [How to Use Coinbase API - Apidog](https://apidog.com/blog/coinbase-api-5/)
