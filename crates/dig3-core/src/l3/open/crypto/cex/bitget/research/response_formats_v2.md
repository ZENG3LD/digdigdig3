# Bitget API V2 Response Formats

This document details the JSON response formats for Bitget V2 API endpoints.

## Standard Response Structure

All V2 API responses follow a consistent structure:

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695865615662,
  "data": { ... }
}
```

### Response Fields

- `code` (string): Status code
  - `"00000"` - Success
  - Other codes indicate errors (see error codes section)
- `msg` (string): Response message, typically `"success"` for successful requests
- `requestTime` (number): Unix timestamp in milliseconds when request was processed
- `data` (object | array): Main response payload, varies by endpoint

## Market Data Responses

### Ticker (Single or Multiple)

**Endpoint**: `GET /api/v2/spot/market/tickers`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": [
    {
      "symbol": "BTCUSDT",
      "high24h": "37775.65",
      "open": "35134.2",
      "low24h": "34413.1",
      "lastPr": "34413.1",
      "quoteVolume": "1234567.89",
      "baseVolume": "35.1234",
      "usdtVolume": "1234567.89",
      "bidPr": "34410.0",
      "askPr": "34415.0",
      "bidSz": "0.0663",
      "askSz": "0.0119",
      "openUtc": "23856.72",
      "ts": "1625125755277",
      "changeUtc24h": "0.00301",
      "change24h": "0.00069"
    }
  ]
}
```

**Data Fields**:
- `symbol` (string): Trading pair symbol (e.g., "BTCUSDT")
- `high24h` (string): 24-hour high price
- `low24h` (string): 24-hour low price
- `open` (string): Opening price (24h ago)
- `lastPr` (string): Last/latest traded price
- `bidPr` (string): Best bid price
- `askPr` (string): Best ask price
- `bidSz` (string): Best bid size/quantity
- `askSz` (string): Best ask size/quantity
- `baseVolume` (string): 24h trading volume in base currency
- `quoteVolume` (string): 24h trading volume in quote currency
- `usdtVolume` (string): 24h volume in USDT equivalent
- `openUtc` (string): Opening price at UTC 00:00
- `ts` (string): Timestamp of ticker data (milliseconds)
- `change24h` (string): 24h price change percentage (e.g., "0.00069" = 0.069%)
- `changeUtc24h` (string): Price change since UTC 00:00

**Note**: Single ticker query returns array with one element. All numeric values are strings.

### Orderbook

**Endpoint**: `GET /api/v2/spot/market/orderbook`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1698303884579,
  "data": {
    "asks": [
      ["34567.15", "0.0131"],
      ["34567.25", "0.0144"],
      ["34567.50", "0.2500"]
    ],
    "bids": [
      ["34567.00", "0.2917"],
      ["34566.85", "0.0145"],
      ["34566.50", "0.5000"]
    ],
    "ts": "1698303884584"
  }
}
```

**Data Fields**:
- `asks` (array): Sell orders, sorted by price ascending (lowest ask first)
  - Each element: `[price, quantity]` (both strings)
- `bids` (array): Buy orders, sorted by price descending (highest bid first)
  - Each element: `[price, quantity]` (both strings)
- `ts` (string): Timestamp from matching engine (milliseconds)

**Parameters**:
- `symbol` (required): Trading pair (e.g., "BTCUSDT")
- `type` (optional): Depth level - "step0" to "step5" (step0 = full depth)
- `limit` (optional): Number of levels per side (default varies)

### Candlestick/Klines (OHLCV)

**Endpoint**: `GET /api/v2/spot/market/candles`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695800278693,
  "data": [
    [
      "1656604800000",
      "37834.5",
      "37849.5",
      "37773.5",
      "37773.5",
      "428.3462",
      "16198849.1079",
      "16198849.1079"
    ],
    [
      "1656601200000",
      "37800.0",
      "37850.0",
      "37750.0",
      "37834.5",
      "521.1234",
      "19712345.6789",
      "19712345.6789"
    ]
  ]
}
```

**Data Array Structure** (each candle is an array):
- Index 0: Timestamp (string, milliseconds)
- Index 1: Open price (string)
- Index 2: High price (string)
- Index 3: Low price (string)
- Index 4: Close price (string)
- Index 5: Base volume (string) - volume in base currency
- Index 6: Quote volume (string) - volume in quote currency
- Index 7: USDT volume (string) - volume in USDT equivalent

**Parameters**:
- `symbol` (required): Trading pair
- `granularity` or `period` (required): Time interval
  - Spot: "1min", "5min", "15min", "30min", "1h", "4h", "12h", "1day", "1week"
  - Futures: "1m", "5m", "15m", "30m", "1H", "4H", "12H", "1D", "1W"
- `limit` (optional): Max 1000, default 100
- `startTime` / `endTime` (optional): Unix timestamp in milliseconds

**Note**: Returns up to 1000 candles. Empty array if no data.

### Recent Fills/Trades

**Endpoint**: `GET /api/v2/spot/market/fills`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": [
    {
      "symbol": "BTCUSDT",
      "tradeId": "1001001001",
      "price": "34567.15",
      "size": "0.1234",
      "side": "buy",
      "ts": "1698303884584"
    }
  ]
}
```

**Data Fields**:
- `symbol` (string): Trading pair
- `tradeId` (string): Unique trade ID
- `price` (string): Trade price
- `size` (string): Trade quantity
- `side` (string): Trade direction - "buy" or "sell"
- `ts` (string): Trade timestamp (milliseconds)

### Symbols/Trading Pairs

**Endpoint**: `GET /api/v2/spot/public/symbols`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": [
    {
      "symbol": "BTCUSDT",
      "baseCoin": "BTC",
      "quoteCoin": "USDT",
      "minTradeAmount": "0.0001",
      "maxTradeAmount": "10000",
      "takerFeeRate": "0.002",
      "makerFeeRate": "0.002",
      "pricePrecision": "2",
      "quantityPrecision": "4",
      "quotePrecision": "8",
      "status": "online",
      "minTradeUSDT": "5",
      "buyLimitPriceRatio": "0.05",
      "sellLimitPriceRatio": "0.05"
    }
  ]
}
```

**Data Fields**:
- `symbol` (string): Trading pair symbol
- `baseCoin` (string): Base currency (e.g., "BTC")
- `quoteCoin` (string): Quote currency (e.g., "USDT")
- `minTradeAmount` (string): Minimum trade quantity
- `maxTradeAmount` (string): Maximum trade quantity
- `takerFeeRate` (string): Taker fee rate (e.g., "0.002" = 0.2%)
- `makerFeeRate` (string): Maker fee rate
- `pricePrecision` (string): Decimal places for price
- `quantityPrecision` (string): Decimal places for quantity
- `quotePrecision` (string): Quote currency precision
- `status` (string): Trading status - "online", "offline", etc.
- `minTradeUSDT` (string): Minimum order value in USDT
- `buyLimitPriceRatio` (string): Max price deviation for buy orders
- `sellLimitPriceRatio` (string): Max price deviation for sell orders

## Trading Responses

### Place Order

**Endpoint**: `POST /api/v2/spot/trade/place-order`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": {
    "orderId": "1098394857234",
    "clientOid": "custom_order_123"
  }
}
```

**Data Fields**:
- `orderId` (string): Exchange-generated order ID
- `clientOid` (string): Client-provided order ID (echoed back)

**Request Body Example**:
```json
{
  "symbol": "BTCUSDT",
  "side": "buy",
  "orderType": "limit",
  "force": "gtc",
  "price": "34000.00",
  "size": "0.01",
  "clientOid": "custom_order_123"
}
```

### Cancel Order

**Endpoint**: `POST /api/v2/spot/trade/cancel-order`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": {
    "orderId": "1098394857234",
    "clientOid": "custom_order_123"
  }
}
```

### Order Info

**Endpoint**: `GET /api/v2/spot/trade/orderInfo`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": [
    {
      "userId": "123456",
      "symbol": "BTCUSDT",
      "orderId": "1098394857234",
      "clientOid": "custom_order_123",
      "price": "34000.00",
      "size": "0.01",
      "orderType": "limit",
      "side": "buy",
      "status": "filled",
      "priceAvg": "34005.00",
      "baseVolume": "0.01",
      "quoteVolume": "340.05",
      "enterPointSource": "API",
      "feeDetail": {
        "deduction": "no",
        "feeCoin": "USDT",
        "totalDeductionFee": "0",
        "totalFee": "0.68"
      },
      "orderSource": "normal",
      "cTime": "1695808949356",
      "uTime": "1695809049456"
    }
  ]
}
```

**Data Fields**:
- `userId` (string): User ID
- `symbol` (string): Trading pair
- `orderId` (string): Order ID
- `clientOid` (string): Client order ID
- `price` (string): Order price
- `size` (string): Order quantity
- `orderType` (string): Order type - "limit", "market"
- `side` (string): Order side - "buy", "sell"
- `status` (string): Order status
  - "new" - Order accepted
  - "partial_fill" - Partially filled
  - "filled" - Completely filled
  - "cancelled" - Cancelled
- `priceAvg` (string): Average fill price
- `baseVolume` (string): Filled quantity (base currency)
- `quoteVolume` (string): Filled value (quote currency)
- `enterPointSource` (string): Entry source - "API", "WEB", "APP"
- `feeDetail` (object): Fee details
  - `deduction` (string): Fee deduction status
  - `feeCoin` (string): Fee currency
  - `totalDeductionFee` (string): Total deduction fee
  - `totalFee` (string): Total fee charged
- `orderSource` (string): Order source - "normal", "plan"
- `cTime` (string): Create time (milliseconds)
- `uTime` (string): Update time (milliseconds)

### Unfilled/Open Orders

**Endpoint**: `GET /api/v2/spot/trade/unfilled-orders`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": {
    "orderList": [
      {
        "userId": "123456",
        "symbol": "BTCUSDT",
        "orderId": "1098394857234",
        "clientOid": "custom_order_123",
        "price": "34000.00",
        "size": "0.01",
        "orderType": "limit",
        "side": "buy",
        "status": "new",
        "priceAvg": "0",
        "baseVolume": "0",
        "quoteVolume": "0",
        "cTime": "1695808949356",
        "uTime": "1695808949356"
      }
    ],
    "maxId": "1098394857234",
    "minId": "1098394857200"
  }
}
```

**Data Fields**:
- `orderList` (array): List of open orders (same structure as orderInfo)
- `maxId` (string): Maximum order ID in result (for pagination)
- `minId` (string): Minimum order ID in result (for pagination)

## Account Responses

### Account Assets/Balance

**Endpoint**: `GET /api/v2/spot/account/assets`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": [
    {
      "coin": "USDT",
      "available": "10000.50",
      "frozen": "500.25",
      "locked": "0",
      "limitAvailable": "10000.50",
      "uTime": "1622697148000"
    },
    {
      "coin": "BTC",
      "available": "0.5",
      "frozen": "0",
      "locked": "0",
      "limitAvailable": "0.5",
      "uTime": "1622697148000"
    }
  ]
}
```

**Data Fields**:
- `coin` (string): Asset/currency symbol (e.g., "USDT", "BTC")
- `available` (string): Available balance for trading
- `frozen` (string): Frozen balance (in orders)
- `locked` (string): Locked balance (other reasons)
- `limitAvailable` (string): Available for withdrawal
- `uTime` (string): Last update time (milliseconds)

**Parameters**:
- `coin` (optional): Filter by specific coin

### Account Info

**Endpoint**: `GET /api/v2/spot/account/info`

**Response**:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": {
    "userId": "123456",
    "inviterId": "0",
    "ips": "",
    "authorities": ["SPOT", "FUTURES"],
    "parentId": "0",
    "traderType": "normal",
    "level": "lv1",
    "kycStatus": "verified"
  }
}
```

## Futures Market Responses

Futures responses follow similar structure but with product-specific fields.

### Futures Ticker

**Endpoint**: `GET /api/v2/mix/market/ticker`

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": {
    "symbol": "BTCUSDT",
    "lastPr": "34567.5",
    "askPr": "34568.0",
    "bidPr": "34567.0",
    "bidSz": "1.234",
    "askSz": "2.345",
    "high24h": "35000.0",
    "low24h": "34000.0",
    "ts": "1695808949356",
    "change24h": "0.0123",
    "baseVolume": "12345.67",
    "quoteVolume": "426789012.34",
    "usdtVolume": "426789012.34",
    "openUtc": "34500.0",
    "changeUtc24h": "0.0098",
    "indexPrice": "34567.8",
    "fundingRate": "0.0001",
    "holdingAmount": "98765.43",
    "deliveryPrice": "0"
  }
}
```

**Additional Futures Fields**:
- `indexPrice` (string): Index price
- `fundingRate` (string): Current funding rate
- `holdingAmount` (string): Total open interest
- `deliveryPrice` (string): Delivery price (for delivery contracts)

### Futures Position

**Endpoint**: `GET /api/v2/mix/position/all-position`

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": [
    {
      "marginCoin": "USDT",
      "symbol": "BTCUSDT",
      "holdSide": "long",
      "openDelegateSize": "0",
      "marginSize": "1000.0",
      "available": "0.5",
      "locked": "0",
      "total": "0.5",
      "leverage": "10",
      "achievedProfits": "50.25",
      "unrealizedPL": "25.50",
      "liquidationPrice": "30000.0",
      "keepMarginRate": "0.004",
      "markPrice": "34567.5",
      "breakEvenPrice": "34000.0",
      "totalFee": "2.5",
      "deductedFee": "0",
      "marginMode": "crossed",
      "positionMode": "hedge_mode",
      "cTime": "1695808949356",
      "uTime": "1695809049456"
    }
  ]
}
```

**Position Fields**:
- `marginCoin` (string): Margin currency
- `symbol` (string): Contract symbol
- `holdSide` (string): Position side - "long", "short"
- `openDelegateSize` (string): Size in open orders
- `marginSize` (string): Margin amount
- `available` (string): Available position size
- `locked` (string): Locked position size
- `total` (string): Total position size
- `leverage` (string): Leverage
- `achievedProfits` (string): Realized P&L
- `unrealizedPL` (string): Unrealized P&L
- `liquidationPrice` (string): Liquidation price
- `markPrice` (string): Mark price
- `breakEvenPrice` (string): Break-even price
- `marginMode` (string): Margin mode - "crossed", "isolated"
- `positionMode` (string): Position mode - "one_way_mode", "hedge_mode"

## Error Response Format

When an error occurs, the response follows the same structure but with error code and message:

```json
{
  "code": "40001",
  "msg": "Invalid parameter",
  "requestTime": 1695808949356,
  "data": null
}
```

### Common Error Codes

- `00000` - Success
- `40001` - Invalid parameter
- `40002` - Missing parameter
- `40003` - Parameter format error
- `40004` - Invalid API key
- `40005` - Invalid signature
- `40006` - Invalid timestamp
- `40007` - Invalid IP
- `40010` - Insufficient balance
- `40011` - Order not found
- `40012` - Order already cancelled
- `43011` - Symbol not found
- `43025` - Order price/quantity precision error
- `50000` - Internal server error
- `50001` - Service unavailable

## Key Differences from V1

1. **Numeric Values**: All numeric values are strings in V2 (same as V1)

2. **Response Wrapper**: Consistent `code`, `msg`, `requestTime`, `data` structure (similar to V1)

3. **Field Names**: More consistent naming
   - V1: Some endpoints used camelCase, others snake_case
   - V2: Consistent camelCase (e.g., `lastPr`, `baseVolume`)

4. **Pagination**:
   - V1: `pageSize`, `pageNo`
   - V2: `limit`, `idLessThan` (cursor-based)

5. **Timestamps**: All in milliseconds (consistent with V1)

6. **Symbol Format in Response**:
   - V1 responses: `"BTCUSDT_SPBL"`
   - V2 responses: `"BTCUSDT"` (no suffix)

## Sources

- [Get Ticker Information | Bitget API](https://www.bitget.com/api-doc/spot/market/Get-Tickers)
- [Get OrderBook Depth | Bitget API](https://www.bitget.com/api-doc/spot/market/Get-Orderbook)
- [Get Candlestick Data | Bitget API](https://www.bitget.com/api-doc/spot/market/Get-Candle-Data)
- [Get Account Assets | Bitget API](https://www.bitget.com/api-doc/spot/account/Get-Account-Assets)
- [Place Order | Bitget API](https://www.bitget.com/api-doc/spot/trade/Place-Order)
- [Bitget API Changelog](https://www.bitget.com/api-doc/common/changelog)
