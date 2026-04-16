# Dhan - Complete Endpoint Reference

## Category: Trading - Order Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/orders | Place new order | Yes | Yes | 25/sec, 250/min, 1000/hr, 7000/day | Static IP required |
| PUT | /v2/orders/{order-id} | Modify pending order | Yes | Yes | 25/sec, max 25 modifications per order | Can modify price, qty, type, validity |
| DELETE | /v2/orders/{order-id} | Cancel order | Yes | Yes | 25/sec | - |
| GET | /v2/orders | Get order book | Yes | Yes | 20/sec | All orders for the day |
| GET | /v2/orders/{order-id} | Get order status | Yes | Yes | 20/sec | Single order details |
| POST | /v2/orders/slicing | Place sliced order | Yes | Yes | 25/sec | For orders above freeze limit |

## Category: Trading - Super Orders (Advanced)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/super/orders | Place super order | Yes | Yes | 25/sec | Entry + Target + SL + Trailing |
| PUT | /v2/super/orders/{order-id} | Modify super order | Yes | Yes | 25/sec | Can modify all legs |
| DELETE | /v2/super/orders/{order-id}/{order-leg} | Cancel super order leg | Yes | Yes | 25/sec | Leg: ENTRY/TARGET/STOPLOSS |
| GET | /v2/super/orders | Get super order book | Yes | Yes | 20/sec | All super orders |
| GET | /v2/super/orders/{order-id} | Get super order status | Yes | Yes | 20/sec | Single super order details |

## Category: Trading - Forever Orders (GTT)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/forever/orders | Place forever order | Yes | Yes | 25/sec | Valid for 365 days |
| PUT | /v2/forever/orders/{order-id} | Modify forever order | Yes | Yes | 25/sec | Single or OCO orders |
| DELETE | /v2/forever/orders/{order-id} | Cancel forever order | Yes | Yes | 25/sec | - |
| GET | /v2/forever/orders | Get forever order book | Yes | Yes | 20/sec | All forever orders |

## Category: Trading - Trade History

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/trades/{order-id} | Get trades by order ID | Yes | Yes | 20/sec | All trades for specific order |
| GET | /v2/trades/{from-date}/{to-date}/{page} | Get trade history | Yes | Yes | 20/sec | Date format: YYYY-MM-DD |

## Category: Portfolio Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/holdings | Get holdings | Yes | Yes | 20/sec | Delivered stocks (T1 + T2) |
| GET | /v2/positions | Get positions | Yes | Yes | 20/sec | Intraday + F&O positions |
| POST | /v2/positions/convert | Convert position | Yes | Yes | 25/sec | Between product types |

## Category: Funds & Statements

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/funds | Get fund limits | Yes | Yes | 20/sec | Available margin/balance |
| GET | /v2/ledger | Get ledger report | Yes | Yes | 20/sec | Requires from-date, to-date query params |

## Category: Market Data - Real-time Quotes

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/marketfeed/ltp | Get Last Traded Price | Conditional | Yes | 1/sec | Up to 1000 instruments |
| POST | /v2/marketfeed/ohlc | Get OHLC + LTP | Conditional | Yes | 1/sec | Up to 1000 instruments |
| POST | /v2/marketfeed/quote | Get full quote + depth | Conditional | Yes | 1/sec | Up to 1000 instruments, includes 5-level depth |

## Category: Market Data - Historical Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/charts/historical | Daily historical data | Conditional | Yes | 5/sec, 100k/day | From instrument inception |
| POST | /v2/charts/intraday | Intraday historical data | Conditional | Yes | 5/sec, 100k/day | 1m/5m/15m/25m/60m, last 5 years, 90-day window |

## Category: Market Data - Option Chain

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/optionchain | Get option chain | Conditional | Yes | 1 req/3sec | OI, Greeks, volume, bid/ask, all strikes |

## Category: Market Data - Instruments

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/instrument/{exchangeSegment} | Get instrument list | Yes | No | Unlimited | CSV format, NSE_EQ/NSE_FNO/BSE_EQ/MCX_COMM |

## Category: EDIS (Electronic Delivery)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/edis/tpin | Generate T-PIN | Yes | Yes | 20/sec | Sent to registered mobile |
| POST | /v2/edis/form | Get EDIS form | Yes | Yes | 20/sec | CDSL HTML form for stock marking |
| POST | /v2/edis/inquiry | Check EDIS status | Yes | Yes | 20/sec | Status of EDIS requests |

## Category: Authentication & Account

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/access_token | Generate access token | Yes | API Key | 20/sec | Requires API key + secret |
| POST | /v2/access_token/renew | Renew access token | Yes | Yes | 20/sec | Only for web-generated tokens |

## Category: WebSocket Control

WebSocket connections don't use REST endpoints for control. All control happens via:
- Query parameters for authentication (token, clientId, authType)
- JSON messages for subscription/unsubscription (sent over WebSocket)

## Parameters Reference

### POST /v2/orders
**Headers:**
- `Content-Type: application/json`
- `access-token: JWT`

**Body Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| dhanClientId | string | Yes | - | Client ID |
| transactionType | string | Yes | - | BUY or SELL |
| exchangeSegment | string | Yes | - | NSE_EQ, NSE_FNO, BSE_EQ, MCX_COMM |
| productType | string | Yes | - | CNC, INTRADAY, MARGIN, MTF, CO, BO |
| orderType | string | Yes | - | MARKET, LIMIT, STOP_LOSS, STOP_LOSS_MARKET |
| validity | string | Yes | - | DAY, IOC |
| securityId | string | Yes | - | Security ID from instrument list |
| quantity | integer | Yes | - | Order quantity |
| disclosedQuantity | integer | No | 0 | Disclosed quantity |
| price | float | Conditional | - | Required for LIMIT orders |
| triggerPrice | float | Conditional | - | Required for STOP_LOSS orders |
| afterMarketOrder | boolean | No | false | AMO order flag |
| amoTime | string | No | OPEN | OPEN or OPEN_30 or OPEN_60 |
| boProfitValue | float | Conditional | - | For BO orders |
| boStopLossValue | float | Conditional | - | For BO orders |

### PUT /v2/orders/{order-id}
**Modifiable Parameters:**
- `quantity` - Order quantity
- `orderType` - MARKET, LIMIT, STOP_LOSS, STOP_LOSS_MARKET
- `legName` - For multi-leg orders
- `price` - Limit price
- `disclosedQuantity` - Disclosed quantity
- `triggerPrice` - Stop loss trigger
- `validity` - DAY, IOC

### POST /v2/super/orders
**Additional Parameters (beyond regular order):**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| targetPrice | float | Yes | - | Target profit price |
| stopLossPrice | float | Yes | - | Stop loss price |
| trailingJump | float | No | 0 | Trailing SL jump value |

### POST /v2/charts/historical
**Body Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| securityId | string | Yes | - | Security ID |
| exchangeSegment | string | Yes | - | NSE_EQ, NSE_FNO, BSE_EQ, MCX_COMM |
| instrument | string | Yes | - | EQUITY, FUTIDX, FUTSTK, OPTIDX, OPTSTK |
| expiryCode | integer | No | 0 | For derivatives |
| fromDate | string | Yes | - | YYYY-MM-DD format |
| toDate | string | Yes | - | YYYY-MM-DD format |

### POST /v2/charts/intraday
**Body Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| securityId | string | Yes | - | Security ID |
| exchangeSegment | string | Yes | - | Exchange segment |
| instrument | string | Yes | - | Instrument type |
| interval | string | Yes | - | 1, 5, 15, 25, 60 (minutes) |
| oi | boolean | No | false | Include Open Interest (for F&O) |
| fromDate | string | Yes | - | YYYY-MM-DD format |
| toDate | string | Yes | - | YYYY-MM-DD format (max 90 days from fromDate) |

### POST /v2/marketfeed/ltp
**Body Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| NSE_EQ | array | No | [] | Array of security IDs for NSE Equity |
| NSE_FNO | array | No | [] | Array of security IDs for NSE F&O |
| BSE_EQ | array | No | [] | Array of security IDs for BSE Equity |
| MCX_COMM | array | No | [] | Array of security IDs for MCX Commodities |

**Note:** Total instruments across all segments must be ≤ 1000

### POST /v2/marketfeed/quote
Same parameters as /v2/marketfeed/ltp

### POST /v2/optionchain
**Body Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| securityId | string | Yes | - | Underlying security ID |
| exchangeSegment | string | Yes | - | NSE_FNO, BSE_FNO, MCX_COMM |
| expiryDate | string | Yes | - | YYYY-MM-DD format |

### GET /v2/ledger
**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| from-date | string | Yes | - | YYYY-MM-DD format |
| to-date | string | Yes | - | YYYY-MM-DD format |

### POST /v2/positions/convert
**Body Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| dhanClientId | string | Yes | - | Client ID |
| exchangeSegment | string | Yes | - | Exchange segment |
| securityId | string | Yes | - | Security ID |
| transactionType | string | Yes | - | BUY or SELL |
| positionType | string | Yes | - | LONG or SHORT |
| convertQty | integer | Yes | - | Quantity to convert |
| convertFrom | string | Yes | - | Source product type |
| convertTo | string | Yes | - | Target product type |

## Notes on Data API Access
- **Free Access**: If you complete 25+ trades in previous 30 days
- **Paid Access**: Rs. 499 + taxes per month if <25 trades
- **Trading APIs**: Always free for all Dhan users
- Data APIs include: Historical data, Market quotes, Live market feed, Option chain

## Static IP Requirement (2026 Update)
- **Required For**: All Order APIs (place/modify/cancel)
- **Not Required For**:
  - Data APIs
  - Sandbox environment
  - Portfolio/Holdings queries
- **How to Set**: Configure in Dhan web portal under API settings
- **Implementation Date**: January 2026

## Rate Limit Summary Table

| Category | Per Second | Per Minute | Per Hour | Per Day |
|----------|-----------|------------|----------|---------|
| Order APIs | 25 | 250 | 1,000 | 7,000 |
| Data APIs | 5 | - | - | 100,000 |
| Quote APIs | 1 | Unlimited | Unlimited | Unlimited |
| Non-Trading APIs | 20 | Unlimited | Unlimited | Unlimited |

**Additional Constraints:**
- Order modifications: Max 25 per order
- Option Chain: 1 request per 3 seconds (due to slow OI updates)
- WebSocket subscriptions: Max 5000 instruments per connection
- WebSocket connections: Max 5 concurrent connections per user
