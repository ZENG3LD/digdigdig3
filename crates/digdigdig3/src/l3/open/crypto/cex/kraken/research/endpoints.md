# Kraken API Endpoints

## Base URLs

### Spot Trading (REST API)
- **Production**: `https://api.kraken.com`
- **Current API Version**: `/0/` (indicated in URL path)
- **Public Endpoints**: `/0/public/`
- **Private Endpoints**: `/0/private/`

### Futures Trading (REST API)
- **Production**: `https://futures.kraken.com`
- **Demo/Testing**: `https://demo-futures.kraken.com`
- **API Version**: `/derivatives/api/v3/`

### WebSocket APIs
- **Spot WebSocket v2**: `wss://ws.kraken.com/v2`
- **Spot WebSocket v1**: `wss://ws.kraken.com` (older version)
- **Futures WebSocket**: `wss://futures.kraken.com/ws/v1`
- **Futures WebSocket (Demo)**: `wss://demo-futures.kraken.com/ws/v1`

---

## MarketData Trait Endpoints

### 1. get_price (Spot)
**Endpoint**: `GET /0/public/Ticker`

**Parameters**:
- `pair` (optional): Asset pair(s) to get data for. Leave blank for all tradeable assets.

**Example**: `https://api.kraken.com/0/public/Ticker?pair=XBTUSD`

**Response**: Returns ticker information including:
- `a`: Ask [price, whole lot volume, lot volume]
- `b`: Bid [price, whole lot volume, lot volume]
- `c`: Last trade closed [price, lot volume]
- `v`: Volume [today, last 24 hours]
- `p`: Volume weighted average price [today, last 24 hours]
- `t`: Number of trades [today, last 24 hours]
- `l`: Low [today, last 24 hours]
- `h`: High [today, last 24 hours]
- `o`: Today's opening price

**Note**: Request with `XBTUSD` returns data under key `XXBTZUSD` (see symbols.md)

---

### 2. get_orderbook (Spot)
**Endpoint**: `GET /0/public/Depth`

**Parameters**:
- `pair` (required): Asset pair to get market depth for
- `count` (optional): Maximum number of asks/bids (default varies)

**Description**: Returns level 2 (L2) order book with individual price levels and aggregated quantities.

**Response Format**:
```json
{
  "error": [],
  "result": {
    "XXBTZUSD": {
      "asks": [
        ["34920.50000", "5.123", 1234567890],
        ["34921.00000", "2.456", 1234567891]
      ],
      "bids": [
        ["34919.00000", "3.789", 1234567890],
        ["34918.50000", "1.234", 1234567891]
      ]
    }
  }
}
```

Each entry: `[price, volume, timestamp]`

---

### 3. get_klines / OHLC (Spot)
**Endpoint**: `GET /0/public/OHLC`

**Parameters**:
- `pair` (required): Asset pair to get OHLC data for
- `interval` (optional): Time frame interval in minutes
  - Allowed values: `1, 5, 15, 30, 60, 240, 1440, 10080, 21600`
  - Default: `1`
- `since` (optional): Return committed OHLC data since given timestamp

**Limitations**:
- Returns up to 720 most recent entries
- Last entry in array is for current (uncommitted) timeframe
- Older data cannot be retrieved regardless of `since` value

**Response Format**:
```json
{
  "error": [],
  "result": {
    "XXBTZUSD": [
      [
        1234567890,  // time (unix timestamp)
        "34900.0",   // open
        "34950.0",   // high
        "34890.0",   // low
        "34920.0",   // close
        "34915.0",   // vwap
        "125.5",     // volume
        1500         // count (number of trades)
      ]
    ],
    "last": 1234567890
  }
}
```

---

### 4. get_ticker (Spot)
**Endpoint**: Same as `get_price` - `GET /0/public/Ticker`

See `get_price` above for full details.

---

### 5. ping (Spot)
**Endpoint**: `GET /0/public/Time`

**Description**: Get the server's time (can be used for connectivity check)

**Response Format**:
```json
{
  "error": [],
  "result": {
    "unixtime": 1234567890,
    "rfc1123": "Mon, 01 Jan 2024 12:00:00 +0000"
  }
}
```

**Alternative**: `GET /0/public/SystemStatus` - Returns system status and trading mode

---

## Trading Trait Endpoints (Spot)

### 1. market_order
**Endpoint**: `POST /0/private/AddOrder`

**Required Parameters**:
- `nonce`: Always-increasing 64-bit unsigned integer
- `ordertype`: `market`
- `type`: `buy` or `sell`
- `volume`: Order volume in lots
- `pair`: Asset pair

**Optional Parameters**:
- `price`: Price (required for limit orders)
- `otp`: Two-factor password (if enabled)
- `userref`: User reference id (32-bit signed number)
- `validate`: Validate inputs only (do not submit order)
- `close[ordertype]`: Conditional close order type
- `close[price]`: Conditional close price
- `trading_agreement`: Must be set to `agree` for certain pairs

**Response**: Returns transaction ID (`txid`) array

**Permissions Required**: `Orders and trades - Create & modify orders`

---

### 2. limit_order
**Endpoint**: `POST /0/private/AddOrder`

**Required Parameters**:
- `nonce`
- `ordertype`: `limit`
- `type`: `buy` or `sell`
- `volume`: Order volume in lots
- `pair`: Asset pair
- `price`: Limit price

**Optional Parameters**: Same as market_order

**Additional Order Types Available**:
- `stop-loss`: Stop loss order
- `take-profit`: Take profit order
- `stop-loss-limit`: Stop loss limit order
- `take-profit-limit`: Take profit limit order
- `settle-position`: Settle position

---

### 3. cancel_order
**Endpoint**: `POST /0/private/CancelOrder`

**Required Parameters**:
- `nonce`
- One of:
  - `txid`: Transaction ID (can be comma-delimited list)
  - `userref`: User reference id
  - `cl_ord_id`: Client order ID

**Response**: Returns count of orders cancelled

**Permissions Required**: Either:
- `Orders and trades - Create & modify orders`, OR
- `Orders and trades - Cancel & close orders`

**Related Endpoints**:
- `POST /0/private/CancelAll`: Cancel all open orders
- `POST /0/private/CancelOrderBatch`: Cancel multiple orders (max 50)

---

### 4. get_order
**Endpoint**: `POST /0/private/QueryOrders`

**Required Parameters**:
- `nonce`
- `txid`: Transaction ID(s) (comma-delimited, up to 50)

**Optional Parameters**:
- `trades`: Whether to include trades related to position in output
- `userref`: Filter by user reference id
- `cl_ord_id`: Filter by client order ID

**Response**: Returns detailed order information including:
- Order description, status, type
- Price, volume (initial and executed)
- Cost, fee
- Open/close times
- Related trade IDs

**Permissions Required**:
- `Orders and trades - Query open orders & trades`, OR
- `Orders and trades - Query closed orders & trades`

---

### 5. get_open_orders
**Endpoint**: `POST /0/private/OpenOrders`

**Required Parameters**:
- `nonce`

**Optional Parameters**:
- `trades`: Include trades in output
- `userref`: Filter by user reference ID

**Response**: Returns information about currently open orders (same format as get_order)

**Permissions Required**: `Orders and trades - Query open orders & trades`

---

## Account Trait Endpoints (Spot)

### 1. get_balance
**Endpoint**: `POST /0/private/Balance`

**Required Parameters**:
- `nonce`

**Response**: Returns all cash balances, net of pending withdrawals

**Response Format**:
```json
{
  "error": [],
  "result": {
    "ZUSD": "10000.5000",
    "XXBT": "0.12345678",
    "XETH": "5.00000000"
  }
}
```

**Asset Extensions**:
- `.B`: Yield-bearing product balances
- `.F`: Automatically earning Kraken Rewards balances
- `.T`: Tokenized asset balances

**Permissions Required**: `Funds permissions - Query`

---

### 2. get_account_info
**Endpoint**: `POST /0/private/TradeBalance`

**Required Parameters**:
- `nonce`

**Optional Parameters**:
- `asset`: Base asset used to determine balance (default: ZUSD)

**Response**: Returns summary of collateral balances, margin position valuations, equity and margin level

**Response Fields**:
- `eb`: Equivalent balance (combined balance of all currencies)
- `tb`: Trade balance (combined balance of all equity currencies)
- `m`: Margin amount of open positions
- `n`: Unrealized net profit/loss of open positions
- `c`: Cost basis of open positions
- `v`: Current floating valuation of open positions
- `e`: Equity (= trade balance + unrealized net profit/loss)
- `mf`: Free margin (= equity - initial margin)
- `ml`: Margin level (= (equity / initial margin) * 100)

**Permissions Required**: `Funds permissions - Query`

---

## Positions Trait Endpoints (Futures)

### 1. get_positions
**Endpoint**: `GET /openpositions`

**Base URL**: `https://futures.kraken.com/derivatives/api/v3`

**Authentication**: Required (API key headers)

**Response**: Returns size and average entry price of all open positions, including matured but unsettled contracts

**Response Fields** (typical):
- `side`: `long` or `short`
- `symbol`: Contract symbol
- `size`: Position size
- `price`: Average entry price
- `fill_time`: Last fill timestamp
- `unrealized_funding`: Unrealized funding P&L
- `pnl`: Unrealized P&L

---

### 2. get_funding_rate
**Endpoint**: `GET /historical-funding-rates`

**Base URL**: `https://futures.kraken.com/derivatives/api/v3`

**Parameters**:
- `symbol` (required): Perpetual contract symbol

**Description**: Returns list of historical funding rates for given market

**Response**: Array of funding rate records with timestamps

**Error Codes**:
- 400: "Symbol is invalid or does not reference a perpetual market"

---

### 3. set_leverage
**Endpoint**: `PUT /leveragepreferences`

**Base URL**: `https://futures.kraken.com/derivatives/api/v3`

**Parameters**:
- `symbol` (required): Contract identifier
- `maxLeverage` (optional): Maximum leverage value (setting this enables isolated margin)
- `marginMode` (optional): `isolated` or `cross`

**Description**: Sets contract's margin mode

**Response**: 200 OK on success

**Error Codes**:
- 87: Contract does not exist
- 88: Contract not a multi-collateral futures contract
- 41: Would cause liquidation

**Note**: Specifying maxLeverage automatically switches to isolated margin mode

---

## Futures Trading Endpoints

### send_order (Futures)
**Endpoint**: `POST /sendorder`

**Base URL**: `https://futures.kraken.com/derivatives/api/v3`

**Order Types Supported**:
- `lmt` (limit)
- `mkt` (market - not explicitly in docs but standard)
- `stp` (stop)
- `take_profit`
- `ioc` (immediate-or-cancel)

**Typical Parameters**:
- `orderType`: Order type
- `symbol`: Futures contract symbol
- `side`: `buy` or `sell`
- `size`: Order size
- `limitPrice`: Limit price (for limit orders)
- `stopPrice`: Stop price (for stop orders)
- `cliOrdId`: Client order ID (optional)

**Response**: Returns order ID and status

---

### cancel_order (Futures)
**Endpoint**: `POST /cancelorder`

**Base URL**: `https://futures.kraken.com/derivatives/api/v3`

**Parameters**:
- `order_id`: Order ID to cancel
- OR `cliOrdId`: Client order ID

---

### Get Accounts (Futures Balance)
**Endpoint**: `GET /accounts`

**Base URL**: `https://futures.kraken.com/derivatives/api/v3`

**Authentication**: Required

**Response**: Account information including:
- Digital asset balances
- Instrument balances
- Margin requirements
- Margin trigger estimates
- Available funds
- Open position P&L
- Portfolio value

---

## Summary Table: Endpoint Mapping to Traits

| Trait Method | Spot Endpoint | Futures Endpoint |
|--------------|---------------|------------------|
| **MarketData** |
| get_price | GET /0/public/Ticker | GET /tickers |
| get_orderbook | GET /0/public/Depth | GET /orderbook |
| get_klines | GET /0/public/OHLC | GET /charts |
| get_ticker | GET /0/public/Ticker | GET /tickers |
| ping | GET /0/public/Time | - |
| **Trading** |
| market_order | POST /0/private/AddOrder | POST /sendorder |
| limit_order | POST /0/private/AddOrder | POST /sendorder |
| cancel_order | POST /0/private/CancelOrder | POST /cancelorder |
| get_order | POST /0/private/QueryOrders | GET /orders/status |
| get_open_orders | POST /0/private/OpenOrders | - |
| **Account** |
| get_balance | POST /0/private/Balance | GET /accounts |
| get_account_info | POST /0/private/TradeBalance | GET /accounts |
| **Positions** (Futures only) |
| get_positions | - | GET /openpositions |
| get_funding_rate | - | GET /historical-funding-rates |
| set_leverage | - | PUT /leveragepreferences |

---

## Notes

1. **Spot vs Futures**: Kraken maintains completely separate APIs for Spot and Futures trading with different base URLs and authentication methods.

2. **HTTP Methods**: Note that Kraken Spot uses POST for all private endpoints (even queries), while Futures uses RESTful conventions (GET for queries, POST for mutations, PUT for updates).

3. **Symbol Formats**: Spot and Futures use different symbol naming conventions (see symbols.md).

4. **Asset Pairs Endpoint**: Use `GET /0/public/AssetPairs` to retrieve all available trading pairs with metadata (wsname, altname, price precision, lot sizes, etc.).

5. **Response Format**: All Spot responses include `error` and `result` keys. Futures responses use `result: "success"` format.
