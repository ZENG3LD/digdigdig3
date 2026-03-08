# Fyers - Complete Endpoint Reference

Base URLs:
- REST API: `https://api.fyers.in`
- Data API: `https://api-t1.fyers.in`

All endpoints require authentication unless otherwise noted.

---

## Category: User/Account APIs

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /api/v3/profile | Get user profile details | Yes | Yes | Part of daily | Returns client details |
| GET | /api/v3/funds | Get available funds/balance | Yes | Yes | Part of daily | Capital & commodity markets |
| GET | /api/v3/holdings | Get equity holdings | Yes | Yes | Part of daily | Includes mutual funds, T1 qty |

### GET /api/v3/profile
**Response Fields:**
- Client details (name, ID, email, mobile)
- PAN, DP ID
- Account type and status

### GET /api/v3/funds
**Response Fields:**
- `fund_limit` - Total available funds
- `collateral` - Collateral amount
- `utilisedDebits` - Utilized debits
- `payout` - Available for payout
- Separate capital market and commodity market balances

### GET /api/v3/holdings
**Response Fields:**
- `symbol` - Trading symbol
- `holdingType` - Type of holding
- `quantity` - Total quantity
- `remainingQuantity` - Available quantity
- `costPrice` - Average cost price
- `marketVal` - Current market value
- `ltp` - Last traded price
- `pl` - Profit/loss
- `fyToken` - Fyers token
- `exchange`, `segment`, `isin`
- `qty_t1` - T1 quantity
- `remainingPledgeQuantity`
- `collateralQuantity`

---

## Category: Transaction/Portfolio APIs

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /api/v3/tradebook | Get trade history | Yes | Yes | Part of daily | Current trading day |
| GET | /api/v3/orderbook | Get order book | Yes | Yes | Part of daily | Active & completed orders |
| GET | /api/v3/positions | Get open positions | Yes | Yes | Part of daily | Current day positions |
| GET | /api/v3/orders/{order_id} | Get single order details | Yes | Yes | Part of daily | Order ID required |

### GET /api/v3/tradebook
**Response Fields:**
- Trade ID, order ID
- Symbol, exchange, segment
- Trade time, quantity, price
- Side (buy/sell)
- Product type

### GET /api/v3/orderbook
**Response Fields:**
- Order ID, client ID
- Symbol, side, type
- Quantity (total, filled, remaining)
- Price (limit, stop)
- Status, validity
- Order time, update time
- Product type
- Disclosed quantity

### GET /api/v3/positions
**Response Fields:**
- Symbol, side
- Net quantity
- Average price (buy, sell)
- Realized P&L, unrealized P&L
- Product type
- Cross currency (for currency derivatives)

---

## Category: Order Placement & Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /api/v3/orders | Place single order | Yes | Yes | 10/sec, 200/min | Execution <50ms |
| POST | /api/v3/orders-multi | Place basket orders | Yes | Yes | 10/sec, 200/min | Up to 10 orders |
| POST | /api/v3/orders-multileg | Place multi-leg order | Yes | Yes | 10/sec, 200/min | 2-3 leg strategies |
| PUT | /api/v3/orders | Modify single order | Yes | Yes | 10/sec, 200/min | Pending orders only |
| PUT | /api/v3/orders-multi | Modify basket orders | Yes | Yes | 10/sec, 200/min | Batch modification |
| DELETE | /api/v3/orders | Cancel single order | Yes | Yes | 10/sec, 200/min | Order ID required |
| DELETE | /api/v3/orders-multi | Cancel basket orders | Yes | Yes | 10/sec, 200/min | Multiple cancellations |
| DELETE | /api/v3/positions | Exit position | Yes | Yes | 10/sec, 200/min | Close position |
| PUT | /api/v3/positions | Convert position | Yes | Yes | 10/sec, 200/min | Change product type |

### POST /api/v3/orders (Place Order)
**Required Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | string | Yes | Trading symbol (e.g., "NSE:SBIN-EQ") |
| qty | int | Yes | Order quantity |
| type | int | Yes | 1=LIMIT, 2=MARKET, 3=STOP, 4=STOPLIMIT |
| side | int | Yes | 1=BUY, -1=SELL |
| productType | string | Yes | INTRADAY, CNC, MARGIN, CO, BO |
| limitPrice | float | Conditional | Required for LIMIT, STOPLIMIT |
| stopPrice | float | Conditional | Required for STOP, STOPLIMIT |
| validity | string | No | DAY (default), IOC |
| disclosedQty | int | No | Disclosed quantity (equity only) |
| offlineOrder | bool | No | True for AMO (After Market Order) |
| stopLoss | float | Conditional | Required for CO, BO |
| takeProfit | float | No | For BO orders |

**Product Types:**
- `INTRADAY` - Intraday square-off
- `CNC` - Cash and Carry (delivery)
- `MARGIN` - Margin (derivatives only)
- `CO` - Cover Order
- `BO` - Bracket Order

**Order Types:**
- `1` - LIMIT
- `2` - MARKET
- `3` - STOP (stop-loss market)
- `4` - STOPLIMIT (stop-loss limit)

**Validity:**
- `DAY` - Valid till end of day
- `IOC` - Immediate or Cancel

### POST /api/v3/orders-multi (Basket Orders)
**Parameters:**
Same as single order, but accepts array of up to 10 orders in request body.

### POST /api/v3/orders-multileg (Multi-leg Orders)
**Parameters:**
- `orderType` - "3L" for 3-leg orders, "2L" for 2-leg
- Multiple symbol/qty/price combinations for each leg
- Supported strategies: spreads, straddles, strangles

### PUT /api/v3/orders (Modify Order)
**Parameters:**
- `id` - Order ID (required)
- `type` - New order type (optional)
- `limitPrice` - New limit price (optional)
- `stopPrice` - New stop price (optional)
- `qty` - New quantity (optional)
- Only modifiable fields need to be sent

### DELETE /api/v3/orders (Cancel Order)
**Parameters:**
- `id` - Order ID (required)

### DELETE /api/v3/positions (Exit Position)
**Parameters:**
- `id` - Position ID (required)
- Closes entire position at market price

### PUT /api/v3/positions (Convert Position)
**Parameters:**
- `symbol` - Trading symbol
- `positionSide` - 1 or -1
- `convertQty` - Quantity to convert
- `convertFrom` - Current product type
- `convertTo` - Target product type

---

## Category: Market Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/quotes | Get market quotes | Yes | Yes | 10/sec, 200/min | Multiple symbols |
| GET | /data/depth/ | Get market depth (L2) | Yes | Yes | 10/sec, 200/min | Top 5 bid/ask |
| GET | /data/history | Get historical OHLC data | Yes | Yes | 10/sec, 200/min | Candles/bars |
| GET | /data/market-status | Get market status | Yes | No | 10/sec, 200/min | Exchange/segment status |

### GET /data/quotes
**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbols | string | Yes | Comma-separated symbols (e.g., "NSE:SBIN-EQ,NSE:RELIANCE-EQ") |

**Response Fields (per symbol):**
- `symbol` - Trading symbol
- `open`, `high`, `low`, `close` - OHLC
- `ltp` - Last traded price
- `prev_close_price` - Previous close
- `chp` - Change in points
- `ch` - Change percentage
- `volume` - Total volume
- `bid`, `ask` - Best bid/ask prices
- `bid_size`, `ask_size` - Bid/ask quantities
- `fyToken` - Fyers token
- `description` - Symbol description
- `original_name` - Full name
- `exchange`, `segment`, `instrument_type`
- `expiry` - Expiry date (for derivatives)
- `strike_price` - Strike price (for options)
- `option_type` - CE or PE (for options)
- `timestamp` - Update timestamp

### GET /data/depth/
**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | string | Yes | Single symbol (e.g., "NSE:SBIN-EQ") |
| ohlcv_flag | int | No | 1 to include OHLCV data |

**Response Fields:**
- `bids` - Array of top 5 bids [price, volume, orders]
- `ask` - Array of top 5 asks [price, volume, orders]
- `totalbuyqty` - Total buy quantity
- `totalsellqty` - Total sell quantity
- `timestamp` - Update timestamp
- OHLCV data (if ohlcv_flag=1):
  - `open`, `high`, `low`, `close`, `volume`
  - `ltp`, `prev_close_price`, `ch`, `chp`

### GET /data/history
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Trading symbol |
| resolution | string | Yes | - | Timeframe (see below) |
| date_format | int | No | 0 | 0=Unix timestamp, 1=DD-MM-YYYY |
| range_from | string | Yes | - | Start date (Unix or formatted) |
| range_to | string | Yes | - | End date (Unix or formatted) |
| cont_flag | int | No | 0 | Continuous flag for futures |

**Resolution Values:**
- `1` - 1 minute
- `2` - 2 minutes
- `3` - 3 minutes
- `5` - 5 minutes
- `10` - 10 minutes
- `15` - 15 minutes
- `30` - 30 minutes
- `45` - 45 minutes
- `60` - 1 hour
- `120` - 2 hours
- `180` - 3 hours
- `240` - 4 hours
- `1D` - 1 day
- `1W` - 1 week
- `1M` - 1 month

**Response Format:**
```json
{
  "s": "ok",
  "candles": [
    [timestamp, open, high, low, close, volume],
    [timestamp, open, high, low, close, volume],
    ...
  ]
}
```

### GET /data/market-status
**Parameters:** None required

**Response Fields:**
- Exchange-wise status (NSE, BSE, MCX, NCDEX)
- Segment-wise status (CM, FO, CD, COMM)
- Current status (open/closed)
- Market timings

---

## Category: Metadata & Reference Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/symbol-master | Get symbol master | Yes | No | Part of daily | CSV download |
| GET | /api/v3/market-calendar | Get trading calendar | Yes | No | Part of daily | Holidays, timings |

### GET /data/symbol-master
**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| exchange | string | Yes | NSE, BSE, MCX, NCDEX |
| segment | string | No | CM, FO, CD, COMM |

**Returns:** CSV file with columns:
- `fytoken` - Fyers internal token
- `symbol` - Trading symbol
- `exchange`, `segment`
- `description` - Full name
- `lot_size` - Lot size (for derivatives)
- `tick_size` - Tick size
- `isin` - ISIN code
- `series` - EQ, BE, etc.
- `expiry_date` - Expiry (for derivatives)
- `strike_price`, `option_type` - For options

---

## Category: E-DIS (Electronic Delivery of Securities)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /api/v3/edis/generate-tpin | Generate TPIN | Yes | Yes | Part of daily | For CDSL authorization |
| GET | /api/v3/edis/transactions | Get EDIS transactions | Yes | Yes | Part of daily | Holdings & status |
| POST | /api/v3/edis/submit-holdings | Submit holdings | Yes | Yes | Part of daily | Redirect to CDSL |
| POST | /api/v3/edis/inquire-transaction | Inquire transaction status | Yes | Yes | Part of daily | Check EDIS status |

---

## Category: Authentication

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /api/v3/generate-authcode | Generate auth code URL | Yes | No | - | Step 1 of OAuth flow |
| POST | /api/v3/validate-authcode | Validate auth code | Yes | No | - | Exchange for access token |
| POST | /api/v3/token | Generate access token | Yes | No | - | Step 2 of OAuth flow |

### POST /api/v3/generate-authcode
**Parameters:**
- `client_id` - App ID
- `redirect_uri` - Redirect URI
- `response_type` - "code"
- `state` - Session state

**Returns:** Authorization URL for user login

### POST /api/v3/validate-authcode / POST /api/v3/token
**Parameters:**
- `grant_type` - "authorization_code"
- `appIdHash` - SHA-256(api_id + app_secret)
- `code` - Authorization code from redirect

**Returns:**
```json
{
  "s": "ok",
  "code": 200,
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

---

## Rate Limits Summary

**Global Rate Limits (API V3):**
- 10 requests per second
- 200 requests per minute
- 100,000 requests per day

**Response Headers:**
```
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 195
X-RateLimit-Reset: 1640000000
```

**Rate Limit Error (HTTP 429):**
```json
{
  "s": "error",
  "code": 429,
  "message": "request limit reached"
}
```

---

## Symbol Format

**General Format:** `EXCHANGE:SYMBOL-SERIES`

**Examples:**
- Equity: `NSE:SBIN-EQ`
- Futures: `NSE:NIFTY24JANFUT`
- Options: `NSE:NIFTY2411921500CE`
- BSE Stock: `BSE:SENSEX-EQ`
- Commodity: `MCX:GOLDM24JANFUT`
- Currency: `NSE:USDINR24JANFUT`

**Series Types:**
- `EQ` - Equity
- `BE` - Book Entry
- `SM` - SME
- Various others for different categories

---

## Error Codes Reference

| Code | Description | Resolution |
|------|-------------|------------|
| 200 | Success | - |
| 401 | Unauthorized / Unauthenticated | Check access token |
| 403 | Forbidden | Check permissions |
| 429 | Rate limit exceeded | Wait and retry |
| -1600 | Authentication failed | Re-authenticate |
| -351 | Symbol limit exceeded | Reduce symbol count |
| -100 | Invalid parameters | Check request format |

---

## Notes

1. All timestamps are in Unix milliseconds unless specified otherwise
2. Historical data availability varies by symbol and timeframe
3. Options historical data may be limited to daily timeframe
4. Basket orders limited to 10 orders per request
5. Multi-leg orders support 2-3 leg strategies
6. WebSocket subscriptions have separate limits (see websocket_full.md)
7. Symbol master files are updated daily
8. Market status reflects real-time exchange status
9. Funds and positions updated in real-time
10. Order execution guaranteed under 50ms for market orders
