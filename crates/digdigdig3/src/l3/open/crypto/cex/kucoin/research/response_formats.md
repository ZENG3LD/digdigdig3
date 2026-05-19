# KuCoin API Response Formats - Complete Reference

Research conducted: 2026-01-20

## Table of Contents

1. [General Response Structure](#general-response-structure)
2. [Price Response (Level1 Ticker)](#price-response-level1-ticker)
3. [Klines/Candles Response](#klinescandles-response)
4. [Orderbook Response](#orderbook-response)
5. [Ticker (24h Stats) Response](#ticker-24h-stats-response)
6. [Order Response](#order-response)
7. [Balance Response](#balance-response)
8. [Position Response (Futures)](#position-response-futures)
9. [Funding Rate Response](#funding-rate-response)
10. [Comparison with parser.rs](#comparison-with-parserrs)

---

## General Response Structure

### Success Response

**Format:**
```json
{
  "code": "200000",
  "data": {
    // response data here
  }
}
```

**Key Points:**
- Success code: `"200000"` (string, not integer)
- Successful responses always have HTTP 200 and `code: "200000"`
- Data is wrapped in `data` field
- No `msg` field on success

### Error Response

**Format:**
```json
{
  "code": "error_code",
  "msg": "error message"
}
```

**Key Points:**
- Error code is NOT `"200000"`
- Contains `msg` field with error description
- No `data` field on error

### Timestamps

- All timestamps are in **milliseconds** (Unix epoch UTC)
- Server time endpoint: `/api/v1/timestamp`
- Request timestamps must be within 5 seconds of server time
- Matching engine operates at nanosecond precision internally

**Our parser.rs:** Correctly expects `"code": "200000"` and extracts `data` field.

---

## Price Response (Level1 Ticker)

### Endpoint
`GET /api/v1/market/orderbook/level1?symbol=BTC-USDT`

### Response Format

```json
{
  "code": "200000",
  "data": {
    "sequence": "1550467636704",
    "price": "0.03715005",
    "size": "0.17",
    "bestBid": "0.03710768",
    "bestBidSize": "3.803",
    "bestAsk": "0.03715004",
    "bestAskSize": "1.788",
    "time": 1550653727731
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `sequence` | string | Ticker sequence number |
| `price` | string | Last traded price |
| `size` | string | Last traded size |
| `bestBid` | string | Best bid price |
| `bestBidSize` | string | Best bid size/quantity |
| `bestAsk` | string | Best ask price |
| `bestAskSize` | string | Best ask size/quantity |
| `time` | integer | Timestamp in milliseconds |

### Notes
- All price/size fields are **strings** (need parsing)
- `time` is an **integer** in milliseconds
- Sequence number is used for synchronization with WebSocket feeds

**Our parser.rs:** Currently uses `parse_price()` which only extracts the `"price"` field. Does not extract `bestBid`, `bestAsk`, or sizes. This is likely correct for the simple price endpoint, but may need adjustment if using level1 endpoint.

---

## Klines/Candles Response

### Endpoint
`GET /api/v1/market/candles?symbol=BTC-USDT&type=1hour`

### Response Format

```json
{
  "code": "200000",
  "data": [
    [
      "1545904980",  // [0] Start time of the candle cycle (SECONDS)
      "0.058",       // [1] Opening price
      "0.049",       // [2] Closing price
      "0.058",       // [3] Highest price
      "0.049",       // [4] Lowest price
      "0.018",       // [5] Transaction volume (base currency)
      "0.000945"     // [6] Transaction amount (quote currency / turnover)
    ],
    ["1545904920", "0.057", "0.058", "0.059", "0.056", "0.021", "0.001197"]
  ]
}
```

### Array Structure

**Index Order:** `[time, open, close, high, low, volume, turnover]`

| Index | Field | Type | Description |
|-------|-------|------|-------------|
| 0 | time | string | Start time in **SECONDS** (not milliseconds) |
| 1 | open | string | Opening price |
| 2 | close | string | Closing price |
| 3 | high | string | Highest price |
| 4 | low | string | Lowest price |
| 5 | volume | string | Trading volume (base currency) |
| 6 | turnover | string | Trading amount (quote currency) |

### Data Ordering

- **Returns up to 1500 candles per request**
- **Sort order:** Likely **newest first** (descending by time)
- Users typically need to sort ascending for charting

### Notes
- Time is in **SECONDS**, not milliseconds (multiply by 1000 for ms)
- All numeric fields are **strings**
- Order endpoints return data in descending order by default

**Our parser.rs:**
- **CORRECT:** Parses indices correctly: `[1]=open, [2]=close, [3]=high, [4]=low, [5]=volume, [6]=turnover`
- **CORRECT:** Multiplies time by 1000: `candle[0] * 1000` (seconds to ms)
- **CORRECT:** Reverses array with `klines.reverse()` assuming newest-first
- **CORRECT:** Uses `quote_volume` for turnover field
- **Note:** Comment says "KuCoin format: [time, open, close, high, low, volume, turnover]" - accurate

---

## Orderbook Response

### Endpoint
`GET /api/v3/market/orderbook/level2?symbol=XBTUSDM` (Futures)
`GET /api/v1/market/orderbook/level2_100?symbol=BTC-USDT` (Spot, partial)

### Response Format

```json
{
  "code": "200000",
  "data": {
    "symbol": "XBTUSDM",
    "sequence": 100,
    "asks": [
      ["5000.0", 1000],
      ["6000.0", 1983]
    ],
    "bids": [
      ["3200.0", 800],
      ["3100.0", 100]
    ],
    "ts": 1604643655040584408
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Trading pair symbol |
| `sequence` | integer | Sequence number for synchronization |
| `asks` | array | Sell orders [price, size] |
| `bids` | array | Buy orders [price, size] |
| `ts` or `time` | integer | Timestamp (nanoseconds or milliseconds) |

### Price/Size Array Format

Each bid/ask entry: `[price, size]`
- `[0]`: price (string or number)
- `[1]`: size/quantity (integer or number)

### Ordering

- **Bids:** Sorted high to low (best bid first)
- **Asks:** Sorted low to high (best ask first)

### Notes
- Level 2 aggregates all orders at each price level
- Sequence number critical for WebSocket sync
- Full orderbook can be large; use partial endpoints for better performance

**Our parser.rs:**
- **CORRECT:** Parses `bids` and `asks` as arrays of `[price, size]` pairs
- **CORRECT:** Uses `parse_f64` for both price (index 0) and size (index 1)
- **ISSUE:** Looks for `data.get("time")` but response may use `ts` field instead
- **ISSUE:** Parses `sequence` as string with `get_str()`, but it's an integer in the response
- **Recommendation:** Use `data.get("ts").or_else(|| data.get("time"))` for timestamp
- **Recommendation:** Parse sequence as `as_i64()` or `as_u64()`, not as string

---

## Ticker (24h Stats) Response

### Endpoint
`GET /api/v1/market/allTickers` or `GET /api/v1/market/stats?symbol=BTC-USDT`

### Response Format (Single Ticker)

```json
{
  "code": "200000",
  "data": {
    "time": 1602832092060,
    "symbol": "BTC-USDT",
    "symbolName": "BTC-USDT",
    "buy": "11328.9",
    "sell": "11329",
    "high": "11610",
    "low": "11200",
    "vol": "2282.70993217",
    "last": "11328.9",
    "volValue": "25550000",
    "changePrice": "100.5",
    "changeRate": "0.0089"
  }
}
```

### Response Format (All Tickers)

```json
{
  "code": "200000",
  "data": {
    "time": 1602832092060,
    "ticker": [
      {
        "symbol": "BTC-USDT",
        "buy": "11328.9",
        "sell": "11329",
        ...
      }
    ]
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Trading pair |
| `last` | string | Last traded price |
| `buy` | string | Best ask price (confusing naming!) |
| `sell` | string | Best bid price (confusing naming!) |
| `high` | string | 24h highest price |
| `low` | string | 24h lowest price |
| `vol` | string | 24h volume (base currency) |
| `volValue` | string | 24h volume (quote currency) |
| `changePrice` | string | 24h price change (absolute) |
| `changeRate` | string | 24h price change (percentage as decimal) |
| `time` | integer | Timestamp in milliseconds |

### CRITICAL NAMING ISSUE

**WARNING:** KuCoin's naming is counterintuitive!
- `buy` = **bestAsk** (price to buy AT, which is the seller's ask)
- `sell` = **bestBid** (price to sell AT, which is the buyer's bid)

This is from the **taker's perspective** (what price you pay to buy/sell), not the order book side.

**Our parser.rs:**
- **CORRECT:** Maps `"last"` to `last_price`
- **CORRECT:** Maps `"buy"` to `bid_price` - Wait, this is INCORRECT!
- **INCORRECT:** `"buy"` should map to `ask_price` (best ask)
- **INCORRECT:** `"sell"` should map to `bid_price` (best bid)
- **CORRECT:** Maps `"high"` to `high_24h`
- **CORRECT:** Maps `"low"` to `low_24h`
- **CORRECT:** Maps `"vol"` to `volume_24h`
- **CORRECT:** Maps `"volValue"` to `quote_volume_24h`
- **CORRECT:** Maps `"changePrice"` to `price_change_24h`
- **CORRECT:** Maps `"changeRate"` to `price_change_percent_24h` and multiplies by 100
- **CORRECT:** Extracts timestamp from `"time"` field

**CRITICAL BUG:** Lines 142-143 swap bid/ask:
```rust
bid_price: Self::get_f64(data, "buy"),   // WRONG! Should be "sell"
ask_price: Self::get_f64(data, "sell"),  // WRONG! Should be "buy"
```

**Should be:**
```rust
bid_price: Self::get_f64(data, "sell"),  // bestBid = sell price
ask_price: Self::get_f64(data, "buy"),   // bestAsk = buy price
```

---

## Order Response

### Place Order Response

**Endpoint:** `POST /api/v1/orders`

**Response:**
```json
{
  "code": "200000",
  "data": {
    "orderId": "5bd6e9286d99522a52e458de"
  }
}
```

### Get Order Details Response

**Endpoint:** `GET /api/v1/orders/{orderId}` or `GET /api/v1/order/client-order/{clientOid}`

**Response:**
```json
{
  "code": "200000",
  "data": {
    "id": "5c35c02703aa673ceec2a168",
    "symbol": "BTC-USDT",
    "opType": "DEAL",
    "type": "limit",
    "side": "buy",
    "price": "10",
    "size": "2",
    "funds": "0",
    "dealFunds": "0.166",
    "dealSize": "2",
    "fee": "0",
    "feeCurrency": "USDT",
    "stp": "",
    "stop": "",
    "stopTriggered": false,
    "stopPrice": "0",
    "timeInForce": "GTC",
    "postOnly": false,
    "hidden": false,
    "iceberg": false,
    "visibleSize": "0",
    "cancelAfter": 0,
    "channel": "IOS",
    "clientOid": "user-order-123",
    "remark": null,
    "tags": null,
    "isActive": false,
    "cancelExist": false,
    "createdAt": 1547026471000,
    "updatedAt": 1547026471001,
    "tradeType": "TRADE"
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Server-assigned order ID |
| `orderId` | string | Same as `id` (used in create response) |
| `clientOid` | string | Client-provided order ID |
| `symbol` | string | Trading pair |
| `side` | string | `buy` or `sell` |
| `type` | string | `limit`, `market`, etc. |
| `price` | string | Order price |
| `size` | string | Order quantity |
| `dealSize` | string | Filled quantity |
| `dealFunds` | string | Total filled value (quote currency) |
| `stopPrice` | string | Stop price (if stop order) |
| `isActive` | boolean | Is order still active? |
| `cancelExist` | boolean | Was order cancelled? |
| `createdAt` | integer | Creation timestamp (milliseconds) |
| `updatedAt` | integer | Update timestamp (milliseconds) |
| `fee` | string | Trading fee |
| `feeCurrency` | string | Fee currency |

### Order Status Values

KuCoin uses **boolean flags** instead of explicit status strings:

- `isActive: true` → Order is **active** (waiting to be matched)
- `isActive: false` → Order is **done** (filled, cancelled, or expired)
- `cancelExist: true` → Order was **cancelled**
- `dealSize > 0 && cancelExist: true` → **Partially filled then cancelled**
- `dealSize >= size && isActive: false` → **Filled**

Status logic:
```
if cancelExist:
    if dealSize > 0: PartiallyFilled (then cancelled)
    else: Cancelled
elif not isActive:
    if dealSize >= size: Filled
    else: PartiallyFilled
elif dealSize > 0:
    PartiallyFilled
else:
    New/Active
```

### List Orders Response

**Endpoint:** `GET /api/v1/orders?status=active`

**Response:**
```json
{
  "code": "200000",
  "data": {
    "currentPage": 1,
    "pageSize": 50,
    "totalNum": 100,
    "totalPage": 2,
    "items": [
      {
        "id": "...",
        "symbol": "BTC-USDT",
        ...
      }
    ]
  }
}
```

Orders are wrapped in `items` array within pagination metadata.

**Our parser.rs:**
- **CORRECT:** Uses `id` or `orderId` (line 193-194)
- **CORRECT:** Parses `clientOid` as `client_order_id`
- **CORRECT:** Uses `size` for quantity
- **CORRECT:** Uses `dealSize` for filled_quantity
- **CORRECT:** Calculates average price from `dealFunds / dealSize`
- **CORRECT:** Parses timestamps `createdAt` and `updatedAt`
- **CORRECT:** Status logic using `isActive`, `cancelExist`, `dealSize` (lines 221-244)
- **CORRECT:** Handles `items` array wrapping (lines 251-254)
- **CORRECT:** Extracts `orderId` from create response (line 264)

---

## Balance Response

### Spot Balance Response

**Endpoint:** `GET /api/v1/accounts?type=trade`

**Response:**
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
    },
    {
      "id": "5bd6e9216d99522a52e458d6",
      "currency": "BTC",
      "type": "trade",
      "balance": "1234356",
      "available": "1234356",
      "holds": "0"
    }
  ]
}
```

### Single Account Detail Response

**Endpoint:** `GET /api/v1/accounts/{accountId}`

**Response:**
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

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Asset/coin symbol |
| `balance` | string | Total assets |
| `available` | string | Available assets (free) |
| `holds` | string | Frozen/locked assets |
| `type` | string | Account type: `main`, `trade`, `margin` |
| `id` | string | Account ID |

### Formula
```
balance = available + holds
```

**Our parser.rs:**
- **CORRECT:** Maps `currency` to `asset`
- **CORRECT:** Maps `available` to `free`
- **CORRECT:** Maps `holds` to `locked`
- **CORRECT:** Calculates `total = free + locked`
- **CORRECT:** Returns array of balances

---

## Position Response (Futures)

### Get Position List Response

**Endpoint:** `GET /api/v1/positions`

**Response:**
```json
{
  "code": "200000",
  "data": [
    {
      "id": "615164f00000000001",
      "symbol": "XBTUSDTM",
      "autoDeposit": false,
      "maintMarginReq": 0.005,
      "riskLimit": 2000000,
      "realLeverage": 1.0,
      "crossMode": false,
      "delevPercentage": 0.52,
      "openingTimestamp": 1632571200000,
      "currentTimestamp": 1632571800000,
      "currentQty": 1,
      "currentCost": 69.814,
      "currentComm": 0.0419,
      "unrealisedCost": 69.814,
      "realisedGrossCost": 0.0,
      "realisedCost": 0.0419,
      "isOpen": true,
      "markPrice": 69814.85,
      "markValue": 69.8148,
      "posCost": 69.814,
      "posCross": 0.0,
      "posInit": 69.814,
      "posComm": 0.0454,
      "posLoss": 0.0,
      "posMargin": 69.8594,
      "posMaint": 0.3839,
      "maintMargin": 69.8933,
      "realisedGrossPnl": 0.0,
      "realisedPnl": -0.0419,
      "unrealisedPnl": 0.0008,
      "unrealisedPnlPcnt": 0.0,
      "unrealisedRoePcnt": 0.0011,
      "avgEntryPrice": 69814.85,
      "liquidationPrice": 0.12,
      "bankruptPrice": 0.1,
      "settleCurrency": "USDT",
      "maintainMargin": 0.02,
      "riskLimitLevel": 1
    }
  ]
}
```

### Get Position Details Response

**Endpoint:** `GET /api/v1/position?symbol=XBTUSDTM`

**Response:** Same structure as individual position in list (single object in `data`, not array)

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Contract symbol |
| `currentQty` | number | Current position quantity (positive=long, negative=short) |
| `avgEntryPrice` | number | Average entry price |
| `markPrice` | number | Current mark price |
| `unrealisedPnl` | number | Unrealized profit/loss |
| `realisedPnl` | number | Realized profit/loss |
| `realLeverage` | number | Actual leverage |
| `maintMargin` | number | Maintenance margin |
| `liquidationPrice` | number | Liquidation price |
| `crossMode` | boolean | Cross margin mode (true) or isolated (false) |
| `isOpen` | boolean | Is position open? |

### Notes
- `currentQty` sign determines position side:
  - Positive → Long
  - Negative → Short
  - Zero → No position
- All PnL values in settlement currency (usually USDT or BTC)

**Our parser.rs:**
- **CORRECT:** Uses `currentQty` for quantity and determines side from sign
- **CORRECT:** Maps `avgEntryPrice` to `entry_price`
- **CORRECT:** Maps `markPrice` to `mark_price`
- **CORRECT:** Maps `unrealisedPnl` to `unrealized_pnl` (note: British spelling)
- **CORRECT:** Maps `realisedPnl` to `realized_pnl`
- **CORRECT:** Maps `realLeverage` to `leverage` (casts to u32)
- **CORRECT:** Maps `liquidationPrice` to `liquidation_price`
- **CORRECT:** Maps `maintMargin` to `margin`
- **CORRECT:** Skips positions with zero quantity
- **CORRECT:** Takes absolute value of quantity for storage

---

## Futures Account Balance Response

**Endpoint:** `GET /api/v1/account-overview`

**Response:**
```json
{
  "code": "200000",
  "data": {
    "accountEquity": 99.8999305281,
    "unrealisedPNL": 0,
    "marginBalance": 99.8999305281,
    "positionMargin": 0,
    "orderMargin": 0,
    "frozenFunds": 0,
    "availableBalance": 99.8999305281,
    "currency": "USDT",
    "riskRatio": 0
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Settlement currency |
| `availableBalance` | number | Available balance |
| `frozenFunds` | number | Frozen funds (in orders + positions) |
| `accountEquity` | number | Total account equity |
| `unrealisedPNL` | number | Unrealized PnL |
| `marginBalance` | number | Margin balance |
| `positionMargin` | number | Position margin |
| `orderMargin` | number | Order margin |
| `riskRatio` | number | Risk ratio |

**Our parser.rs:**
- **CORRECT:** Maps `currency` to `asset`
- **CORRECT:** Maps `availableBalance` to `free`
- **CORRECT:** Maps `frozenFunds` to `locked`
- **CORRECT:** Calculates total as `free + locked`
- **CORRECT:** Returns as single-element Vec<Balance>

---

## Funding Rate Response

### Current Funding Rate Response

**Endpoint:** `GET /api/v1/funding-rate/{symbol}/current`

**Response:**
```json
{
  "code": "200000",
  "data": {
    "symbol": ".XBTUSDTMFPI8H",
    "granularity": 28800000,
    "timePoint": 1731441600000,
    "value": 0.000641,
    "predictedValue": 0.000052,
    "fundingRateCap": 0.003,
    "fundingRateFloor": -0.003
  }
}
```

### Public Funding History Response

**Endpoint:** `GET /api/v1/contract/funding-rates?symbol=XBTUSDTM&from=...&to=...`

**Response:**
```json
{
  "code": "200000",
  "data": [
    {
      "symbol": "XBTUSDTM",
      "fundingRate": 0.0001,
      "timepoint": 1731441600000
    }
  ]
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Funding rate symbol (may differ from contract symbol) |
| `value` or `fundingRate` | number | Current funding rate |
| `predictedValue` | number | Predicted next funding rate |
| `timePoint` or `timepoint` | integer | Timestamp in milliseconds |
| `granularity` | integer | Settlement interval (milliseconds) |
| `fundingRateCap` | number | Maximum funding rate |
| `fundingRateFloor` | number | Minimum funding rate |

### Notes
- Funding rate is typically +/- 0.01% to 0.3%
- Settlement every 8 hours (28800000 ms) for most contracts
- Formula: `Funding Rate = Clamp(MA[(BestBid+BestAsk)/2 - Index] / Index + Interest, floor, cap)`

**Our parser.rs:**
- **CORRECT:** Maps `symbol` to `symbol`
- **CORRECT:** Maps `value` to `rate` using `require_f64`
- **CORRECT:** Maps `timePoint` to `timestamp`
- **MISSING:** Does not extract `predictedValue` (could be useful)
- **NOTE:** Sets `next_funding_time` to `None` (could calculate from timePoint + granularity)

---

## Comparison with parser.rs

### Summary of Issues Found

| Issue | Severity | Location | Description |
|-------|----------|----------|-------------|
| Bid/Ask swap | **CRITICAL** | `parse_ticker()` L142-143 | `buy` and `sell` fields are swapped |
| Orderbook sequence type | Minor | `parse_orderbook()` L131 | Parses as string, should be integer |
| Orderbook timestamp field | Minor | `parse_orderbook()` L128 | Uses `time`, should also check `ts` |
| Missing predicted funding | Info | `parse_funding_rate()` | Could extract `predictedValue` |

### Detailed Analysis

#### 1. CRITICAL: Bid/Ask Swap in Ticker (Lines 142-143)

**Current (WRONG):**
```rust
bid_price: Self::get_f64(data, "buy"),   // buy is bestAsk!
ask_price: Self::get_f64(data, "sell"),  // sell is bestBid!
```

**Should be:**
```rust
bid_price: Self::get_f64(data, "sell"),  // sell = bestBid
ask_price: Self::get_f64(data, "buy"),   // buy = bestAsk
```

**Explanation:** KuCoin uses taker perspective naming:
- `buy` = price you pay to buy = bestAsk (seller's ask)
- `sell` = price you get to sell = bestBid (buyer's bid)

#### 2. Minor: Orderbook Sequence Type (Line 131)

**Current:**
```rust
sequence: Self::get_str(data, "sequence").map(String::from),
```

**Should be:**
```rust
sequence: data.get("sequence")
    .and_then(|s| s.as_i64().or_else(|| s.as_str()?.parse().ok()))
    .map(|n| n.to_string()),
```

KuCoin returns sequence as integer, not string. Parser should handle both.

#### 3. Minor: Orderbook Timestamp Field (Line 128)

**Current:**
```rust
timestamp: data.get("time").and_then(|t| t.as_i64()).unwrap_or(0),
```

**Should be:**
```rust
timestamp: data.get("ts")
    .or_else(|| data.get("time"))
    .and_then(|t| t.as_i64())
    .unwrap_or(0),
```

Futures uses `ts` (nanoseconds), spot uses `time` (milliseconds).

#### 4. Info: Missing Predicted Funding Rate

Could add:
```rust
pub struct FundingRate {
    pub symbol: String,
    pub rate: f64,
    pub predicted_rate: Option<f64>,  // Add this
    pub next_funding_time: Option<i64>,
    pub timestamp: i64,
}
```

Parse as:
```rust
predicted_rate: Self::get_f64(data, "predictedValue"),
next_funding_time: data.get("granularity")
    .and_then(|g| g.as_i64())
    .and_then(|g| data.get("timePoint")?.as_i64().map(|t| t + g)),
```

### What parser.rs Gets Right

- General response structure parsing (`extract_data`, error handling)
- Klines array structure and ordering (correct reversal)
- Klines time conversion (seconds to milliseconds)
- Order status logic using boolean flags
- Order field mapping (`id`/`orderId`, `size`, `dealSize`, etc.)
- Balance field mapping (`currency`, `available`, `holds`)
- Position field mapping and sign-based side determination
- Futures account balance parsing
- All numeric parsing with string/number flexibility

### Recommendations

1. **Fix bid/ask swap immediately** - this causes wrong prices
2. Add fallback for orderbook `ts` field
3. Consider adding predicted funding rate support
4. Add integration tests with real API response samples
5. Document KuCoin's confusing `buy`/`sell` naming in comments

---

## Sources

- [KuCoin API Documentation - Success Response](https://www.kucoin.com/docs/basic-info/connection-method/request/success-response)
- [KuCoin API Documentation - Get Ticker](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-ticker)
- [KuCoin API Documentation - Get Klines](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-klines)
- [KuCoin API Documentation - Get Full Order Book Level 2](https://www.kucoin.com/docs/rest/futures-trading/market-data/get-full-order-book-level-2)
- [KuCoin API Documentation - Get All Tickers](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-all-tickers)
- [KuCoin API Documentation - Place Order](https://www.kucoin.com/docs/rest/spot-trading/orders/place-order)
- [KuCoin API Documentation - Get Account List](https://www.kucoin.com/docs/rest/account/basic-info/get-account-list-spot-margin-trade_hf)
- [KuCoin API Documentation - Get Position Details](https://www.kucoin.com/docs/rest/futures-trading/positions/get-position-details)
- [KuCoin API Documentation - Get Current Funding Rate](https://www.kucoin.com/docs/rest/futures-trading/funding-fees/get-current-funding-rate)
- [KuCoin API Documentation - Timestamps](https://www.kucoin.com/docs/basic-info/connection-method/types/timestamps)
- [GitHub - Kucoin/kucoin-api-docs](https://github.com/Kucoin/kucoin-api-docs)
- [GitHub - kucoin-klines](https://github.com/gudlc/kucoin-klines)
