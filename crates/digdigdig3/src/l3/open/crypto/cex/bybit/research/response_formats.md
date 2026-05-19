# Bybit V5 API Response Formats - Complete Reference

Research conducted: 2026-01-20

## Table of Contents

1. [General Response Structure](#general-response-structure)
2. [Price Response (Ticker)](#price-response-ticker)
3. [Klines/Candles Response](#klinescandles-response)
4. [Orderbook Response](#orderbook-response)
5. [Order Response](#order-response)
6. [Balance Response](#balance-response)
7. [Position Response (Futures)](#position-response-futures)
8. [Funding Rate Response](#funding-rate-response)
9. [Key Differences from KuCoin](#key-differences-from-kucoin)

---

## General Response Structure

### Success Response

**Format:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    // response data here
  },
  "retExtInfo": {},
  "time": 1702617474601
}
```

**Key Points:**
- Success code: `0` (integer, not string like KuCoin's "200000")
- Successful responses always have `retCode: 0` and `retMsg: "OK"`
- Data is wrapped in `result` field
- Additional info in `retExtInfo` (usually empty object)
- `time` field contains server timestamp in milliseconds

### Error Response

**Format:**
```json
{
  "retCode": 10001,
  "retMsg": "error message",
  "result": {},
  "retExtInfo": {},
  "time": 1702617474601
}
```

**Key Points:**
- Error code is NOT `0`
- Contains `retMsg` field with error description
- `result` may be empty object on error
- Common error codes:
  - `10001`: Parameter error
  - `10003`: Invalid API key
  - `10004`: Invalid sign
  - `10006`: Too many visits (rate limit)
  - `110001`: Order does not exist

### Timestamps

- All timestamps are in **milliseconds** (Unix epoch UTC)
- Server time endpoint: `/v5/market/time`
- Timestamp validation: `server_time - recv_window ≤ timestamp < server_time + 1000`

**Difference from KuCoin**: Bybit uses `retCode` (integer) vs KuCoin's `code` (string)

---

## Price Response (Ticker)

### Endpoint
`GET /v5/market/tickers?category=spot&symbol=BTCUSDT`

### Response Format

```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "category": "spot",
    "list": [
      {
        "symbol": "BTCUSDT",
        "lastPrice": "40000.00",
        "bid1Price": "39999.00",
        "bid1Size": "1.5",
        "ask1Price": "40001.00",
        "ask1Size": "2.3",
        "highPrice24h": "41000.00",
        "lowPrice24h": "39000.00",
        "volume24h": "12345.67",
        "turnover24h": "493827000.00",
        "usdIndexPrice": "40000.50"
      }
    ]
  },
  "time": 1702617474601
}
```

### Field Definitions (Spot)

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Trading pair |
| `lastPrice` | string | Last traded price |
| `bid1Price` | string | Best bid price |
| `bid1Size` | string | Best bid size/quantity |
| `ask1Price` | string | Best ask price |
| `ask1Size` | string | Best ask size/quantity |
| `highPrice24h` | string | 24h highest price |
| `lowPrice24h` | string | 24h lowest price |
| `volume24h` | string | 24h volume (base currency) |
| `turnover24h` | string | 24h turnover (quote currency) |
| `usdIndexPrice` | string | USD index price (spot specific) |

### Field Definitions (Futures/Linear)

Additional fields for `category=linear`:

| Field | Type | Description |
|-------|------|-------------|
| `markPrice` | string | Current mark price |
| `indexPrice` | string | Current index price |
| `fundingRate` | string | Current funding rate |
| `nextFundingTime` | string | Next funding timestamp (milliseconds) |
| `openInterest` | string | Total open interest |
| `openInterestValue` | string | Open interest value in USD |

### Notes
- All price/size fields are **strings** (need parsing)
- Data wrapped in `list` array (can contain multiple symbols)
- Ticker updates in real-time via REST or WebSocket

**Difference from KuCoin**:
- Bybit wraps data in `list` array
- Field names differ: `bid1Price` vs KuCoin's `bestBid`
- No confusing `buy`/`sell` naming (Bybit uses clear `bid1`/`ask1`)

---

## Klines/Candles Response

### Endpoint
`GET /v5/market/kline?category=spot&symbol=BTCUSDT&interval=60`

### Response Format

```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "category": "spot",
    "symbol": "BTCUSDT",
    "list": [
      [
        "1670608800000",  // [0] Start time (milliseconds)
        "40000.00",       // [1] Open price
        "40500.00",       // [2] High price
        "39900.00",       // [3] Low price
        "40200.00",       // [4] Close price
        "123.456",        // [5] Volume (base currency)
        "4960000.00"      // [6] Turnover (quote currency)
      ],
      ["1670605200000", "39800.00", "40100.00", "39700.00", "40000.00", "100.123", "3990000.00"]
    ]
  },
  "time": 1702617474601
}
```

### Array Structure

**Index Order:** `[time, open, high, low, close, volume, turnover]`

| Index | Field | Type | Description |
|-------|-------|------|-------------|
| 0 | time | string | Start time in **milliseconds** (not seconds) |
| 1 | open | string | Opening price |
| 2 | high | string | Highest price |
| 3 | low | string | Lowest price |
| 4 | close | string | Closing price |
| 5 | volume | string | Trading volume (base currency) |
| 6 | turnover | string | Trading amount (quote currency) |

### Data Ordering

- **Returns up to 1000 candles per request**
- **Sort order:** **Newest first** (descending by time)
- Users typically need to reverse for charting

### Notes
- Time is in **milliseconds** (unlike KuCoin spot which uses seconds)
- All numeric fields are **strings**
- Results sorted in reverse order by default

**Difference from KuCoin**:
- ✅ Bybit uses milliseconds for all timestamps
- ❌ KuCoin spot uses seconds for kline start time
- Bybit: `[time, open, high, low, close, volume, turnover]`
- KuCoin: `[time, open, close, high, low, volume, turnover]` (different order!)

**CRITICAL**: Open/close order differs from KuCoin!

---

## Orderbook Response

### Endpoint
`GET /v5/market/orderbook?category=spot&symbol=BTCUSDT&limit=50`

### Response Format

```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "s": "BTCUSDT",
    "b": [
      ["39999.00", "1.5"],
      ["39998.00", "2.3"]
    ],
    "a": [
      ["40001.00", "1.8"],
      ["40002.00", "2.1"]
    ],
    "ts": 1702617474601,
    "u": 123456,
    "seq": 7890123
  },
  "time": 1702617474601
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `s` | string | Symbol name |
| `b` | array | Bids (buy orders), sorted by price descending |
| `a` | array | Asks (sell orders), sorted by price ascending |
| `ts` | integer | Timestamp (milliseconds) when data was generated |
| `u` | integer | Update ID (sequential) |
| `seq` | integer | Cross sequence for comparing orderbook levels |

### Price/Size Array Format

Each bid/ask entry: `[price, size]`
- `[0]`: price (string)
- `[1]`: size/quantity (string)

### Ordering

- **Bids:** Sorted high to low (best bid first)
- **Asks:** Sorted low to high (best ask first)

### Notes
- Spot supports 1-200 levels
- Linear/inverse supports 1-500 levels
- WebSocket provides incremental updates

**Difference from KuCoin**:
- Short field names: `s`, `b`, `a` vs KuCoin's `symbol`, `bids`, `asks`
- Both price and size are strings (same as KuCoin)
- `ts` field always in milliseconds (KuCoin has `ts` for futures, `time` for spot)

---

## Order Response

### Place Order Response

**Endpoint:** `POST /v5/order/create`

**Response:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "orderId": "6501cc87-b408-4f33-8542-ad234962c833",
    "orderLinkId": "custom-order-123"
  },
  "retExtInfo": {},
  "time": 1682963996331
}
```

### Get Order Details Response

**Endpoint:** `GET /v5/order/realtime?category=spot&orderId=xxx`

**Response:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "list": [
      {
        "orderId": "6501cc87-b408-4f33-8542-ad234962c833",
        "orderLinkId": "custom-order-123",
        "symbol": "BTCUSDT",
        "side": "Buy",
        "orderType": "Limit",
        "price": "40000.00",
        "qty": "0.01",
        "leavesQty": "0.005",
        "cumExecQty": "0.005",
        "cumExecValue": "200.00",
        "avgPrice": "40000.00",
        "orderStatus": "PartiallyFilled",
        "timeInForce": "GTC",
        "createdTime": "1682963996331",
        "updatedTime": "1682963998331"
      }
    ]
  },
  "time": 1682963998331
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `orderId` | string | System-assigned order ID |
| `orderLinkId` | string | User-provided custom order ID |
| `symbol` | string | Trading pair |
| `side` | string | "Buy" or "Sell" |
| `orderType` | string | "Limit", "Market", etc. |
| `price` | string | Order price |
| `qty` | string | Order quantity |
| `leavesQty` | string | Remaining quantity (unfilled) |
| `cumExecQty` | string | Cumulative filled quantity |
| `cumExecValue` | string | Total filled value (quote currency) |
| `avgPrice` | string | Average execution price |
| `orderStatus` | string | Order status (see below) |
| `timeInForce` | string | "GTC", "IOC", "FOK", "PostOnly" |
| `createdTime` | string | Creation timestamp (milliseconds) |
| `updatedTime` | string | Update timestamp (milliseconds) |

### Order Status Values

Bybit uses **explicit status strings**:

- `Created`: Order created but not yet triggered
- `New`: Order created
- `Rejected`: Order rejected
- `PartiallyFilled`: Order partially filled
- `Filled`: Order fully filled
- `Cancelled`: Order cancelled
- `PendingCancel`: Cancellation pending

**Status logic is straightforward** - use `orderStatus` field directly.

**Difference from KuCoin**:
- ✅ Explicit status strings (simpler than KuCoin's boolean flags)
- ❌ KuCoin uses `isActive`, `cancelExist` flags
- Field names: `leavesQty` vs KuCoin's `remainSize`
- Timestamps as strings vs integers in KuCoin

---

## Balance Response

### Wallet Balance Response

**Endpoint:** `GET /v5/account/wallet-balance?accountType=UNIFIED`

**Response:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "list": [
      {
        "totalEquity": "50000.00",
        "accountIMRate": "0.0250",
        "totalMarginBalance": "50000.00",
        "totalInitialMargin": "1250.00",
        "accountType": "UNIFIED",
        "totalAvailableBalance": "48750.00",
        "accountMMRate": "0",
        "totalPerpUPL": "0.00",
        "totalWalletBalance": "50000.00",
        "totalMaintenanceMargin": "0.00",
        "coin": [
          {
            "coin": "USDT",
            "equity": "10000.00",
            "usdValue": "10000.00",
            "walletBalance": "10000.00",
            "availableToWithdraw": "9800.00",
            "locked": "200.00",
            "unrealisedPnl": "0.00",
            "borrowAmount": "0.00",
            "totalOrderIM": "200.00",
            "totalPositionIM": "0.00",
            "totalPositionMM": "0.00"
          },
          {
            "coin": "BTC",
            "equity": "1.0",
            "usdValue": "40000.00",
            "walletBalance": "1.0",
            "availableToWithdraw": "1.0",
            "locked": "0.0",
            "unrealisedPnl": "0.00"
          }
        ]
      }
    ]
  },
  "time": 1702617474601
}
```

### Account-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `totalEquity` | string | Total equity in USD |
| `totalWalletBalance` | string | Aggregate wallet balance in USD |
| `totalAvailableBalance` | string | Usable balance |
| `totalPerpUPL` | string | Unrealized P&L from perpetuals |
| `accountType` | string | "UNIFIED" |

### Per-Coin Fields

| Field | Type | Description |
|-------|------|-------------|
| `coin` | string | Asset/coin symbol |
| `equity` | string | Asset equity (walletBalance + unrealisedPnl) |
| `walletBalance` | string | Available balance for the asset |
| `locked` | string | Amount in open spot orders |
| `availableToWithdraw` | string | Withdrawable amount |
| `unrealisedPnl` | string | Unrealized profit/loss |
| `borrowAmount` | string | Borrowed amount (margin) |
| `usdValue` | string | USD value of asset |

### Formula
```
equity = walletBalance + unrealisedPnl
availableBalance = equity - locked - margin
```

**Difference from KuCoin**:
- Unified account structure (spot + futures combined)
- More detailed margin fields
- Field names: `walletBalance` vs KuCoin's `available`
- Bybit returns only non-zero assets by default

---

## Position Response (Futures)

### Get Position List Response

**Endpoint:** `GET /v5/position/list?category=linear`

**Response:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "list": [
      {
        "positionIdx": 0,
        "symbol": "BTCUSDT",
        "side": "Buy",
        "size": "0.5",
        "positionValue": "20000.00",
        "avgPrice": "40000.00",
        "markPrice": "40100.00",
        "liqPrice": "35000.00",
        "bustPrice": "34500.00",
        "positionIM": "400.00",
        "positionMM": "100.00",
        "unrealisedPnl": "50.00",
        "cumRealisedPnl": "120.00",
        "positionStatus": "Normal",
        "leverage": "50",
        "updatedTime": "1702617474601",
        "createdTime": "1702600000000"
      }
    ],
    "nextPageCursor": ""
  },
  "time": 1702617474601
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Contract symbol |
| `side` | string | "Buy" (long) or "Sell" (short) |
| `size` | string | Position size (always positive) |
| `avgPrice` | string | Average entry price |
| `positionValue` | string | Position value |
| `markPrice` | string | Current mark price |
| `liqPrice` | string | Liquidation price |
| `bustPrice` | string | Bankruptcy price |
| `leverage` | string | Position leverage |
| `unrealisedPnl` | string | Unrealized profit/loss |
| `cumRealisedPnl` | string | Cumulative realized P&L |
| `positionStatus` | string | "Normal", "Liq" (liquidating), "Adl" (auto-deleveraging) |
| `positionIdx` | integer | Position index: 0 (one-way), 1 (buy hedge), 2 (sell hedge) |
| `updatedTime` | string | Last update timestamp (milliseconds) |
| `createdTime` | string | Position open timestamp (milliseconds) |

### Notes
- `size` sign determines position side (but always shown positive)
- `side` field explicitly shows "Buy" or "Sell"
- All P&L values in settlement currency (usually USDT)
- Results wrapped in `list` array with pagination cursor

**Difference from KuCoin**:
- Explicit `side` field (KuCoin uses signed `currentQty`)
- More detailed margin fields (`positionIM`, `positionMM`)
- Field names: `avgPrice` vs KuCoin's `avgEntryPrice`
- Pagination via cursor vs page number

---

## Funding Rate Response

### Get Funding Rate History Response

**Endpoint:** `GET /v5/market/funding/history?category=linear&symbol=BTCUSDT&limit=1`

**Response:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "category": "linear",
    "list": [
      {
        "symbol": "BTCUSDT",
        "fundingRate": "0.0001",
        "fundingRateTimestamp": "1702617600000"
      }
    ]
  },
  "time": 1702617474601
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Contract symbol |
| `fundingRate` | string | Funding rate |
| `fundingRateTimestamp` | string | Funding timestamp (milliseconds) |

### Notes
- Funding rate is typically +/- 0.01% to 0.3%
- Settlement every 8 hours for most perpetual contracts
- Positive rate: Longs pay shorts
- Negative rate: Shorts pay longs

**Difference from KuCoin**:
- Simpler structure (no predicted value in basic endpoint)
- Field name: `fundingRateTimestamp` vs KuCoin's `timePoint`
- Always in `list` array format

---

## Key Differences from KuCoin

### Response Structure

| Feature | Bybit V5 | KuCoin |
|---------|----------|---------|
| Success code | `retCode: 0` (integer) | `code: "200000"` (string) |
| Success message | `retMsg: "OK"` | No `msg` field on success |
| Data wrapper | `result` | `data` |
| Error message | `retMsg` | `msg` |
| Timestamp field | `time` (milliseconds) | `data.time` (varies) |

### Field Naming

| Data Type | Bybit V5 | KuCoin |
|-----------|----------|---------|
| Bid price | `bid1Price` | `bestBid` or `buy` |
| Ask price | `ask1Price` | `bestAsk` or `sell` |
| Order ID | `orderId` | `id` or `orderId` |
| Filled quantity | `cumExecQty` | `dealSize` |
| Remaining quantity | `leavesQty` | `remainSize` |
| Entry price | `avgPrice` | `avgEntryPrice` |
| Unrealized PnL | `unrealisedPnl` | `unrealisedPnl` (same) |

### Kline Order

**CRITICAL DIFFERENCE**:
- Bybit: `[time, open, high, low, close, volume, turnover]`
- KuCoin: `[time, open, close, high, low, volume, turnover]`

**High and close positions are swapped!**

### Timestamp Formats

- Bybit: **All** timestamps in milliseconds
- KuCoin: Spot kline start times in seconds, everything else in milliseconds

### Data Wrapping

- Bybit: Most responses wrap data in `list` array
- KuCoin: Mixed - some use arrays, some use objects directly

---

## Sources

Research compiled from official Bybit V5 API documentation:

- [Bybit V5 API Introduction](https://bybit-exchange.github.io/docs/v5/intro)
- [Get Tickers](https://bybit-exchange.github.io/docs/v5/market/tickers)
- [Get Orderbook](https://bybit-exchange.github.io/docs/v5/market/orderbook)
- [Get Kline](https://bybit-exchange.github.io/docs/v5/market/kline)
- [Place Order](https://bybit-exchange.github.io/docs/v5/order/create-order)
- [Get Open & Closed Orders](https://bybit-exchange.github.io/docs/v5/order/open-order)
- [Get Order History](https://bybit-exchange.github.io/docs/v5/order/order-list)
- [Get Wallet Balance](https://bybit-exchange.github.io/docs/v5/account/wallet-balance)
- [Get Position Info](https://bybit-exchange.github.io/docs/v5/position)
- [Get Funding History](https://bybit-exchange.github.io/docs/v5/market/history-fund-rate)
- [Bybit API Cheat Sheet](https://vezgo.com/blog/bybit-api-cheat-sheet-for-developers/)

---

**Research completed**: 2026-01-20
**Critical finding**: Kline array order differs from KuCoin - must handle carefully in parser
