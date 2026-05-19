# BingX API Endpoints

Comprehensive documentation of BingX REST API endpoints for V5 connector implementation.

**Base URL:** `https://open-api.bingx.com`

---

## Market Data Endpoints (Public)

### Spot Market Data

#### Get Trading Symbols
- **Endpoint:** `GET /openApi/spot/v1/common/symbols`
- **Authentication:** Not required
- **Parameters:** None
- **Description:** Get all available spot trading symbols
- **Response:** List of trading pairs with configuration

#### Get Recent Trades
- **Endpoint:** `GET /openApi/spot/v1/market/trades`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, required) - Trading pair (e.g., "BTC-USDT")
  - `limit` (integer, optional) - Number of trades to return
- **Description:** Get recent trades for a symbol

#### Get Order Book (Depth)
- **Endpoint:** `GET /openApi/spot/v1/market/depth`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `limit` (integer, optional) - Depth limit (default: 20, max: 1000)
- **Description:** Get order book depth information

#### Get Kline/Candlestick Data
- **Endpoint:** `GET /openApi/spot/v1/market/kline`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `interval` (string, required) - Kline interval: `1min`, `5min`, `15min`, `30min`, `60min`, `1day`
  - `startTime` (long, optional) - Start time in milliseconds
  - `endTime` (long, optional) - End time in milliseconds
  - `limit` (integer, optional) - Number of klines (default: 500, max: 1440)
- **Description:** Get kline/candlestick data for a symbol

#### Get 24hr Ticker
- **Endpoint:** `GET /openApi/spot/v1/ticker/24hr`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair (if not provided, returns all symbols)
- **Description:** Get 24-hour price change statistics

#### Get Symbol Price Ticker
- **Endpoint:** `GET /openApi/spot/v1/ticker/price`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
- **Description:** Get latest price for symbol(s)

#### Get Best Bid/Ask Price
- **Endpoint:** `GET /openApi/spot/v1/ticker/bookTicker`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
- **Description:** Get best bid/ask prices

### Perpetual Swap (Futures) Market Data

#### Get Swap Contract Info
- **Endpoint:** `GET /openApi/swap/v2/quote/contracts`
- **Authentication:** Not required
- **Parameters:** None
- **Description:** Get all perpetual swap contract information

#### Get Swap Order Book
- **Endpoint:** `GET /openApi/swap/v2/quote/depth`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, required) - Trading pair (e.g., "BTC-USDT")
  - `limit` (integer, optional) - Depth limit (default: 20)
- **Description:** Get order book depth for perpetual swap

#### Get Swap Recent Trades
- **Endpoint:** `GET /openApi/swap/v2/quote/trades`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `limit` (integer, optional) - Number of trades
- **Description:** Get recent trades for perpetual swap

#### Get Swap Kline Data
- **Endpoint:** `GET /openApi/swap/v2/quote/klines`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `interval` (string, required) - Kline interval: `1min`, `5min`, `15min`, `30min`, `60min`, `1day`
  - `startTime` (long, optional) - Start time in milliseconds
  - `endTime` (long, optional) - End time in milliseconds
  - `limit` (integer, optional) - Number of klines (default: 500, max: 1440)
- **Description:** Get kline data for perpetual swap

#### Get Swap Ticker
- **Endpoint:** `GET /openApi/swap/v2/quote/ticker`
- **Authentication:** Not required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
- **Description:** Get 24hr ticker statistics

---

## Trading Endpoints (Authenticated)

### Spot Trading

#### Place Spot Order
- **Endpoint:** `POST /openApi/spot/v1/trade/order`
- **Authentication:** Required (API Key + Signature)
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `side` (string, required) - `BUY` or `SELL`
  - `type` (string, required) - Order type: `MARKET`, `LIMIT`
  - `quantity` (decimal, required for LIMIT) - Order quantity
  - `quoteOrderQty` (decimal, required for MARKET) - Quote quantity
  - `price` (decimal, required for LIMIT) - Order price
  - `timestamp` (long, required) - Request timestamp in milliseconds
  - `recvWindow` (long, optional) - Request validity window
- **Description:** Place a new spot order

#### Query Spot Order
- **Endpoint:** `GET /openApi/spot/v1/trade/order`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `orderId` (long, optional) - Order ID
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query spot order details

#### Cancel Spot Order
- **Endpoint:** `DELETE /openApi/spot/v1/trade/order`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `orderId` (long, required) - Order ID to cancel
  - `timestamp` (long, required) - Request timestamp
- **Description:** Cancel an active spot order

#### Get Open Orders
- **Endpoint:** `GET /openApi/spot/v1/trade/openOrders`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Get all open spot orders

#### Get Order History
- **Endpoint:** `GET /openApi/spot/v1/trade/historyOrders`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
  - `orderId` (long, optional) - Order ID
  - `startTime` (long, optional) - Start time
  - `endTime` (long, optional) - End time
  - `limit` (integer, optional) - Number of records
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query historical spot orders

#### Cancel All Orders
- **Endpoint:** `DELETE /openApi/spot/v1/trade/cancelAllOrders`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Cancel all open orders for a symbol

### Perpetual Swap Trading

#### Place Swap Order
- **Endpoint:** `POST /openApi/swap/v2/trade/order`
- **Authentication:** Required
- **Rate Limit:** 10 requests/second
- **Parameters:**
  - `symbol` (string, required) - Trading pair (e.g., "BTC-USDT")
  - `side` (string, required) - `BUY` or `SELL`
  - `positionSide` (string, optional) - `LONG` or `SHORT` (for hedge mode)
  - `type` (string, required) - Order type: `MARKET`, `LIMIT`, `STOP_MARKET`, `STOP`, `TAKE_PROFIT_MARKET`, `TAKE_PROFIT`, `TRAILING_STOP_MARKET`
  - `quantity` (decimal, required) - Order quantity
  - `price` (decimal, required for LIMIT orders) - Order price
  - `stopPrice` (decimal, required for STOP orders) - Trigger price
  - `priceRate` (decimal, optional) - Price rate for trailing stop
  - `activationPrice` (decimal, optional) - Activation price for trailing stop
  - `workingType` (string, optional) - `MARK_PRICE` or `CONTRACT_PRICE`
  - `timestamp` (long, required) - Request timestamp
  - `recvWindow` (long, optional) - Request validity window
- **Description:** Place a new perpetual swap order

#### Cancel Swap Order
- **Endpoint:** `DELETE /openApi/swap/v2/trade/order`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `orderId` (long, required) - Order ID
  - `timestamp` (long, required) - Request timestamp
- **Description:** Cancel an active swap order

#### Cancel All Swap Orders
- **Endpoint:** `DELETE /openApi/swap/v2/trade/allOrders`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Cancel all open orders for a symbol

#### Get Swap Order
- **Endpoint:** `GET /openApi/swap/v2/trade/order`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `orderId` (long, optional) - Order ID
  - `clientOrderID` (string, optional) - Client order ID
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query swap order details

#### Get Swap Open Orders
- **Endpoint:** `GET /openApi/swap/v2/trade/openOrders`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Get all open swap orders

#### Get Swap Order History
- **Endpoint:** `GET /openApi/swap/v2/trade/allOrders`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `orderId` (long, optional) - Order ID
  - `startTime` (long, optional) - Start time
  - `endTime` (long, optional) - End time
  - `limit` (integer, optional) - Number of records (max: 1000)
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query all swap orders (historical and current)

#### Close All Positions
- **Endpoint:** `POST /openApi/swap/v2/trade/closeAllPositions`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Close all positions for a symbol

---

## Account Endpoints (Authenticated)

### Spot Account

#### Get Spot Account Balance
- **Endpoint:** `GET /openApi/spot/v1/account/balance`
- **Authentication:** Required
- **Parameters:**
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query spot account balance

#### Get Commission Rate
- **Endpoint:** `GET /openApi/spot/v1/account/commissionRate`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query commission rate for spot trading

### Perpetual Swap Account

#### Get Swap Balance
- **Endpoint:** `GET /openApi/swap/v2/user/balance`
- **Authentication:** Required
- **Parameters:**
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query perpetual swap account balance

#### Get Swap Balance (V3)
- **Endpoint:** `GET /openApi/swap/v3/user/balance`
- **Authentication:** Required
- **Parameters:**
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query perpetual swap account balance (newer version)

#### Get Commission Rate
- **Endpoint:** `GET /openApi/swap/v2/user/commissionRate`
- **Authentication:** Required
- **Parameters:**
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query commission rate for swap trading

#### Get Income History
- **Endpoint:** `GET /openApi/swap/v2/user/income`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
  - `incomeType` (string, optional) - Income type
  - `startTime` (long, optional) - Start time
  - `endTime` (long, optional) - End time
  - `limit` (integer, optional) - Number of records
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query account profit/loss fund flow

---

## Position Endpoints (Authenticated)

### Standard Contract Positions

#### Get All Positions (Standard)
- **Endpoint:** `GET /openApi/contract/v1/allPosition`
- **Authentication:** Required
- **Parameters:**
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query all standard contract positions

#### Get Contract Balance
- **Endpoint:** `GET /openApi/contract/v1/balance`
- **Authentication:** Required
- **Parameters:**
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query standard contract account balance

### Perpetual Swap Positions

#### Get Swap Positions
- **Endpoint:** `GET /openApi/swap/v2/user/positions`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, optional) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query perpetual swap positions

#### Set Leverage
- **Endpoint:** `POST /openApi/swap/v2/trade/leverage`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `side` (string, required) - `LONG` or `SHORT`
  - `leverage` (integer, required) - Leverage value (1-125)
  - `timestamp` (long, required) - Request timestamp
- **Description:** Set leverage for a position

#### Get Leverage
- **Endpoint:** `GET /openApi/swap/v2/trade/leverage`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query current leverage settings

#### Set Margin Mode
- **Endpoint:** `POST /openApi/swap/v2/trade/marginType`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `marginType` (string, required) - `ISOLATED` or `CROSSED`
  - `timestamp` (long, required) - Request timestamp
- **Description:** Set margin mode (isolated or crossed)

#### Query Margin Mode
- **Endpoint:** `GET /openApi/swap/v2/trade/marginType`
- **Authentication:** Required
- **Parameters:**
  - `symbol` (string, required) - Trading pair
  - `timestamp` (long, required) - Request timestamp
- **Description:** Query current margin mode

---

## WebSocket User Data Endpoints

### Create Listen Key (Spot)
- **Endpoint:** `POST /openApi/spot/v1/user/listen-key`
- **Authentication:** Required (API Key in header)
- **Parameters:** None
- **Description:** Generate a listen key for spot user data stream (valid for 1 hour)

### Extend Listen Key (Spot)
- **Endpoint:** `PUT /openApi/spot/v1/user/listen-key`
- **Authentication:** Required
- **Parameters:** None
- **Description:** Extend listen key validity (recommended every 30 minutes)

### Delete Listen Key (Spot)
- **Endpoint:** `DELETE /openApi/spot/v1/user/listen-key`
- **Authentication:** Required
- **Parameters:** None
- **Description:** Delete/invalidate a listen key

### Create Listen Key (Swap)
- **Endpoint:** `POST /openApi/swap/v2/user/listen-key`
- **Authentication:** Required
- **Parameters:** None
- **Description:** Generate a listen key for swap user data stream

### Extend Listen Key (Swap)
- **Endpoint:** `PUT /openApi/swap/v2/user/listen-key`
- **Authentication:** Required
- **Parameters:** None
- **Description:** Extend swap listen key validity

### Delete Listen Key (Swap)
- **Endpoint:** `DELETE /openApi/swap/v2/user/listen-key`
- **Authentication:** Required
- **Parameters:** None
- **Description:** Delete/invalidate swap listen key

---

## Trait Mapping

### MarketData Trait

| Method | Spot Endpoint | Futures Endpoint |
|--------|---------------|------------------|
| `get_symbols()` | `GET /openApi/spot/v1/common/symbols` | `GET /openApi/swap/v2/quote/contracts` |
| `get_orderbook()` | `GET /openApi/spot/v1/market/depth` | `GET /openApi/swap/v2/quote/depth` |
| `get_trades()` | `GET /openApi/spot/v1/market/trades` | `GET /openApi/swap/v2/quote/trades` |
| `get_klines()` | `GET /openApi/spot/v1/market/kline` | `GET /openApi/swap/v2/quote/klines` |
| `get_ticker()` | `GET /openApi/spot/v1/ticker/24hr` | `GET /openApi/swap/v2/quote/ticker` |

### Trading Trait

| Method | Spot Endpoint | Futures Endpoint |
|--------|---------------|------------------|
| `place_order()` | `POST /openApi/spot/v1/trade/order` | `POST /openApi/swap/v2/trade/order` |
| `cancel_order()` | `DELETE /openApi/spot/v1/trade/order` | `DELETE /openApi/swap/v2/trade/order` |
| `get_order()` | `GET /openApi/spot/v1/trade/order` | `GET /openApi/swap/v2/trade/order` |
| `get_open_orders()` | `GET /openApi/spot/v1/trade/openOrders` | `GET /openApi/swap/v2/trade/openOrders` |
| `get_order_history()` | `GET /openApi/spot/v1/trade/historyOrders` | `GET /openApi/swap/v2/trade/allOrders` |

### Account Trait

| Method | Spot Endpoint | Futures Endpoint |
|--------|---------------|------------------|
| `get_balance()` | `GET /openApi/spot/v1/account/balance` | `GET /openApi/swap/v2/user/balance` |
| `get_account_info()` | `GET /openApi/spot/v1/account/balance` | `GET /openApi/swap/v2/user/balance` |

### Positions Trait (Futures only)

| Method | Endpoint |
|--------|----------|
| `get_positions()` | `GET /openApi/swap/v2/user/positions` |
| `set_leverage()` | `POST /openApi/swap/v2/trade/leverage` |
| `get_leverage()` | `GET /openApi/swap/v2/trade/leverage` |
| `set_margin_mode()` | `POST /openApi/swap/v2/trade/marginType` |
| `close_position()` | `POST /openApi/swap/v2/trade/closeAllPositions` |

---

## Notes

1. **Timestamp Requirement:** All authenticated endpoints require a `timestamp` parameter in milliseconds. The server rejects requests with timestamps older than 5 seconds.

2. **Signature Required:** All authenticated endpoints require HMAC SHA256 signature of request parameters.

3. **Symbol Format:**
   - Spot and Swap use hyphenated format: `BTC-USDT`
   - Some older endpoints may accept `BTCUSDT` format

4. **Kline Intervals:** BingX uses `1min`, `5min`, `15min`, `30min`, `60min`, `1day` (not `1m`, `5m`, etc.)

5. **Order Types:**
   - Spot: `MARKET`, `LIMIT`
   - Swap: `MARKET`, `LIMIT`, `STOP_MARKET`, `STOP`, `TAKE_PROFIT_MARKET`, `TAKE_PROFIT`, `TRAILING_STOP_MARKET`

6. **API Version Migration:** BingX is phasing out V1 endpoints in favor of V2. Use V2 endpoints when available.

---

## Sources

- [BingX API Docs](https://bingx-api.github.io/docs/)
- [BingX API GitHub Repository](https://github.com/BingX-API/docs)
- [BingX Standard Contract API](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [BingX Swap V2 API](https://github.com/BingX-API/BingX-swap-api-v2-doc)
- [CCXT BingX Documentation](https://docs.ccxt.com/exchanges/bingx)
- [BingX API Rate Limit Upgrades](https://bingx.com/en/support/articles/31103871611289)
