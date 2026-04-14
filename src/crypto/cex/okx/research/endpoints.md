# OKX API v5 Endpoints

## Base URLs

### REST API
- **Production**: `https://www.okx.com`
- **Demo Trading**: `https://www.okx.com` (with header `x-simulated-trading: 1`)

### WebSocket API
**Production:**
- Public: `wss://ws.okx.com:8443/ws/v5/public`
- Private: `wss://ws.okx.com:8443/ws/v5/private`
- Business: `wss://ws.okx.com:8443/ws/v5/business`

**Demo Trading:**
- Public: `wss://wspap.okx.com:8443/ws/v5/public`
- Private: `wss://wspap.okx.com:8443/ws/v5/private`
- Business: `wss://wspap.okx.com:8443/ws/v5/business`

---

## MarketData Trait Endpoints

### get_price
**Endpoint:** `GET /api/v5/market/ticker`

**Parameters:**
- `instId` (required): Instrument ID (e.g., `BTC-USDT`)

**Rate Limit:** 20 requests per 2 seconds (IP-based)

**Response Fields:**
- `instId`: Instrument ID
- `last`: Last traded price
- `lastSz`: Last traded size
- `askPx`: Best ask price
- `askSz`: Ask size
- `bidPx`: Best bid price
- `bidSz`: Bid size
- `open24h`: 24h opening price
- `high24h`: 24h highest price
- `low24h`: 24h lowest price
- `vol24h`: 24h trading volume (base currency for SPOT/spread, USD for crypto-margined)
- `volCcy24h`: 24h trading volume (quote currency)
- `sodUtc0`: Start of day price (UTC 0)
- `sodUtc8`: Start of day price (UTC 8)
- `ts`: Timestamp in milliseconds

### get_orderbook
**Endpoint:** `GET /api/v5/market/books`

**Parameters:**
- `instId` (required): Instrument ID
- `sz` (optional): Order book depth (default 1, max 400)

**Alternative Endpoint:** `GET /api/v5/market/books-full` (all levels)

**Rate Limit:** 20 requests per 2 seconds (IP-based)

**Response Structure:**
- `asks`: Array of [price, size, deprecated_field, amount]
- `bids`: Array of [price, size, deprecated_field, amount]
- `ts`: Timestamp in milliseconds

**Array Index Mapping:**
- Index 0: Price level
- Index 1: Quantity at price level
- Index 2: (Deprecated, ignore)
- Index 3: Amount

### get_klines
**Endpoint:** `GET /api/v5/market/candles`

**Parameters:**
- `instId` (required): Instrument ID
- `bar` (optional): Bar size (default: `1m`)
  - Hong Kong time: `1m`, `3m`, `5m`, `15m`, `30m`, `1H`, `2H`, `4H`, `6H`, `12H`, `1D`, `1W`, `1M`, `3M`, `6M`, `1Y`
  - UTC time: `6Hutc`, `12Hutc`, `1Dutc`, `1Wutc`, `1Mutc`, `3Mutc`, `6Mutc`, `1Yutc`
- `after` (optional): Pagination - request data after this timestamp
- `before` (optional): Pagination - request data before this timestamp
- `limit` (optional): Number of results (max 300, default 100)

**Historical Data:** `GET /api/v5/market/history-candles` (same parameters)

**Rate Limit:** 20 requests per 2 seconds (IP-based)

**Response Structure:**
Each candle is an array: `[timestamp, open, high, low, close, volume, volumeCcy, volumeCcyQuote, confirm]`
- Index 0: Timestamp (milliseconds)
- Index 1: Open price
- Index 2: High price
- Index 3: Low price
- Index 4: Close price
- Index 5: Volume (trading currency)
- Index 6: Volume (quote currency)
- Index 7: Volume (USD for contracts)
- Index 8: Confirm (0 = in progress, 1 = complete)

**Note:** Shorter timeframes (1m, 5m, 1H) use Hong Kong time. Use UTC-suffixed bars (e.g., `1Dutc`) for UTC alignment.

### get_ticker
**Endpoint:** `GET /api/v5/market/ticker` (same as get_price)

**Bulk Tickers:** `GET /api/v5/market/tickers`

**Parameters (bulk):**
- `instType` (required): `SPOT`, `MARGIN`, `SWAP`, `FUTURES`, `OPTION`
- `uly` (optional): Underlying (e.g., `BTC-USD`)
- `instFamily` (optional): Instrument family
- `instId` (optional): Specific instrument

### ping
**Endpoint:** `GET /api/v5/public/time`

**Rate Limit:** 20 requests per 2 seconds (IP-based)

**Response:**
```json
{
  "code": "0",
  "msg": "",
  "data": [{
    "ts": "1672841403093"
  }]
}
```

### Additional Market Data Endpoints

**Recent Trades:**
- `GET /api/v5/market/trades`
  - Parameters: `instId` (required), `limit` (optional, max 500)

**Historical Trades:**
- `GET /api/v5/market/history-trades`
  - Parameters: `instId` (required), `type`, `after`, `before`, `limit`

---

## Trading Trait Endpoints

### market_order
**Endpoint:** `POST /api/v5/trade/order`

**Parameters:**
- `instId` (required): Instrument ID
- `tdMode` (required): Trade mode (`cash`, `cross`, `isolated`)
- `side` (required): `buy` or `sell`
- `ordType` (required): `market`
- `sz` (required): Quantity
- `clOrdId` (optional): Client order ID (max 32 alphanumeric, case-sensitive)
- `tag` (optional): Order tag

**Rate Limit:** Instrument ID level, 1,000 requests per 2 seconds (sub-account level)

**Permission:** Trade

**Response Fields:**
- `ordId`: Exchange order ID
- `clOrdId`: Client order ID
- `sCode`: Status code ("0" = success)
- `sMsg`: Status message

### limit_order
**Endpoint:** `POST /api/v5/trade/order`

**Parameters:**
- `instId` (required): Instrument ID
- `tdMode` (required): Trade mode
- `side` (required): `buy` or `sell`
- `ordType` (required): `limit`
- `px` (required): Price
- `sz` (required): Quantity
- `clOrdId` (optional): Client order ID
- `tag` (optional): Order tag

**Rate Limit:** Same as market_order

### cancel_order
**Endpoint:** `POST /api/v5/trade/cancel-order`

**Parameters:**
- `instId` (required): Instrument ID
- `ordId` (conditional): Order ID from exchange (required if no `clOrdId`)
- `clOrdId` (conditional): Client order ID (required if no `ordId`)

**Rate Limit:** Independent from place/amend limits

**Permission:** Trade

### get_order
**Endpoint:** `GET /api/v5/trade/order`

**Parameters:**
- `instId` (required): Instrument ID
- `ordId` (conditional): Order ID
- `clOrdId` (conditional): Client order ID

**Rate Limit:** 20 requests per 2 seconds (User ID)

**Permission:** Read

**Response Fields:**
- `ordId`, `clOrdId`, `instId`, `instType`
- `px`: Price
- `sz`: Size
- `state`: Order state (`live`, `partially_filled`, `filled`, `canceled`)
- `avgPx`: Average filled price
- `accFillSz`: Accumulated fill quantity
- `fillPx`: Last fill price
- `fillSz`: Last fill quantity
- `cTime`: Creation time
- `uTime`: Update time

### get_open_orders
**Endpoint:** `GET /api/v5/trade/orders-pending`

**Parameters:**
- `instType` (optional): Filter by instrument type
- `instId` (optional): Filter by instrument ID
- `ordType` (optional): Filter by order type
- `state` (optional): Filter by state
- `after` (optional): Pagination
- `before` (optional): Pagination
- `limit` (optional): Max 100

**Rate Limit:** 20 requests per 2 seconds

**Permission:** Read

### Additional Trading Endpoints

**Place Multiple Orders:**
- `POST /api/v5/trade/batch-orders`

**Cancel Multiple Orders:**
- `POST /api/v5/trade/cancel-batch-orders`

**Amend Order:**
- `POST /api/v5/trade/amend-order`
  - Independent rate limit from place/cancel

**Amend Multiple Orders:**
- `POST /api/v5/trade/amend-batch-orders`

**Order History (7 days):**
- `GET /api/v5/trade/orders-history`

**Order History (3 months archive):**
- `GET /api/v5/trade/orders-history-archive`

---

## Account Trait Endpoints

### get_balance
**Endpoint:** `GET /api/v5/account/balance`

**Parameters:**
- `ccy` (optional): Single or multiple currencies (comma-separated)

**Rate Limit:** 10 requests per 2 seconds

**Permission:** Read

**Response Fields:**
- `totalEq`: Total equity in USD
- `isoEq`: Isolated margin equity in USD
- `adjEq`: Adjusted equity in USD
- `details`: Array of currency balances
  - `ccy`: Currency
  - `eq`: Equity
  - `cashBal`: Cash balance
  - `availBal`: Available balance
  - `frozenBal`: Frozen balance
  - `ordFrozen`: Margin frozen for open orders
  - `upl`: Unrealized P&L

**Alternative (Funding Account):**
- `GET /api/v5/asset/balances`

### get_account_info
**Endpoint:** `GET /api/v5/account/config`

**Rate Limit:** 20 requests per 2 seconds

**Permission:** Read

**Response Fields:**
- `uid`: User ID
- `acctLv`: Account level (`1` = Simple, `2` = Single-currency margin, `3` = Multi-currency margin, `4` = Portfolio margin)
- `posMode`: Position mode (`long_short_mode`, `net_mode`)
- `autoLoan`: Auto-borrow enabled
- `greeksType`: Greeks type
- `level`: VIP level
- `levelTmp`: Temporary VIP level

**Get Instruments:**
- `GET /api/v5/public/instruments`
  - Parameters: `instType` (required), `uly`, `instFamily`, `instId`

---

## Positions Trait Endpoints

### get_positions
**Endpoint:** `GET /api/v5/account/positions`

**Parameters:**
- `instType` (optional): Filter by instrument type
- `instId` (optional): Filter by instrument ID
- `posId` (optional): Filter by position ID

**Rate Limit:** 10 requests per 2 seconds

**Permission:** Read

**Response Fields:**
- `instId`, `instType`, `mgnMode`
- `posId`: Position ID
- `posSide`: Position side (`long`, `short`, `net`)
- `pos`: Quantity of positions
- `availPos`: Available position to close
- `avgPx`: Average open price
- `upl`: Unrealized P&L
- `uplRatio`: Unrealized P&L ratio
- `lever`: Leverage
- `liqPx`: Estimated liquidation price
- `markPx`: Latest mark price
- `margin`: Margin
- `mgnRatio`: Margin ratio
- `cTime`: Creation time
- `uTime`: Update time

### get_funding_rate
**Endpoint:** `GET /api/v5/public/funding-rate`

**Parameters:**
- `instId` (required): Instrument ID (must be SWAP)

**Rate Limit:** 20 requests per 2 seconds (IP-based)

**Permission:** Public (no auth required)

**Response Fields:**
- `instId`: Instrument ID
- `instType`: `SWAP`
- `fundingRate`: Current funding rate
- `nextFundingRate`: Next period funding rate
- `fundingTime`: Funding time (milliseconds)
- `nextFundingTime`: Next funding time

**Funding Rate History:**
- `GET /api/v5/public/funding-rate-history`
  - Parameters: `instId` (required), `after`, `before`, `limit` (max 100)

### set_leverage
**Endpoint:** `POST /api/v5/account/set-leverage`

**Parameters:**
- `instId` (optional): Instrument ID
- `ccy` (optional): Currency (for MARGIN)
- `lever` (required): Leverage (e.g., `5`)
- `mgnMode` (required): Margin mode (`cross`, `isolated`)
- `posSide` (optional): Position side (for long/short mode)

**Rate Limit:** 20 requests per 2 seconds

**Permission:** Trade

**Response Fields:**
- `instId`, `lever`, `mgnMode`, `posSide`

**Get Leverage:**
- `GET /api/v5/account/leverage-info`
  - Parameters: `instId` (required), `mgnMode` (required)

### Additional Position Endpoints

**Get Maximum Order Quantity:**
- `GET /api/v5/account/max-size`
  - Parameters: `instId` (required), `tdMode` (required), `ccy`, `px`

**Increase/Decrease Margin:**
- `POST /api/v5/account/position/margin-balance`
  - Parameters: `instId`, `posSide`, `type` (`add` or `reduce`), `amt`

**Set Position Mode:**
- `POST /api/v5/account/set-position-mode`
  - Parameters: `posMode` (`long_short_mode` or `net_mode`)

---

## WebSocket Support

All trading operations (place, cancel, amend) can also be performed via WebSocket on the private channel with the same rate limits as REST endpoints.

**Rate limits for placing, amending, and canceling orders are independent from each other.**

---

## Notes

1. **Unified API:** V5 endpoints are unified across all instrument types (SPOT, SWAP, FUTURES, MARGIN, OPTION)
2. **Error Code 50011:** Rate limit exceeded
3. **Error Code 50061:** Order rate limit exceeded (1,000 per 2 seconds at sub-account level)
4. **Timestamp Expiration:** Requests expire 30 seconds after the timestamp
5. **Demo Trading:** Use header `x-simulated-trading: 1` with production URLs
