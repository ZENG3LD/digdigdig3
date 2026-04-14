# Bybit V5 API Endpoints Research

**Research Date**: 2026-01-20

This document contains comprehensive research on Bybit's V5 API endpoints for both Spot and Futures (Linear/USDT Perpetual) trading.

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
- [Summary](#summary)

---

## Base URLs

### Production (Mainnet)

| Environment | Type | Base URL |
|------------|------|----------|
| **REST API** | All Products | `https://api.bybit.com` |
| **REST API Bytick** | All Products | `https://api.bytick.com` |
| **REST API Mainnet** | All Products | `https://api-mainnet.bybit.com` |

**WebSocket URLs:**
- Spot: `wss://stream.bybit.com/v5/public/spot`
- Linear (USDT Perpetual): `wss://stream.bybit.com/v5/public/linear`
- Inverse: `wss://stream.bybit.com/v5/public/inverse`
- Option: `wss://stream.bybit.com/v5/public/option`
- Private: `wss://stream.bybit.com/v5/private`

### Testnet

| Type | Base URL |
|------|----------|
| **REST API** | `https://api-testnet.bybit.com` |
| **WebSocket Spot** | `wss://stream-testnet.bybit.com/v5/public/spot` |
| **WebSocket Linear** | `wss://stream-testnet.bybit.com/v5/public/linear` |
| **WebSocket Private** | `wss://stream-testnet.bybit.com/v5/private` |

**Note**: Bybit V5 uses a unified base URL structure. Product differentiation is handled via the `category` parameter in requests:
- `category=spot` for Spot trading
- `category=linear` for USDT Perpetual/Futures
- `category=inverse` for Inverse contracts
- `category=option` for Options

---

## SPOT Endpoints

### SPOT Market Data

#### 1. Get Server Time

- **Endpoint**: `GET /v5/market/time`
- **Full URL**: `https://api.bybit.com/v5/market/time`
- **Auth Required**: No (Public)
- **Parameters**: None
- **Description**: Get API server time in milliseconds
- **Response Example**:
  ```json
  {
    "retCode": 0,
    "retMsg": "OK",
    "result": {
      "timeSecond": "1702617474",
      "timeNano": "1702617474601069158"
    },
    "time": 1702617474601
  }
  ```
- **Notes**: Use `time` field (milliseconds) for timestamp sync

**Current Implementation**: ✓ To be implemented

---

#### 2. Get Ticker (Single Symbol)

- **Endpoint**: `GET /v5/market/tickers`
- **Full URL**: `https://api.bybit.com/v5/market/tickers?category=spot&symbol=BTCUSDT`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (optional, string): Trading pair (e.g., "BTCUSDT")
- **Description**: Get latest price snapshot, best bid/ask, and 24h trading volume
- **Response Fields**:
  - `symbol`: Trading pair
  - `lastPrice`: Last traded price
  - `bid1Price`: Best bid price
  - `bid1Size`: Best bid size
  - `ask1Price`: Best ask price
  - `ask1Size`: Best ask size
  - `highPrice24h`: 24h highest price
  - `lowPrice24h`: 24h lowest price
  - `volume24h`: 24h volume (base currency)
  - `turnover24h`: 24h turnover (quote currency)
  - `usdIndexPrice`: USD index price (spot specific)

**Current Implementation**: ✓ To be implemented

---

#### 3. Get Orderbook

- **Endpoint**: `GET /v5/market/orderbook`
- **Full URL**: `https://api.bybit.com/v5/market/orderbook?category=spot&symbol=BTCUSDT&limit=50`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (required, string): Trading pair
  - `limit` (optional, int): Orderbook depth levels (spot: 1-200, default 25)
- **Description**: Get current orderbook depth
- **Response Fields**:
  - `s`: Symbol name
  - `b`: Bids array, sorted by price descending
  - `a`: Asks array, sorted by price ascending
  - `ts`: Timestamp (milliseconds)
  - `u`: Update ID
  - `seq`: Cross sequence
- **Array Format**: Each bid/ask entry is `[price, size]` (both strings)

**Current Implementation**: ✓ To be implemented

---

#### 4. Get Klines (Candlestick Data)

- **Endpoint**: `GET /v5/market/kline`
- **Full URL**: `https://api.bybit.com/v5/market/kline?category=spot&symbol=BTCUSDT&interval=60&start=1670601600000&end=1670608800000`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (required, string): Trading pair
  - `interval` (required, string): Candlestick interval
  - `start` (optional, long): Start time in milliseconds
  - `end` (optional, long): End time in milliseconds
  - `limit` (optional, int): Number of records (default 200, max 1000)
- **Supported Intervals**:
  - `1`, `3`, `5`, `15`, `30` (minutes)
  - `60`, `120`, `240`, `360`, `720` (minutes)
  - `D` (day), `W` (week), `M` (month)
- **Response Array Format**: `[startTime, open, high, low, close, volume, turnover]`
  - Index 0: Start time in milliseconds
  - Index 1: Open price
  - Index 2: High price
  - Index 3: Low price
  - Index 4: Close price
  - Index 5: Trading volume
  - Index 6: Turnover value
- **Notes**:
  - Results sorted in reverse by startTime (newest first)
  - Maximum 1000 records per request

**Current Implementation**: ✓ To be implemented

---

#### 5. Get Trading Symbols

- **Endpoint**: `GET /v5/market/instruments-info`
- **Full URL**: `https://api.bybit.com/v5/market/instruments-info?category=spot`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (optional, string): Specific trading pair
  - `baseCoin` (optional, string): Filter by base coin
  - `limit` (optional, int): Data size per page (max 1000)
  - `cursor` (optional, string): Pagination cursor
- **Response Fields** (per symbol):
  - `symbol`: Trading pair
  - `baseCoin`, `quoteCoin`: Base and quote assets
  - `status`: Trading status ("Trading", "PreLaunch")
  - `lotSizeFilter`: Min/max order quantity
  - `priceFilter`: Min/max price, tick size
  - `innovation`: Whether it's an innovation zone token

**Current Implementation**: ✓ To be implemented

---

### SPOT Trading

#### 1. Create Order (Place Order)

- **Endpoint**: `POST /v5/order/create`
- **Full URL**: `https://api.bybit.com/v5/order/create`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Required Parameters** (JSON body):
  - `category` (string): "spot"
  - `symbol` (string): Trading pair (e.g., "BTCUSDT")
  - `side` (enum): "Buy" or "Sell"
  - `orderType` (enum): "Limit" or "Market"
  - `qty` (string): Order quantity

  **For limit orders**:
  - `price` (string): Order price

- **Optional Parameters**:
  - `timeInForce` (enum): "GTC" (Good Till Canceled), "IOC" (Immediate or Cancel), "FOK" (Fill or Kill), "PostOnly"
  - `orderLinkId` (string): Custom order ID (max 36 characters)
  - `isLeverage` (int): 0 (spot), 1 (spot margin)
  - `orderFilter` (string): "Order", "tpslOrder", "StopOrder"

- **Response**:
  ```json
  {
    "retCode": 0,
    "retMsg": "OK",
    "result": {
      "orderId": "6501cc87-b408-4f33-8542-ad234962c833",
      "orderLinkId": ""
    },
    "time": 1682963996331
  }
  ```

**Current Implementation**: ✓ To be implemented

---

#### 2. Cancel Order by Order ID

- **Endpoint**: `POST /v5/order/cancel`
- **Full URL**: `https://api.bybit.com/v5/order/cancel`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Required Parameters** (JSON body):
  - `category` (string): "spot"
  - `symbol` (string): Trading pair
  - `orderId` OR `orderLinkId` (string): One is required

**Current Implementation**: ✓ To be implemented

---

#### 3. Get Order by ID

- **Endpoint**: `GET /v5/order/realtime`
- **Full URL**: `https://api.bybit.com/v5/order/realtime?category=spot&symbol=BTCUSDT&orderId=xxx`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (optional, string): Trading pair
  - `orderId` OR `orderLinkId` (optional, string)

**Current Implementation**: ✓ To be implemented

---

#### 4. Get Open Orders

- **Endpoint**: `GET /v5/order/realtime`
- **Full URL**: `https://api.bybit.com/v5/order/realtime?category=spot&openOnly=0`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (optional, string): Filter by trading pair
  - `openOnly` (optional, int): 0 (all active orders, default)
  - `orderFilter` (optional, string): Filter type
  - `limit` (optional, int): Records per page (default 20, max 50)
  - `cursor` (optional, string): Pagination cursor

**Current Implementation**: ✓ To be implemented

---

#### 5. Get Order History

- **Endpoint**: `GET /v5/order/history`
- **Full URL**: `https://api.bybit.com/v5/order/history?category=spot`
- **Auth Required**: Yes (Private)
- **Query Parameters**:
  - `category` (required, string): "spot"
  - `symbol` (optional, string): Trading pair
  - `startTime` (optional, long): Start time (milliseconds)
  - `endTime` (optional, long): End time (milliseconds)
  - `limit` (optional, int): Records per page (default 20, max 50)
  - `cursor` (optional, string): Pagination cursor

**Current Implementation**: ✓ To be implemented

---

#### 6. Cancel All Orders

- **Endpoint**: `POST /v5/order/cancel-all`
- **Full URL**: `https://api.bybit.com/v5/order/cancel-all`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Required Parameters** (JSON body):
  - `category` (string): "spot"
  - `symbol` (optional, string): Cancel for specific symbol

**Current Implementation**: ✓ To be implemented

---

### SPOT Account

#### 1. Get Wallet Balance

- **Endpoint**: `GET /v5/account/wallet-balance`
- **Full URL**: `https://api.bybit.com/v5/account/wallet-balance?accountType=UNIFIED`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `accountType` (required, string): "UNIFIED"
  - `coin` (optional, string): Specific coin(s), comma-separated (e.g., "USDT,BTC")
- **Response Fields**:
  - Account-level:
    - `totalEquity`: Total equity in USD
    - `totalWalletBalance`: Aggregate wallet balance in USD
    - `totalAvailableBalance`: Usable balance
  - Per-coin:
    - `coin`: Asset symbol
    - `walletBalance`: Available balance
    - `locked`: Amount in open spot orders
    - `equity`: Asset equity
    - `unrealisedPnl`: Unrealized P&L

**Current Implementation**: ✓ To be implemented

---

## FUTURES Endpoints

### FUTURES Market Data

#### 1. Get Server Time

- Same as Spot: `GET /v5/market/time`

---

#### 2. Get Ticker (Single Symbol)

- **Endpoint**: `GET /v5/market/tickers`
- **Full URL**: `https://api.bybit.com/v5/market/tickers?category=linear&symbol=BTCUSDT`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "linear"
  - `symbol` (optional, string): Contract symbol (e.g., "BTCUSDT")
- **Response Fields** (additional to spot):
  - `markPrice`: Current mark price
  - `indexPrice`: Current index price
  - `fundingRate`: Current funding rate
  - `nextFundingTime`: Next funding timestamp (milliseconds)
  - `openInterest`: Total open interest
  - `openInterestValue`: Open interest value in USD

**Current Implementation**: ✓ To be implemented

---

#### 3. Get Orderbook

- **Endpoint**: `GET /v5/market/orderbook`
- **Full URL**: `https://api.bybit.com/v5/market/orderbook?category=linear&symbol=BTCUSDT&limit=50`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "linear"
  - `symbol` (required, string): Contract symbol
  - `limit` (optional, int): Orderbook depth levels (linear: 1-500, default 25)

**Current Implementation**: ✓ To be implemented

---

#### 4. Get Klines (Candlestick Data)

- **Endpoint**: `GET /v5/market/kline`
- **Full URL**: `https://api.bybit.com/v5/market/kline?category=linear&symbol=BTCUSDT&interval=60`
- **Auth Required**: No (Public)
- **Query Parameters**: Same as spot, but `category=linear`

**Current Implementation**: ✓ To be implemented

---

#### 5. Get Contracts Info (Instruments)

- **Endpoint**: `GET /v5/market/instruments-info`
- **Full URL**: `https://api.bybit.com/v5/market/instruments-info?category=linear`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "linear"
  - `symbol` (optional, string): Specific contract
  - `limit` (optional, int): Data size per page (max 1000)
  - `cursor` (optional, string): Pagination cursor
- **Response Fields** (per contract):
  - `symbol`: Contract symbol
  - `contractType`: "LinearPerpetual"
  - `status`: Contract status
  - `baseCoin`, `quoteCoin`, `settleCoin`: Currency info
  - `launchTime`: Launch timestamp
  - `deliveryTime`: Delivery timestamp (0 for perpetual)
  - `leverageFilter`: Min/max leverage
  - `lotSizeFilter`: Min/max order quantity

**Current Implementation**: ✓ To be implemented

---

#### 6. Get Funding Rate

- **Endpoint**: `GET /v5/market/funding/history`
- **Full URL**: `https://api.bybit.com/v5/market/funding/history?category=linear&symbol=BTCUSDT&limit=1`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `category` (required, string): "linear"
  - `symbol` (required, string): Contract symbol
  - `startTime` (optional, long): Start time (milliseconds)
  - `endTime` (optional, long): End time (milliseconds)
  - `limit` (optional, int): Records (default 200, max 200)
- **Response Fields**:
  - `symbol`: Contract symbol
  - `fundingRate`: Funding rate
  - `fundingRateTimestamp`: Funding timestamp (milliseconds)

**Current Implementation**: ✓ To be implemented

---

### FUTURES Trading

#### 1. Create Order (Place Order)

- **Endpoint**: `POST /v5/order/create`
- **Full URL**: `https://api.bybit.com/v5/order/create`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Required Parameters** (JSON body):
  - `category` (string): "linear"
  - `symbol` (string): Contract symbol (e.g., "BTCUSDT")
  - `side` (enum): "Buy" or "Sell"
  - `orderType` (enum): "Limit" or "Market"
  - `qty` (string): Order quantity

  **For limit orders**:
  - `price` (string): Order price

- **Optional Parameters**:
  - `timeInForce` (enum): "GTC", "IOC", "FOK", "PostOnly"
  - `orderLinkId` (string): Custom order ID
  - `positionIdx` (int): 0 (one-way), 1 (hedge buy), 2 (hedge sell)
  - `reduceOnly` (boolean): Reduce-only flag
  - `takeProfit` (string): Take profit price
  - `stopLoss` (string): Stop loss price

**Current Implementation**: ✓ To be implemented

---

#### 2. Cancel Order by Order ID

- Same as Spot: `POST /v5/order/cancel` with `category=linear`

---

#### 3. Get Order by ID

- Same as Spot: `GET /v5/order/realtime` with `category=linear`

---

#### 4. Get Open Orders

- Same as Spot: `GET /v5/order/realtime` with `category=linear`

---

#### 5. Get Order History

- Same as Spot: `GET /v5/order/history` with `category=linear`

---

#### 6. Cancel All Orders

- Same as Spot: `POST /v5/order/cancel-all` with `category=linear`

---

### FUTURES Account

#### 1. Get Position Info

- **Endpoint**: `GET /v5/position/list`
- **Full URL**: `https://api.bybit.com/v5/position/list?category=linear&symbol=BTCUSDT`
- **Auth Required**: Yes (Private)
- **HTTP Method**: GET
- **Query Parameters**:
  - `category` (required, string): "linear"
  - `symbol` (optional, string): Contract symbol
  - `settleCoin` (optional, string): Settlement coin
  - `limit` (optional, int): Data size per page (default 20, max 200)
  - `cursor` (optional, string): Pagination cursor
- **Response Fields**:
  - `symbol`: Contract symbol
  - `side`: "Buy" (long) or "Sell" (short)
  - `size`: Position size (always positive)
  - `avgPrice`: Average entry price
  - `positionValue`: Position value
  - `leverage`: Position leverage
  - `markPrice`: Current mark price
  - `liqPrice`: Liquidation price
  - `unrealisedPnl`: Unrealized P&L
  - `positionStatus`: "Normal", "Liq", "Adl"

**Current Implementation**: ✓ To be implemented

---

#### 2. Set Leverage

- **Endpoint**: `POST /v5/position/set-leverage`
- **Full URL**: `https://api.bybit.com/v5/position/set-leverage`
- **Auth Required**: Yes (Private)
- **HTTP Method**: POST
- **Required Parameters** (JSON body):
  - `category` (string): "linear"
  - `symbol` (string): Contract symbol
  - `buyLeverage` (string): Buy leverage
  - `sellLeverage` (string): Sell leverage

**Current Implementation**: ✓ To be implemented

---

#### 3. Get Wallet Balance (Futures)

- Same as Spot: `GET /v5/account/wallet-balance` with `accountType=UNIFIED`
- Unified account covers both spot and futures balances

---

## Summary

### Overall Status: ✅ COMPREHENSIVE

Bybit V5 API uses a unified architecture with the `category` parameter distinguishing between product types.

### Key Findings:

1. **Base URLs**: Single unified REST base URL for all products
2. **Endpoint Paths**: Consistent structure across spot/futures
3. **HTTP Methods**: GET for queries, POST for mutations
4. **Category Parameter**: Required for all endpoints ("spot", "linear", "inverse", "option")
5. **Symbol Formatting**: No separators - "BTCUSDT" for both spot and futures
6. **Response Format**: Consistent `retCode`, `retMsg`, `result`, `time` structure

### Differences from KuCoin:

1. **Unified Architecture**: Single set of endpoints with category parameter vs separate spot/futures URLs
2. **Symbol Format**: `BTCUSDT` (no hyphen) vs `BTC-USDT` (spot) / `XBTUSDTM` (futures)
3. **Response Structure**: `retCode`/`retMsg` vs `code`/`msg`
4. **Pagination**: Cursor-based vs page-based
5. **Timestamps**: Always milliseconds (no seconds in kline start time)

---

## Sources

Research compiled from the following official sources:

- [Bybit V5 API Introduction](https://bybit-exchange.github.io/docs/v5/intro)
- [Bybit V5 Integration Guide](https://bybit-exchange.github.io/docs/v5/guide)
- [Get Tickers](https://bybit-exchange.github.io/docs/v5/market/tickers)
- [Get Orderbook](https://bybit-exchange.github.io/docs/v5/market/orderbook)
- [Get Kline](https://bybit-exchange.github.io/docs/v5/market/kline)
- [Get Instruments Info](https://bybit-exchange.github.io/docs/v5/market/instrument)
- [Get Recent Public Trades](https://bybit-exchange.github.io/docs/v5/market/recent-trade)
- [Place Order](https://bybit-exchange.github.io/docs/v5/order/create-order)
- [Cancel Order](https://bybit-exchange.github.io/docs/v5/order/cancel-order)
- [Get Open & Closed Orders](https://bybit-exchange.github.io/docs/v5/order/open-order)
- [Get Order History](https://bybit-exchange.github.io/docs/v5/order/order-list)
- [Get Position Info](https://bybit-exchange.github.io/docs/v5/position)
- [Get Wallet Balance](https://bybit-exchange.github.io/docs/v5/account/wallet-balance)
- [Bybit V5 Changelog](https://bybit-exchange.github.io/docs/changelog/v5)
