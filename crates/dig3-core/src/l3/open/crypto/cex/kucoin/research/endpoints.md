# KuCoin API Endpoints Research

**Research Date**: 2026-01-20

This document contains comprehensive research on KuCoin's official API endpoints for both Spot and Futures trading.

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
- [WebSocket Token Endpoints](#websocket-token-endpoints)
- [Discrepancies with Current Implementation](#discrepancies-with-current-implementation)

---

## Base URLs

### Production (Mainnet)

| Environment | Type | Base URL |
|------------|------|----------|
| **Unified Account** | REST | `https://api.kucoin.com` |
| **Spot & Margin** | REST | `https://api.kucoin.com` |
| **Futures** | REST | `https://api-futures.kucoin.com` |
| **Broker** | REST | `https://api-broker.kucoin.com` |
| **Spot & Margin** | WebSocket | `wss://ws-api-spot.kucoin.com` |
| **Futures** | WebSocket | `wss://ws-api-futures.kucoin.com` |

**Alternative WebSocket URLs:**
- Unified Account Spot: `wss://x-push-spot.kucoin.com`
- Unified Account Futures: `wss://x-push-futures.kucoin.com`
- Private WebSocket: `wss://wsapi-push.kucoin.com`
- Add/Cancel Order WebSocket: `wss://wsapi.kucoin.com`

### Sandbox (Testnet)

| Type | Base URL |
|------|----------|
| **Spot REST** | `https://openapi-sandbox.kucoin.com` |
| **Futures REST** | `https://api-sandbox-futures.kucoin.com` |
| **Spot WebSocket** | `wss://ws-api-sandbox.kucoin.com` |
| **Futures WebSocket** | `wss://ws-api-sandbox-futures.kucoin.com` |

**Note**: Current implementation in `endpoints.rs` correctly implements these URLs.

---

## SPOT Endpoints

### SPOT Market Data

#### 1. Get Server Time

- **Endpoint**: `GET /api/v1/timestamp`
- **Full URL**: `https://api.kucoin.com/api/v1/timestamp`
- **Auth Required**: No (Public)
- **Parameters**: None
- **Rate Limit Weight**: 3
- **Description**: Get API server time in milliseconds
- **Response Example**:
  ```json
  {
    "code": "200000",
    "data": 1729100692873
  }
  ```
- **Notes**: Recommended for time sync. Server-client time difference must be less than 5 seconds.

**Current Implementation**: ✓ Correct - `/api/v1/timestamp`

---

#### 2. Get Ticker (Single Symbol)

- **Endpoint**: `GET /api/v1/market/orderbook/level1`
- **Full URL**: `https://api.kucoin.com/api/v1/market/orderbook/level1?symbol=BTC-USDT`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `symbol` (required, string): Trading pair (e.g., "BTC-USDT")
- **Description**: Level 1 Market Data - best bid/ask price and size, last traded price/size
- **Response Fields**:
  - `sequence`: Sequence number
  - `price`: Last traded price
  - `size`: Last traded amount
  - `bestBid`: Best bid price
  - `bestBidSize`: Best bid size
  - `bestAsk`: Best ask price
  - `bestAskSize`: Best ask size
  - `time`: Timestamp (milliseconds)

**Current Implementation**: ✓ Correct - `/api/v1/market/orderbook/level1` (SpotPrice)

---

#### 3. Get Orderbook

KuCoin provides multiple orderbook depth levels:

##### Level 1 (Ticker)
- **Endpoint**: `GET /api/v1/market/orderbook/level1`
- **Parameters**: `symbol` (required)
- **Description**: Best bid/ask only

##### Level 2 - Partial (20 depth)
- **Endpoint**: `GET /api/v1/market/orderbook/level2_20`
- **Full URL**: `https://api.kucoin.com/api/v1/market/orderbook/level2_20?symbol=BTC-USDT`
- **Auth Required**: No (Public)
- **Parameters**: `symbol` (required)
- **Description**: Returns 20 bids and 20 asks
- **Response**: Faster, less traffic

##### Level 2 - Partial (100 depth)
- **Endpoint**: `GET /api/v1/market/orderbook/level2_100`
- **Full URL**: `https://api.kucoin.com/api/v1/market/orderbook/level2_100?symbol=BTC-USDT`
- **Auth Required**: No (Public)
- **Parameters**: `symbol` (required)
- **Description**: Returns 100 bids and 100 asks
- **Response**: Faster, less traffic

##### Level 2 - Full Orderbook
- **Endpoint**: `GET /api/v3/market/orderbook/level2`
- **Auth Required**: No (Public)
- **Parameters**: `symbol` (required)
- **Description**: Full order book depth (all levels)
- **Note**: Use WebSocket incremental feed for real-time updates

**Current Implementation**: ✓ Correct - `/api/v1/market/orderbook/level2_100` (SpotOrderbook)

---

#### 4. Get Klines (Candlestick Data)

- **Endpoint**: `GET /api/v1/market/candles`
- **Full URL**: `https://api.kucoin.com/api/v1/market/candles?symbol=BTC-USDT&type=1min&startAt=1566703297&endAt=1566789757`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `symbol` (required, string): Trading pair
  - `type` (required, enum): Candlestick interval
  - `startAt` (optional, int64): Start time in seconds (unix timestamp)
  - `endAt` (optional, int64): End time in seconds (unix timestamp)
- **Supported Intervals** (`type`):
  - `1min`, `3min`, `5min`, `15min`, `30min`
  - `1hour`, `2hour`, `4hour`, `6hour`, `8hour`, `12hour`
  - `1day`, `1week`, `1month`
- **Max Records**: 1500 per request
- **Notes**:
  - Klines may be incomplete (no data if no ticks in interval)
  - Do NOT poll frequently - use WebSocket for real-time data
  - Data sorted by time ascending

**Current Implementation**: ⚠️ **DISCREPANCY**
- Current: `/api/v1/market/candles`
- Note: Interval mapping in `map_kline_interval()` is correct

---

#### 5. Get 24h Ticker Stats (Single Symbol)

- **Endpoint**: `GET /api/v1/market/stats`
- **Full URL**: `https://api.kucoin.com/api/v1/market/stats?symbol=BTC-USDT`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `symbol` (required, string): Trading pair
- **Response Fields**:
  - `symbol`: Trading pair
  - `high`: Highest price in 24h
  - `low`: Lowest price in 24h
  - `vol`: 24h volume (base currency)
  - `volValue`: 24h volume (quote currency)
  - `last`: Last traded price
  - `buy`: Best bid price
  - `sell`: Best ask price
  - `changePrice`: 24h price change
  - `changeRate`: 24h change rate
  - `averagePrice`: 24h average price
  - `time`: Timestamp

**Current Implementation**: ✓ Correct - `/api/v1/market/stats` (SpotTicker)

---

#### 6. Get All Tickers

- **Endpoint**: `GET /api/v1/market/allTickers`
- **Full URL**: `https://api.kucoin.com/api/v1/market/allTickers`
- **Auth Required**: No (Public)
- **Parameters**: None
- **Description**: Market tickers for ALL trading pairs (including 24h volume)
- **Update Frequency**: Snapshot every 2 seconds
- **Response Structure**:
  ```json
  {
    "code": "200000",
    "data": {
      "time": 1550653727731,
      "ticker": [
        {
          "symbol": "BTC-USDT",
          "symbolName": "BTC-USDT",
          "buy": "0.00001191",
          "sell": "0.00001206",
          "changeRate": "0.057",
          "changePrice": "0.00000065",
          "high": "0.0000123",
          "low": "0.00001109",
          "vol": "45161.5073",
          "volValue": "0.53836190",
          "last": "0.00001204",
          "averagePrice": "0.00001175"
        }
      ]
    }
  }
  ```

**Current Implementation**: ✓ Correct - `/api/v1/market/allTickers` (SpotAllTickers)

---

#### 7. Get Symbols / Trading Pairs Info

- **Endpoint**: `GET /api/v2/symbols`
- **Full URL**: `https://api.kucoin.com/api/v2/symbols`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `market` (optional, string): Filter by market (e.g., "USDS", "BTC")
- **Description**: List of available trading pairs with trading rules
- **Response Fields** (per symbol):
  - `symbol`: Trading pair (e.g., "BTC-USDT")
  - `name`: Display name
  - `baseCurrency`, `quoteCurrency`: Base and quote assets
  - `baseMinSize`, `baseMaxSize`: Min/max order size (base)
  - `quoteMinSize`, `quoteMaxSize`: Min/max order size (quote)
  - `baseIncrement`: Min size increment
  - `quoteIncrement`: Min price increment
  - `priceIncrement`: Tick size
  - `feeCurrency`: Fee currency
  - `enableTrading`: Trading enabled flag
  - `isMarginEnabled`: Margin trading flag
  - `priceLimitRate`: Price limit rate

**Current Implementation**: ✓ Correct - `/api/v2/symbols` (SpotSymbols)

---

### SPOT Trading

#### 1. Create Order (Place Order)

- **Endpoint**: `POST /api/v1/orders`
- **Full URL**: `https://api.kucoin.com/api/v1/orders`
- **Auth Required**: Yes (Private) - Requires "Spot" trading permission
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Required Parameters** (JSON body):
  - `clientOid` (string): Unique order ID from client
  - `side` (enum): "buy" or "sell"
  - `symbol` (string): Trading pair (e.g., "BTC-USDT")
  - `type` (enum): "limit" or "market"

  **For limit orders**:
  - `price` (string): Order price per unit
  - `size` (string): Order amount (base currency)

  **For market buy orders**:
  - `funds` (string): Quote currency amount to spend

  **For market sell orders**:
  - `size` (string): Amount to sell (base currency)

- **Optional Parameters**:
  - `timeInForce` (enum): "GTC" (Good Till Canceled), "IOC" (Immediate or Cancel), "FOK" (Fill or Kill)
  - `postOnly` (boolean): Post-only flag (limit orders only)
  - `hidden` (boolean): Hidden order flag
  - `iceberg` (boolean): Iceberg order flag
  - `visibleSize` (string): Visible size for iceberg orders
  - `cancelAfter` (long): Cancel after N seconds
  - `remark` (string): Order remarks/notes (max 100 characters)
  - `tradeType` (enum): "TRADE" (spot, default), "MARGIN_TRADE" (cross margin), "MARGIN_ISOLATED_TRADE" (isolated margin)

- **Response**:
  ```json
  {
    "code": "200000",
    "data": {
      "orderId": "5bd6e9286d99522a52e458de"
    }
  }
  ```

- **Limits**:
  - Max 2000 active orders per account
  - Max 200 active orders per trading pair

- **Notes**:
  - When order successfully placed, only orderId is returned
  - Use WebSocket to monitor order updates

**Current Implementation**: ✓ Correct - `/api/v1/orders` (SpotCreateOrder)

---

#### 2. Cancel Order by Order ID

- **Endpoint**: `DELETE /api/v1/orders/{orderId}`
- **Full URL**: `https://api.kucoin.com/api/v1/orders/5bd6e9286d99522a52e458de`
- **Auth Required**: Yes (Private) - Requires "Spot" trading permission
- **HTTP Method**: DELETE
- **Path Parameters**:
  - `orderId` (string): Order ID to cancel
- **Response**:
  ```json
  {
    "code": "200000",
    "data": {
      "cancelledOrderIds": ["5bd6e9286d99522a52e458de"]
    }
  }
  ```

- **Notes**:
  - Only a cancellation request - actual cancellation is async
  - Do NOT cancel until receiving "Open" message from WebSocket
  - Get result via WebSocket or query order status

**Current Implementation**: ✓ Correct - `/api/v1/orders/{orderId}` (SpotCancelOrder)

---

#### 3. Get Order by ID

- **Endpoint**: `GET /api/v1/orders/{orderId}`
- **Full URL**: `https://api.kucoin.com/api/v1/orders/5bd6e9286d99522a52e458de`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Path Parameters**:
  - `orderId` (string): Order ID to query
- **Response Fields**:
  - Order details including status, filled amount, fees, etc.

**Current Implementation**: ✓ Correct - `/api/v1/orders/{orderId}` (SpotGetOrder)

---

#### 4. Get Open Orders

- **Endpoint**: `GET /api/v1/orders`
- **Full URL**: `https://api.kucoin.com/api/v1/orders?status=active&symbol=BTC-USDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `status` (required, string): "active" (for open orders) or "done" (for historical)
  - `symbol` (optional, string): Filter by trading pair
  - `side` (optional, enum): "buy" or "sell"
  - `type` (optional, enum): "limit" or "market"
  - `tradeType` (optional, enum): "TRADE" (default), "MARGIN_TRADE", "MARGIN_ISOLATED_TRADE"
  - `startAt` (optional, long): Start time (milliseconds)
  - `endAt` (optional, long): End time (milliseconds)
  - `currentPage` (optional, int): Page number (default: 1)
  - `pageSize` (optional, int): Records per page (default: 50, max: 500)

- **Notes**:
  - Active orders: no time limit
  - Done orders: max 24*7 hours range
  - Results sorted descending by time

**Current Implementation**: ✓ Correct - `/api/v1/orders` (SpotOpenOrders)

---

#### 5. Get All Orders (History)

- **Endpoint**: `GET /api/v1/orders`
- **Auth Required**: Yes (Private)
- **Query Parameters**: Same as "Get Open Orders" but with `status=done`
- **History Retention**:
  - Cancelled orders: 1 month
  - Filled orders: 6 months

**Current Implementation**: ✓ Correct - `/api/v1/orders` (SpotAllOrders)

---

#### 6. Cancel All Orders

- **Endpoint**: `DELETE /api/v1/orders`
- **Full URL**: `https://api.kucoin.com/api/v1/orders?symbol=BTC-USDT&tradeType=TRADE`
- **Auth Required**: Yes (Private)
- **HTTP Method**: DELETE
- **Query Parameters**:
  - `symbol` (optional, string): Cancel for specific symbol
  - `tradeType` (optional, enum): "TRADE", "MARGIN_TRADE", "MARGIN_ISOLATED_TRADE"

**Current Implementation**: ✓ Correct - `/api/v1/orders` (SpotCancelAllOrders)

---

### SPOT Account

#### 1. Get Accounts (List Balances)

- **Endpoint**: `GET /api/v1/accounts`
- **Full URL**: `https://api.kucoin.com/api/v1/accounts?currency=BTC&type=trade`
- **Auth Required**: Yes (Private) - Requires "General" permission
- **HTTP Method**: GET
- **Query Parameters**:
  - `currency` (optional, string): Filter by currency (e.g., "BTC")
  - `type` (optional, enum): Account type - "main", "trade", "trade_hf", "margin"
- **Response Example**:
  ```json
  {
    "code": "200000",
    "data": [
      {
        "id": "5bd6e9286d99522a52e458de",
        "currency": "BTC",
        "type": "main",
        "balance": "237582.04299",
        "available": "237582.032",
        "holds": "0.01099"
      }
    ]
  }
  ```

- **Account Types**:
  - `main`: Storage, deposit, withdrawal (cannot trade directly)
  - `trade`: Trading account (for spot trading)
  - `trade_hf`: High-frequency trading account
  - `margin`: Margin trading account

- **Response Fields**:
  - `id`: Account ID
  - `currency`: Currency code
  - `type`: Account type
  - `balance`: Total balance
  - `available`: Available balance for trading/withdrawal
  - `holds`: Funds on hold (in orders)

**Current Implementation**: ✓ Correct - `/api/v1/accounts` (SpotAccounts)

---

#### 2. Get Account Detail

- **Endpoint**: `GET /api/v1/accounts/{accountId}`
- **Full URL**: `https://api.kucoin.com/api/v1/accounts/5bd6e9286d99522a52e458de`
- **Auth Required**: Yes (Private) - Requires "General" permission
- **HTTP Method**: GET
- **Path Parameters**:
  - `accountId` (string): Account ID from "Get Accounts" endpoint
- **Response Example**:
  ```json
  {
    "code": "200000",
    "data": {
      "currency": "KCS",
      "balance": "1000000060.6299",
      "available": "1000000060.6299",
      "holds": "0"
    }
  }
  ```

**Current Implementation**: ✓ Correct - `/api/v1/accounts/{accountId}` (SpotAccountDetail)

---

## FUTURES Endpoints

### FUTURES Market Data

#### 1. Get Server Time

- Same as Spot: `GET /api/v1/timestamp`
- Base URL: `https://api-futures.kucoin.com/api/v1/timestamp`

---

#### 2. Get Ticker (Single Symbol)

- **Endpoint**: `GET /api/v1/ticker`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/ticker?symbol=XBTUSDM`
- **Auth Required**: No (Public)
- **Rate Limit Weight**: 2
- **Query Parameters**:
  - `symbol` (required, string): Contract symbol (e.g., "XBTUSDM", "XBTUSDTM")
- **Description**: Last traded price/size, best bid/ask price/size
- **Response Example**:
  ```json
  {
    "code": "200000",
    "data": {
      "sequence": 1001,
      "symbol": "XBTUSDM",
      "side": "buy",
      "size": 10,
      "price": "7000.0",
      "bestBidSize": 20,
      "bestBidPrice": "7000.0",
      "bestAskSize": 30,
      "bestAskPrice": "7001.0",
      "tradeId": "5cbd7377a6ffab0c7ba98b26",
      "ts": 1550653727731
    }
  }
  ```

**Current Implementation**: ✓ Correct - `/api/v1/ticker` (FuturesPrice, FuturesTicker)

---

#### 3. Get Orderbook

##### Full Orderbook (Level 2)
- **Endpoint**: `GET /api/v1/level2/snapshot`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/level2/snapshot?symbol=XBTUSDM`
- **Auth Required**: No (Public)
- **Parameters**: `symbol` (required)
- **Description**: Full order book snapshot

##### Partial Orderbook - 100 Depth
- **Endpoint**: `GET /api/v1/level2/depth100`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/level2/depth100?symbol=XBTUSDM`
- **Auth Required**: No (Public)
- **Parameters**: `symbol` (required)
- **Description**: Top 100 bids and asks

##### Partial Orderbook - 50 Depth
- **Endpoint**: `GET /api/v1/level2/depth50`
- **Parameters**: `symbol` (required)

##### Partial Orderbook - 5 Depth
- **Endpoint**: `GET /api/v1/level2/depth5`
- **Parameters**: `symbol` (required)

**Current Implementation**: ✓ Correct - `/api/v1/level2/depth100` (FuturesOrderbook)

---

#### 4. Get Klines (Candlestick Data)

- **Endpoint**: `GET /api/v1/kline/query`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/kline/query?symbol=XBTUSDTM&granularity=60&from=1750389927000&to=1750393527000`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `symbol` (required, string): Contract symbol
  - `granularity` (required, int): Time interval in minutes
  - `from` (optional, long): Start time (milliseconds)
  - `to` (optional, long): End time (milliseconds)
- **Supported Granularities** (minutes):
  - `1`, `5`, `15`, `30`, `60`, `120`, `240`, `480`, `720`, `1440`, `10080` (1 week)
- **Max Records**: 500 per request
- **Notes**: If time range + granularity exceeds 500 records, only 500 returned

**Current Implementation**: ⚠️ **DISCREPANCY**
- Current: `/api/v1/kline/query`
- Correct ✓
- Note: Granularity is in minutes (not string intervals like Spot)

---

#### 5. Get Contracts Info (Symbols List)

- **Endpoint**: `GET /api/v1/contracts/active`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/contracts/active`
- **Auth Required**: No (Public)
- **Rate Limit Weight**: 3
- **Parameters**: None
- **Description**: List of all tradable contracts with details
- **Response Fields** (per contract):
  - `symbol`: Contract symbol (e.g., "XBTUSDM")
  - `rootSymbol`: Root symbol
  - `type`: Contract type ("FFWCSX" = perpetual)
  - `firstOpenDate`: First open date
  - `expireDate`: Expiry date (null for perpetual)
  - `settleDate`: Settlement date
  - `baseCurrency`, `quoteCurrency`: Base and quote currencies
  - `settleCurrency`: Settlement currency
  - `maxOrderQty`: Max order quantity
  - `maxPrice`: Max price
  - `lotSize`: Lot size
  - `tickSize`: Price tick size
  - `indexPriceTickSize`: Index price tick
  - `multiplier`: Multiplier (e.g., 0.001 means 1 lot = 0.001 BTC)
  - `initialMargin`: Initial margin rate
  - `maintainMargin`: Maintenance margin rate
  - `maxRiskLimit`: Max risk limit
  - `minRiskLimit`: Min risk limit
  - `riskStep`: Risk limit step
  - `makerFeeRate`, `takerFeeRate`: Fee rates
  - `takerFixFee`, `makerFixFee`: Fixed fees
  - `settlementFee`: Settlement fee
  - `isDeleverage`: Is ADL enabled
  - `isQuanto`: Is quanto contract
  - `isInverse`: Is inverse contract
  - `markMethod`: Mark price method
  - `fairMethod`: Fair price method
  - `fundingBaseSymbol`, `fundingQuoteSymbol`: Funding symbols
  - `fundingRateSymbol`: Funding rate symbol
  - `indexSymbol`: Index symbol
  - `settlementSymbol`: Settlement symbol
  - `status`: Contract status ("Open")

- **Note**: Basic unit is "lots". Multiply by `multiplier` to get actual crypto amount

**Current Implementation**: ✓ Correct - `/api/v1/contracts/active` (FuturesContracts)

---

#### 6. Get Funding Rate

- **Endpoint**: `GET /api/v1/funding-rate/{symbol}/current`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/funding-rate/XBTUSDTM/current`
- **Auth Required**: No (Public)
- **Path Parameters**:
  - `symbol` (string): Contract symbol
- **Response Fields**:
  - `symbol`: Contract symbol
  - `granularity`: Granularity (milliseconds)
  - `timePoint`: Time point (milliseconds)
  - `value`: Current funding rate
  - `predictedValue`: Predicted funding rate

**Current Implementation**: ✓ Correct - `/api/v1/funding-rate/{symbol}/current` (FundingRate)

---

#### 7. Get All Tickers

- **Endpoint**: `GET /api/v1/allTickers`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/allTickers`
- **Auth Required**: No (Public)
- **Parameters**: None
- **Description**: Market tickers for all futures contracts

**Current Implementation**: ✓ Correct - `/api/v1/allTickers` (FuturesAllTickers)

---

### FUTURES Trading

#### 1. Create Order (Place Order)

- **Endpoint**: `POST /api/v1/orders`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/orders`
- **Auth Required**: Yes (Private) - Requires "Futures Trading" permission
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Required Parameters** (JSON body):
  - `clientOid` (string): Unique order ID from client
  - `symbol` (string): Contract symbol (e.g., "XBTUSDTM")
  - `side` (enum): "buy" or "sell"
  - `leverage` (string): Leverage (e.g., "3")
  - `type` (enum): "limit" or "market"

  **For limit orders**:
  - `price` (string): Order price
  - `size` (int): Order quantity (number of contracts)

  **For market orders**:
  - `size` (int): Order quantity

- **Optional Parameters**:
  - `marginMode` (enum): "ISOLATED" or "CROSS"
  - `positionSide` (enum): "BOTH" (one-way mode), "LONG", "SHORT" (hedge mode)
  - `timeInForce` (enum): "GTC", "IOC", "FOK"
  - `postOnly` (boolean): Post-only flag
  - `hidden` (boolean): Hidden order
  - `iceberg` (boolean): Iceberg order
  - `visibleSize` (int): Visible size for iceberg
  - `reduceOnly` (boolean): Reduce-only flag
  - `closeOrder` (boolean): Close position order
  - `forceHold` (boolean): Force hold
  - `remark` (string): Order remarks (max 100 chars)
  - `stopPrice` (string): Stop price (for stop orders)
  - `stopPriceType` (enum): "TP" (take profit), "IP" (index price), "MP" (mark price)

- **Example Request**:
  ```json
  {
    "clientOid": "5c52e11203aa677f33e493fb",
    "symbol": "XBTUSDTM",
    "marginMode": "ISOLATED",
    "leverage": 3,
    "positionSide": "BOTH",
    "side": "buy",
    "type": "limit",
    "size": 1,
    "price": "100000",
    "timeInForce": "GTC",
    "reduceOnly": false,
    "remark": "order_remarks"
  }
  ```

- **Response**:
  ```json
  {
    "code": "200000",
    "data": {
      "orderId": "5bd6e9286d99522a52e458de"
    }
  }
  ```

**Current Implementation**: ✓ Correct - `/api/v1/orders` (FuturesCreateOrder)

---

#### 2. Cancel Order by Order ID

- **Endpoint**: `DELETE /api/v1/orders/{orderId}`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/orders/5bd6e9286d99522a52e458de`
- **Auth Required**: Yes (Private) - Requires "Futures Trading" permission
- **HTTP Method**: DELETE
- **Path Parameters**:
  - `orderId` (string): Order ID to cancel

**Current Implementation**: ✓ Correct - `/api/v1/orders/{orderId}` (FuturesCancelOrder)

---

#### 3. Get Order by ID

- **Endpoint**: `GET /api/v1/orders/{orderId}`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/orders/5bd6e9286d99522a52e458de`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Path Parameters**:
  - `orderId` (string): Order ID to query

- **Alternative by clientOid**:
  - **Endpoint**: `GET /api/v1/orders/byClientOid`
  - **Query Parameter**: `clientOid`

**Current Implementation**: ✓ Correct - `/api/v1/orders/{orderId}` (FuturesGetOrder)

---

#### 4. Get Open Orders

- **Endpoint**: `GET /api/v1/orders`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/orders?status=active&symbol=XBTUSDTM`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `status` (optional, string): "active" or "done" (default: both)
  - `symbol` (optional, string): Filter by contract symbol
  - `side` (optional, enum): "buy" or "sell"
  - `type` (optional, enum): "limit" or "market"
  - `startAt` (optional, long): Start time (milliseconds)
  - `endAt` (optional, long): End time (milliseconds)
  - `currentPage` (optional, int): Page number
  - `pageSize` (optional, int): Records per page

- **Notes**:
  - Active orders: no time limit
  - Done orders: max 24*7 hours range
  - If only `endAt` specified, `startAt` = `endAt` - 24h

**Current Implementation**: ✓ Correct - `/api/v1/orders` (FuturesOpenOrders)

---

#### 5. Get All Orders (History)

- **Endpoint**: `GET /api/v1/orders`
- Same as "Get Open Orders" but with `status=done`

**Current Implementation**: ✓ Correct - `/api/v1/orders` (FuturesAllOrders)

---

#### 6. Cancel All Orders

- **Endpoint**: `DELETE /api/v1/orders`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/orders?symbol=XBTUSDTM`
- **Auth Required**: Yes (Private)
- **HTTP Method**: DELETE
- **Query Parameters**:
  - `symbol` (optional, string): Cancel orders for specific symbol

**Current Implementation**: ✓ Correct - `/api/v1/orders` (FuturesCancelAllOrders)

---

### FUTURES Account

#### 1. Get Account Overview (Balance)

- **Endpoint**: `GET /api/v1/account-overview`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/account-overview?currency=XBT`
- **Auth Required**: Yes (Private) - Requires "General" permission
- **HTTP Method**: GET
- **Query Parameters**:
  - `currency` (optional, string): Currency code (e.g., "XBT", "USDT")
- **Response Fields**:
  - `accountEquity`: Account equity = marginBalance + unrealisedPNL
  - `unrealisedPNL`: Unrealised profit and loss
  - `marginBalance`: Margin balance = positionMargin + orderMargin + frozenFunds + availableBalance - unrealisedPNL
  - `positionMargin`: Position margin
  - `orderMargin`: Order margin (funds in open orders)
  - `frozenFunds`: Frozen funds (withdrawals, transfers)
  - `availableBalance`: Available balance
  - `currency`: Currency code
  - `riskRatio`: Risk ratio

**Current Implementation**: ✓ Correct - `/api/v1/account-overview` (FuturesAccount)

---

#### 2. Get Positions List

- **Endpoint**: `GET /api/v1/positions`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/positions`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `currency` (optional, string): Filter by settlement currency
- **Description**: List of all positions

**Current Implementation**: ✓ Correct - `/api/v1/positions` (FuturesPositions)

---

#### 3. Get Position Detail (Single Symbol)

- **Endpoint**: `GET /api/v1/position`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/position?symbol=XBTUSDM`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `symbol` (required, string): Contract symbol
- **Response Fields**:
  - `id`: Position ID
  - `symbol`: Contract symbol
  - `autoDeposit`: Auto deposit margin flag
  - `maintMarginReq`: Maintenance margin requirement
  - `riskLimit`: Current risk limit
  - `realLeverage`: Real leverage
  - `crossMode`: Cross margin mode flag
  - `delevPercentage`: ADL percentage
  - `openingTimestamp`: Opening timestamp
  - `currentTimestamp`: Current timestamp
  - `currentQty`: Current quantity (signed: + long, - short)
  - `currentCost`: Current cost
  - `currentComm`: Current commission
  - `unrealisedCost`: Unrealised cost
  - `realisedGrossCost`: Realised gross cost
  - `realisedCost`: Realised cost
  - `isOpen`: Position open flag
  - `markPrice`: Mark price
  - `markValue`: Mark value
  - `posCost`: Position cost
  - `posCross`: Position cross margin
  - `posInit`: Position initial margin
  - `posComm`: Position commission
  - `posLoss`: Position loss
  - `posMargin`: Position margin
  - `posMaint`: Position maintenance margin
  - `maintMargin`: Maintenance margin
  - `realisedGrossPnl`: Realised gross PnL
  - `realisedPnl`: Realised PnL
  - `unrealisedPnl`: Unrealised PnL
  - `unrealisedPnlPcnt`: Unrealised PnL percentage
  - `unrealisedRoePcnt`: Unrealised ROE percentage
  - `avgEntryPrice`: Average entry price
  - `liquidationPrice`: Liquidation price
  - `bankruptPrice`: Bankruptcy price
  - `settleCurrency`: Settlement currency
  - `isInverse`: Inverse contract flag

**Current Implementation**: ✓ Correct - `/api/v1/position` (FuturesPosition)

---

#### 4. Set Leverage (Modify Risk Limit Level)

- **Endpoint**: `POST /api/v1/position/risk-limit-level/change`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/position/risk-limit-level/change`
- **Auth Required**: Yes (Private) - Requires "Futures Trading" permission
- **HTTP Method**: POST
- **Rate Limit Weight**: 5
- **Content-Type**: `application/json`
- **Request Body** (JSON):
  - `symbol` (string): Contract symbol (e.g., "XBTUSDTM")
  - `level` (int): Risk limit level (e.g., 2)

- **Example Request**:
  ```json
  {
    "symbol": "XBTUSDTM",
    "level": 2
  }
  ```

- **Example Response**:
  ```json
  {
    "code": "200000",
    "data": true
  }
  ```

- **Important Notes**:
  - **Only valid for Isolated Margin mode**
  - Adjusting level will **cancel all open orders** for that symbol
  - Response only indicates if request submitted successfully
  - Get actual result via WebSocket "Position Change Events"

**Current Implementation**: ✓ Correct - `/api/v1/position/risk-limit-level/change` (FuturesSetLeverage)

**Note**: The endpoint modifies "risk limit level", not leverage directly. Risk limit level determines max position size and margin requirements, which affects leverage.

---

## WebSocket Token Endpoints

### Public Token (No Authentication)

#### Spot & Margin
- **Endpoint**: `POST /api/v1/bullet-public`
- **Full URL**: `https://api.kucoin.com/api/v1/bullet-public`
- **Auth Required**: No
- **HTTP Method**: POST
- **Description**: Get token for public WebSocket channels (market data)

#### Futures
- **Endpoint**: `POST /api/v1/bullet-public`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/bullet-public`
- **Auth Required**: No
- **HTTP Method**: POST
- **Description**: Get token for public futures WebSocket channels

**Response Format** (both):
```json
{
  "code": "200000",
  "data": {
    "token": "2neAiuYvAU61ZDXANAGAsiL4-iAExhsBXZxftpOeh_55i3Ysy2q2LEsEWU64mdzUOPusi34M_wGoSf7iNyEWJ...",
    "instanceServers": [
      {
        "endpoint": "wss://ws-api-spot.kucoin.com/",
        "encrypt": true,
        "protocol": "websocket",
        "pingInterval": 18000,
        "pingTimeout": 10000
      }
    ]
  }
}
```

**Current Implementation**: ✓ Correct - `/api/v1/bullet-public` (WsPublicToken)

---

### Private Token (Authentication Required)

#### Spot & Margin (Classic Account)
- **Endpoint**: `POST /api/v1/bullet-private`
- **Full URL**: `https://api.kucoin.com/api/v1/bullet-private`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Description**: Get token for private WebSocket channels (account, orders)

#### Spot & Margin (Unified Account / Pro API)
- **Endpoint**: `POST /api/v2/bullet-private`
- **Full URL**: `https://api.kucoin.com/api/v2/bullet-private`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Description**: Get token for Pro API private channels

#### Futures (Classic)
- **Endpoint**: `POST /api/v1/bullet-private`
- **Full URL**: `https://api-futures.kucoin.com/api/v1/bullet-private`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Description**: Get token for private futures WebSocket channels

**Response Format**: Same as public token

**Current Implementation**: ⚠️ **POTENTIAL ISSUE**
- Current: `/api/v1/bullet-private`
- Note: There's also `/api/v2/bullet-private` for Pro API (Unified Account)
- For Classic Account (Spot/Futures), `/api/v1/bullet-private` is correct

---

## Discrepancies with Current Implementation

After comparing with `endpoints.rs`, here are the findings:

### ✅ CORRECT Implementations

**Base URLs**:
- Mainnet URLs: ✓ Correct
- Testnet URLs: ✓ Correct

**Spot Endpoints**:
- ✓ Timestamp: `/api/v1/timestamp`
- ✓ SpotPrice: `/api/v1/market/orderbook/level1`
- ✓ SpotOrderbook: `/api/v1/market/orderbook/level2_100`
- ✓ SpotKlines: `/api/v1/market/candles`
- ✓ SpotTicker: `/api/v1/market/stats`
- ✓ SpotAllTickers: `/api/v1/market/allTickers`
- ✓ SpotSymbols: `/api/v2/symbols`
- ✓ SpotCreateOrder: `/api/v1/orders`
- ✓ SpotCancelOrder: `/api/v1/orders/{orderId}`
- ✓ SpotGetOrder: `/api/v1/orders/{orderId}`
- ✓ SpotOpenOrders: `/api/v1/orders`
- ✓ SpotAllOrders: `/api/v1/orders`
- ✓ SpotCancelAllOrders: `/api/v1/orders`
- ✓ SpotAccounts: `/api/v1/accounts`
- ✓ SpotAccountDetail: `/api/v1/accounts/{accountId}`

**Futures Endpoints**:
- ✓ FuturesPrice: `/api/v1/ticker`
- ✓ FuturesOrderbook: `/api/v1/level2/depth100`
- ✓ FuturesKlines: `/api/v1/kline/query`
- ✓ FuturesTicker: `/api/v1/ticker`
- ✓ FuturesAllTickers: `/api/v1/allTickers`
- ✓ FuturesContracts: `/api/v1/contracts/active`
- ✓ FundingRate: `/api/v1/funding-rate/{symbol}/current`
- ✓ FuturesCreateOrder: `/api/v1/orders`
- ✓ FuturesCancelOrder: `/api/v1/orders/{orderId}`
- ✓ FuturesGetOrder: `/api/v1/orders/{orderId}`
- ✓ FuturesOpenOrders: `/api/v1/orders`
- ✓ FuturesAllOrders: `/api/v1/orders`
- ✓ FuturesCancelAllOrders: `/api/v1/orders`
- ✓ FuturesAccount: `/api/v1/account-overview`
- ✓ FuturesPositions: `/api/v1/positions`
- ✓ FuturesPosition: `/api/v1/position`
- ✓ FuturesSetLeverage: `/api/v1/position/risk-limit-level/change`

**WebSocket**:
- ✓ WsPublicToken: `/api/v1/bullet-public`
- ✓ WsPrivateToken: `/api/v1/bullet-private`

**Symbol Formatting**:
- ✓ Spot format: `{BASE}-{QUOTE}` (e.g., "BTC-USDT")
- ✓ Futures format: `{BASE}{QUOTE}M` with BTC→XBT mapping (e.g., "XBTUSDM")

**Kline Interval Mapping** (Spot):
- ✓ Correct mapping from `1m` → `1min`, `1h` → `1hour`, etc.

### ⚠️ NOTES / CLARIFICATIONS

1. **FuturesSetLeverage Endpoint**:
   - Current endpoint `/api/v1/position/risk-limit-level/change` is correct
   - However, this modifies "risk limit level", not leverage directly
   - Risk limit level affects max position size and margin, which indirectly controls leverage
   - Only works for **Isolated Margin** mode
   - Cancels all open orders when changed
   - Consider documenting this behavior

2. **Futures Klines Granularity**:
   - Futures uses numeric granularity in **minutes**: `1, 5, 15, 30, 60, 120, 240, 480, 720, 1440, 10080`
   - Spot uses string intervals: `1min, 3min, 5min, 15min, 30min, 1hour, 2hour, 4hour, 6hour, 8hour, 12hour, 1day, 1week, 1month`
   - Current implementation should handle this difference

3. **WebSocket Private Token**:
   - Classic Account: `/api/v1/bullet-private` ✓
   - Unified Account (Pro API): `/api/v2/bullet-private`
   - Current implementation uses v1, which is correct for Classic Account

### 📝 Additional Endpoints NOT in Current Implementation

These endpoints exist in the API but are not currently implemented:

**Spot**:
- `GET /api/v1/market/orderbook/level2_20` - Orderbook 20 depth (alternative to level2_100)
- `GET /api/v3/market/orderbook/level2` - Full orderbook (all levels)
- `GET /api/v1/orders/byClientOid` - Get order by clientOid
- `DELETE /api/v1/orders/client-order/{clientOid}` - Cancel order by clientOid
- `POST /api/v1/hf/orders` - High-frequency trading orders
- Many margin trading specific endpoints

**Futures**:
- `GET /api/v1/level2/snapshot` - Full orderbook snapshot (alternative)
- `GET /api/v1/level2/depth50` - Orderbook 50 depth
- `GET /api/v1/level2/depth5` - Orderbook 5 depth
- `GET /api/v1/contracts/{symbol}` - Single contract details
- `GET /api/v1/orders/byClientOid` - Get order by clientOid
- `POST /api/v1/position/margin/auto-deposit-status` - Auto deposit margin
- `POST /api/v1/position/margin/deposit-margin` - Manually add margin
- Take profit / stop loss order endpoints

These can be added as needed based on feature requirements.

---

## Summary

### Overall Status: ✅ EXCELLENT

The current implementation in `endpoints.rs` is **highly accurate** and aligns well with KuCoin's official API documentation. All critical endpoints for spot and futures trading are correctly implemented.

### Key Findings:

1. **Base URLs**: Correctly implemented for both production and sandbox
2. **Endpoint Paths**: All 100% accurate
3. **HTTP Methods**: Correctly assigned (GET/POST/DELETE)
4. **Auth Requirements**: Properly flagged (public vs private)
5. **Symbol Formatting**: Correct for both Spot and Futures (including BTC→XBT mapping)
6. **Interval Mapping**: Correct for Spot klines

### Recommendations:

1. **Document FuturesSetLeverage behavior**:
   - Clarify it's for risk limit level (not direct leverage)
   - Note it only works for Isolated Margin
   - Warn that it cancels open orders

2. **Consider adding optional endpoints** as needed:
   - Alternative orderbook depths
   - ClientOid-based order operations
   - Margin management endpoints

3. **Future enhancements**:
   - High-frequency trading endpoints (`/api/v1/hf/orders`)
   - Unified Account (Pro API) endpoints if needed

---

## Sources

Research compiled from the following official sources:

- [KuCoin API Documentation](https://www.kucoin.com/docs-new)
- [Base URL Documentation](https://www.kucoin.com/docs/basic-info/base-url)
- [Spot Trading Market Data](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-ticker)
- [Spot Trading Orders](https://www.kucoin.com/docs/rest/spot-trading/orders/place-order)
- [Get All Tickers](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-all-tickers)
- [Get Symbols List](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-symbols-list)
- [Get Klines - Spot](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-klines)
- [Get Server Time](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-server-time)
- [Account Detail - Spot](https://www.kucoin.com/docs/rest/account/basic-info/get-account-detail-spot-margin-trade_hf)
- [Futures Trading Orders](https://www.kucoin.com/docs/rest/futures-trading/orders/place-order)
- [Get Klines - Futures](https://www.kucoin.com/docs/rest/futures-trading/market-data/get-klines)
- [Get All Symbols - Futures](https://www.kucoin.com/docs-new/rest/futures-trading/market-data/get-all-symbols)
- [Get Account Overview - Futures](https://www.kucoin.com/docs/rest/funding/funding-overview/get-account-detail-futures)
- [Get Position Details](https://www.kucoin.com/docs/rest/futures-trading/positions/get-position-details)
- [Modify Risk Limit Level](https://www.kucoin.com/docs/rest/futures-trading/risk-limit/modify-risk-limit-level)
- [WebSocket Public Token](https://www.kucoin.com/docs/websocket/basic-info/apply-connect-token/public-token-no-authentication-required-)
- [WebSocket Private Token](https://www.kucoin.com/docs/websocket/basic-info/apply-connect-token/private-channels-authentication-request-required-)
- [Sandbox Documentation](https://www.kucoin.com/docs/beginners/sandbox)
