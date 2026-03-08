# Phemex API Endpoints

Complete endpoint reference for V5 connector implementation covering all required traits.

## Base URLs

| Environment | REST API | WebSocket |
|-------------|----------|-----------|
| Production | `https://api.phemex.com` | `wss://ws.phemex.com/ws` |
| High Rate Limit (VIP) | `https://vapi.phemex.com` | `wss://vapi.phemex.com/ws` |
| Testnet | `https://testnet-api.phemex.com` | `wss://testnet.phemex.com/ws` |

## MarketData Trait Endpoints

### Get Server Time
- **Endpoint:** `GET /public/time`
- **Auth:** Not required
- **Response:** `{"code": 0, "msg": "OK", "data": {"timestamp": 1234567890}}`

### Get Product Information
- **Endpoint:** `GET /public/products`
- **Auth:** Not required
- **Description:** Returns all trading symbols with metadata
- **Response Fields:**
  - `symbol` - Symbol identifier
  - `priceScale` - Price scaling factor (typically 4 or 8)
  - `ratioScale` - Ratio scaling factor (typically 8)
  - `valueScale` - Value scaling factor (typically 4 or 8)
  - `type` - Product type (PerpetualV2, Spot, etc.)
  - `quoteCurrency` - Quote currency
  - `settleCurrency` - Settlement currency
  - `contractSize` - Contract size (for futures)
  - `lotSize` - Minimum order size
  - `tickSize` - Minimum price increment
  - `maxPriceEp` - Maximum price (scaled)
  - `minPriceEp` - Minimum price (scaled)
  - `maxOrderQty` - Maximum order quantity

### Get Order Book
**Spot:**
- **Endpoint:** `GET /md/orderbook?symbol={symbol}`
- **Auth:** Not required
- **Parameters:**
  - `symbol` (required) - e.g., `sBTCUSDT`
- **Response:**
  ```json
  {
    "code": 0,
    "msg": "OK",
    "data": {
      "asks": [[priceEp, size], ...],
      "bids": [[priceEp, size], ...],
      "sequence": 123456,
      "timestamp": 1234567890
    }
  }
  ```

**Contract:**
- **Endpoint:** `GET /md/orderbook?symbol={symbol}`
- **Depth:** Returns top 30 levels

**Full Order Book (Contract):**
- **Endpoint:** `GET /md/fullbook?symbol={symbol}`
- **Depth:** Returns complete order book

### Get Recent Trades
- **Endpoint:** `GET /md/trade?symbol={symbol}`
- **Auth:** Not required
- **Response:**
  ```json
  {
    "code": 0,
    "data": {
      "trades": [
        {
          "timestamp": 1234567890,
          "side": "Buy",
          "priceEp": 12345000,
          "size": 100
        }
      ]
    }
  }
  ```

### Get 24-Hour Ticker
- **Endpoint:** `GET /md/ticker/24hr?symbol={symbol}`
- **Auth:** Not required
- **Parameters:** `symbol` - e.g., `BTCUSD` or `sBTCUSDT`
- **Response Fields:**
  - `open` - 24h opening price
  - `high` - 24h highest price
  - `low` - 24h lowest price
  - `close` - Current/closing price
  - `volume` - 24h volume
  - `turnover` - 24h turnover value

### Get Klines/Candlesticks
- **Endpoint:** `GET /exchange/public/md/v2/kline`
- **Auth:** Not required
- **Parameters:**
  - `symbol` (required) - e.g., `BTCUSD`, `sBTCUSDT`
  - `resolution` (required) - Time interval in seconds:
    - `60` - 1 minute
    - `300` - 5 minutes
    - `900` - 15 minutes
    - `1800` - 30 minutes
    - `3600` - 1 hour
    - `14400` - 4 hours
    - `86400` - 1 day
    - `604800` - 1 week
    - `2592000` - 1 month
    - `7776000` - 1 season
    - `31104000` - 1 year
  - `limit` - Number of klines (5, 10, 50, 100, 500, 1000)
  - `from` - Start timestamp
  - `to` - End timestamp

### Get Funding Rate History (Futures)
- **Endpoint:** `GET /api-data/public/data/funding-rate-history`
- **Auth:** Not required
- **Parameters:**
  - `symbol` (required)
  - `limit` - Number of records

## Trading Trait Endpoints

### Place Order

**Spot:**
- **Endpoint:** `POST /spot/orders` or `PUT /spot/orders`
- **Auth:** Required (HMAC SHA256)
- **Parameters:**
  - `symbol` (required) - e.g., `sBTCUSDT`
  - `clOrdID` (required) - Client order ID
  - `side` (required) - `Buy` or `Sell`
  - `qtyType` (required) - `ByBase` or `ByQuote`
  - `baseQtyEv` / `quoteQtyEv` - Quantity (scaled)
  - `priceEp` - Price (scaled), required for limit orders
  - `ordType` (required) - Order type (see Order Types section)
  - `timeInForce` - `GoodTillCancel`, `PostOnly`, `ImmediateOrCancel`, `FillOrKill`
  - `stopPxEp` - Stop price (scaled) for conditional orders
  - `triggerType` - `ByLastPrice` or `ByMarkPrice`

**Contract:**
- **Endpoint:** `POST /orders` or `PUT /orders`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required) - e.g., `BTCUSD`
  - `clOrdID` (optional) - Client order ID
  - `side` (required) - `Buy` or `Sell`
  - `orderQty` (required) - Order quantity
  - `priceEp` (conditional) - Price (scaled)
  - `ordType` (required) - Order type
  - `timeInForce` - Time in force
  - `reduceOnly` - Boolean, reduce position only
  - `closeOnTrigger` - Boolean, close position on trigger
  - `takeProfitEp` - Take profit price (scaled)
  - `stopLossEp` - Stop loss price (scaled)
  - `triggerType` - Price trigger type

**Hedged Contract:**
- **Endpoint:** `POST /g-orders/create` or `POST /g-orders`
- **Additional Parameter:**
  - `posSide` (required) - `Long`, `Short`, or `Merged`

### Amend/Replace Order

**Spot:**
- **Endpoint:** `PUT /spot/orders`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `orderID` or `clOrdID` (required)
  - Modifiable fields: `priceEp`, `baseQtyEv`, `quoteQtyEv`, `stopPxEp`

**Contract:**
- **Endpoint:** `PUT /orders/replace`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `orderID` (required)
  - Modifiable fields: `priceEp`, `orderQty`, `stopPxEp`, `takeProfitEp`, `stopLossEp`

**Hedged Contract:**
- **Endpoint:** `PUT /g-orders/replace`

### Cancel Order

**Spot:**
- **Endpoint:** `DELETE /spot/orders?symbol={symbol}&orderID={orderID}`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `orderID` or `clOrdID` (required, pick one)

**Contract:**
- **Endpoint:** `DELETE /orders?symbol={symbol}&orderID={orderID}`
- **Auth:** Required
- **Alternative:** `DELETE /orders?symbol={symbol}&clOrdID={clOrdID}`

**Hedged Contract:**
- **Endpoint:** `DELETE /g-orders/cancel`

### Bulk Cancel Orders

**Spot:**
- **Endpoint:** `DELETE /spot/orders/all?symbol={symbol}`
- **Auth:** Required
- **Parameters:** `symbol` (required)

**Contract:**
- **Endpoint:** `DELETE /orders/all?symbol={symbol}`
- **Auth:** Required
- **Alternative:** `DELETE /orders?symbol={symbol}` (cancel multiple specific orders)

**Hedged Contract:**
- **Endpoint:** `DELETE /g-orders`

### Query Open Orders

**Spot:**
- **Endpoint:** `GET /spot/orders/active?symbol={symbol}`
- **Auth:** Required
- **Parameters:**
  - `symbol` (optional) - Filter by symbol
  - `orderID` (optional) - Query specific order

**Contract:**
- **Endpoint:** `GET /orders/activeList?symbol={symbol}`
- **Auth:** Required
- **Parameters:**
  - `symbol` (optional)
  - `ordStatus` (optional) - Filter by status: `New`, `PartiallyFilled`, `Untriggered`

### Query Closed Orders

**Contract:**
- **Endpoint:** `GET /exchange/order/list`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `start` - Start timestamp
  - `end` - End timestamp
  - `offset` - Pagination offset
  - `limit` - Number of records

### Query Order by ID

**Contract:**
- **Endpoint:** `GET /exchange/order?orderID={orderID}`
- **Alternative:** `GET /exchange/order?clOrdID={clOrdID}`
- **Auth:** Required

### Query Trade History

**Spot:**
- **Endpoint:** (Query through order endpoints)

**Contract:**
- **Endpoint:** `GET /exchange/order/trade?symbol={symbol}`
- **Auth:** Required
- **Parameters:**
  - `symbol` (optional)
  - `start` - Start timestamp
  - `end` - End timestamp
  - `limit` - Number of records

## Account Trait Endpoints

### Query Account Balance

**Spot:**
- **Endpoint:** `GET /spot/wallets`
- **Auth:** Required
- **Parameters:**
  - `currency` (optional) - Filter by currency (e.g., `BTC`, `USDT`)
- **Response:**
  ```json
  {
    "code": 0,
    "data": {
      "balances": [
        {
          "currency": "BTC",
          "balanceEv": 100000000,
          "lockedTradingBalanceEv": 0,
          "lockedWithdrawEv": 0
        }
      ]
    }
  }
  ```

**Contract:**
- **Endpoint:** `GET /accounts/accountPositions?currency={currency}`
- **Auth:** Required
- **Parameters:** `currency` (required) - e.g., `BTC`, `USD`
- **Response:** Includes account balance and positions

**Enhanced (with real-time PnL):**
- **Endpoint:** `GET /accounts/positions`
- **Auth:** Required
- **Note:** Higher rate limit weight (25)

### Transfer Between Accounts

- **Endpoint:** `POST /assets/transfer`
- **Auth:** Required
- **Parameters:**
  - `currency` (required) - e.g., `BTC`, `USDT`
  - `amountEv` (required) - Amount (scaled)
  - `moveOp` (required) - Direction:
    - `1` - Futures to Spot
    - `2` - Spot to Futures
- **Response:**
  ```json
  {
    "code": 0,
    "data": {
      "linkKey": "transfer-id",
      "userId": 123456,
      "currency": "BTC",
      "amountEv": 100000000,
      "side": 2,
      "status": 10
    }
  }
  ```

### Query Transfer History

- **Endpoint:** `GET /assets/transfer`
- **Auth:** Required
- **Parameters:**
  - `currency` (optional)
  - `start` - Start timestamp
  - `end` - End timestamp
  - `offset` - Pagination offset
  - `limit` - Number of records

## Positions Trait Endpoints

### Query Positions

**Contract:**
- **Endpoint:** `GET /accounts/accountPositions?currency={currency}`
- **Auth:** Required
- **Response Fields:**
  - `symbol` - Contract symbol
  - `side` - Position side (`Buy`/`Sell`)
  - `size` - Position size
  - `avgEntryPriceEp` - Average entry price (scaled)
  - `posCostEv` - Position cost (scaled)
  - `assignedPosBalanceEv` - Assigned balance (isolated mode)
  - `unrealisedPnlEv` - Unrealized PnL (scaled)
  - `realisedPnlEv` - Realized PnL (scaled)
  - `cumClosedPnlEv` - Cumulative closed PnL
  - `leverageEr` - Leverage (scaled, sign indicates margin mode)
  - `riskLimitEv` - Risk limit (scaled)
  - `liquidationPriceEp` - Liquidation price (scaled)

**Hedged Mode:**
- Same endpoint returns separate `Long` and `Short` positions for each symbol

### Set Leverage

- **Endpoint:** `PUT /positions/leverage`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `leverageEr` (required) - Leverage (scaled)
    - Positive value = Isolated margin mode
    - Zero or negative = Cross margin mode
  - `posSide` (hedged mode only) - `Long` or `Short`

### Set Risk Limit

- **Endpoint:** `PUT /positions/riskLimit`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `riskLimitEv` (required) - Risk limit value (scaled)

### Assign Position Balance (Isolated Mode)

- **Endpoint:** `POST /positions/assign`
- **Auth:** Required
- **Parameters:**
  - `symbol` (required)
  - `posBalanceEv` (required) - Balance to assign (scaled)
  - `posSide` (hedged mode only) - `Long` or `Short`

## Order Types

All markets support these order types:

| Order Type | Description | Trigger Direction |
|------------|-------------|-------------------|
| `Limit` | Standard limit order | Immediate |
| `Market` | Execute at best available price | Immediate |
| `Stop` | Stop-loss market order | Against position |
| `StopLimit` | Stop-loss limit order | Against position |
| `MarketIfTouched` | Take-profit market order | With position |
| `LimitIfTouched` | Take-profit limit order | With position |
| `MarketAsLimit` | Market order with limit price | Immediate |
| `StopAsLimit` | Stop market executed as limit | Against position |
| `MarketIfTouchedAsLimit` | Take-profit market as limit | With position |

## TimeInForce Options

| Value | Description |
|-------|-------------|
| `GoodTillCancel` | Remains active until filled or canceled |
| `PostOnly` | Only adds liquidity (maker orders) |
| `ImmediateOrCancel` | Fill immediately, cancel remainder |
| `FillOrKill` | Fill completely or cancel entirely |

## Order Status Values

| Status | Description |
|--------|-------------|
| `New` | Order placed and active |
| `PartiallyFilled` | Order partially executed |
| `Filled` | Order completely executed |
| `Canceled` | Order canceled |
| `Rejected` | Order rejected by system |
| `Triggered` | Conditional order triggered |
| `Untriggered` | Conditional order waiting for trigger |

## Response Format

All REST API endpoints (except `/md/*`) return standardized JSON:

```json
{
  "code": 0,           // 0 = success, non-zero = error
  "msg": "OK",         // Status message
  "data": { ... }      // Response data (varies by endpoint)
}
```

## Notes

1. All spot symbols are prefixed with `s` (e.g., `sBTCUSDT`)
2. Contract symbols have no prefix (e.g., `BTCUSD`, `ETHUSD`)
3. Fields with suffix `Ep` are scaled prices
4. Fields with suffix `Er` are scaled ratios
5. Fields with suffix `Ev` are scaled values
6. Scaling factors are defined per symbol in `/public/products`
7. Hedged mode requires `posSide` parameter for position-specific operations
