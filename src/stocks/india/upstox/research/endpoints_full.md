# Upstox - Complete Endpoint Reference

## Category: Authentication & Authorization

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/login/authorization/dialog | OAuth authorization | Yes | No | N/A | Redirects to login |
| POST | /v2/login/authorization/token | Get access token | Yes | No | N/A | Exchange code for token |

## Category: Market Data - Quotes & Prices

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/market-quote/ltp | Get LTP (Last Traded Price) | Yes | Yes | 50/s, 500/min | Up to 500 instruments |
| GET | /v2/market-quote/quotes | Get full market quotes | Yes | Yes | 50/s, 500/min | Up to 500 instruments, includes depth |
| GET | /v2/market-quote/ohlc | Get OHLC data | Yes | Yes | 50/s, 500/min | Current day OHLC |

## Category: Market Data - Historical

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/historical-candle/{instrument_key}/{interval}/{to_date} | Historical candles (legacy) | Yes | No | 50/s, 500/min | Public endpoint |
| GET | /v2/historical-candle/{instrument_key}/{interval}/{to_date}/{from_date} | Historical candles with range | Yes | No | 50/s, 500/min | Public endpoint |
| GET | /v3/historical-candle/{instrument_key}/{unit}/{interval}/{to_date}/{from_date} | Historical candles v3 | Yes | Yes | 50/s, 500/min | Expanded units & intervals |
| GET | /v2/historical-candle/intraday/{instrument_key}/{interval} | Intraday candles (legacy) | Yes | Yes | 50/s, 500/min | Current day only |
| GET | /v3/historical-candle/intraday/{instrument_key}/{unit}/{interval} | Intraday candles v3 | Yes | Yes | 50/s, 500/min | Expanded intervals |

## Category: Market Data - Option Chain

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/option/chain | Get option chain | Yes | Yes | 50/s, 500/min | Not available for MCX |
| GET | /v2/option/contract | Get option contract details | Yes | Yes | 50/s, 500/min | Specific contract info |

## Category: Market Data - Instruments & Metadata

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| N/A | https://assets.upstox.com/market-quote/instruments/exchange/complete.json.gz | Complete instruments list | Yes | No | N/A | Download JSON, BOD update ~6AM |
| N/A | https://assets.upstox.com/market-quote/instruments/exchange/NSE.json.gz | NSE instruments | Yes | No | N/A | Download JSON |
| N/A | https://assets.upstox.com/market-quote/instruments/exchange/BSE.json.gz | BSE instruments | Yes | No | N/A | Download JSON |
| N/A | https://assets.upstox.com/market-quote/instruments/exchange/MCX.json.gz | MCX instruments | Yes | No | N/A | Download JSON |

## Category: Trading - Order Placement

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/order/place | Place order (legacy) | Paid | Yes | 50/s, 500/min | Deprecated, use v3 |
| POST | /v3/order/place | Place order v3 | Paid | Yes | 50/s, 500/min | Auto-slicing support |
| POST | /v2/order/multi/place | Place multiple orders | Paid | Yes | 4/s, 40/min | Beta, max 200 orders |
| PUT | /v2/order/modify | Modify order | Paid | Yes | 50/s, 500/min | Modify pending order |
| DELETE | /v2/order/cancel | Cancel order | Paid | Yes | 50/s, 500/min | Cancel pending order |
| DELETE | /v2/order/multi/cancel | Cancel all orders | Paid | Yes | 4/s, 40/min | Beta, by segment/tag |

## Category: Trading - GTT Orders

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v3/order/gtt/place | Place GTT order | Paid | Yes | 50/s, 500/min | Good Till Trigger |
| PUT | /v3/order/gtt/modify | Modify GTT order | Paid | Yes | 50/s, 500/min | Modify untriggered GTT |
| DELETE | /v3/order/gtt/cancel | Cancel GTT order | Paid | Yes | 50/s, 500/min | Cancel untriggered GTT |
| GET | /v2/gtt/order/{gtt_order_id} | Get GTT order details | Paid | Yes | 50/s, 500/min | Specific GTT order |
| GET | /v2/gtt/orders | Get all GTT orders | Paid | Yes | 50/s, 500/min | Does not include completed |

## Category: Trading - Order Information

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/order/details | Get order book | Paid | Yes | 50/s, 500/min | All orders for the day |
| GET | /v2/order/{order_id} | Get order details | Paid | Yes | 50/s, 500/min | Specific order |
| GET | /v2/order/trades/{order_id} | Get order trades | Paid | Yes | 50/s, 500/min | Trades for specific order |
| GET | /v2/order/trades | Get all trades | Paid | Yes | 50/s, 500/min | All trades for the day |
| GET | /v2/order/history | Get trade history | Paid | Yes | 50/s, 500/min | Historical trades |

## Category: Portfolio - Positions & Holdings

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/portfolio/short-term-positions | Get positions | Paid | Yes | 50/s, 500/min | Day & overnight positions |
| GET | /v2/portfolio/long-term-holdings | Get holdings | Paid | Yes | 50/s, 500/min | Long-term holdings |
| GET | /v3/portfolio/mtf-positions | Get MTF positions | Paid | Yes | 50/s, 500/min | Margin Trading positions |
| PUT | /v2/portfolio/convert-position | Convert position | Paid | Yes | 50/s, 500/min | Intraday to delivery, etc. |
| DELETE | /v2/portfolio/positions | Exit all positions | Paid | Yes | 4/s, 40/min | Beta, max 200 positions |

## Category: Account - Funds & Margins

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/user/get-funds-and-margin | Get fund & margin | Paid | Yes | 50/s, 500/min | Equity & commodity combined |
| GET | /v2/charges/margin/{instrument_key} | Get margin requirement | Paid | Yes | 50/s, 500/min | For specific instrument |

## Category: Account - Charges & P&L

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/trade/profit-loss/charges | Get trade charges | Paid | Yes | 50/s, 500/min | Brokerage, taxes, fees |
| GET | /v2/trade/profit-loss/data | Get P&L data | Paid | Yes | 50/s, 500/min | Trade-wise P&L |
| GET | /v2/charges/brokerage | Get brokerage details | Paid | Yes | 50/s, 500/min | Calculate fees |

## Category: User Information

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/user/profile | Get user profile | Paid | Yes | 50/s, 500/min | User details |

## Category: WebSocket Control

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/feed/market-data-feed/authorize | Authorize market feed | Yes | Yes | N/A | Get WebSocket URL |
| GET | /v2/feed/portfolio-stream-feed/authorize | Authorize portfolio feed | Paid | Yes | N/A | Get WebSocket URL |

## Category: Webhook Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| N/A | Configured in app settings | Webhook registration | Paid | No | N/A | Set via My Apps page |

---

## Parameters Reference

### GET /v2/market-quote/quotes
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| instrument_key | string | Yes | - | Comma-separated list, max 500 instruments |

**Example:** `?instrument_key=NSE_EQ|INE669E01016,BSE_EQ|INE002A01018`

### POST /v3/order/place
**Request Body:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| quantity | integer | Yes | - | Order quantity (units or lots) |
| product | string | Yes | - | I (Intraday), D (Delivery), MTF |
| validity | string | Yes | DAY | DAY or IOC |
| price | float | Yes | - | Order price (0 for market) |
| tag | string | No | - | Custom identifier |
| instrument_token | string | Yes | - | e.g., NSE_EQ\|INE669E01016 |
| order_type | string | Yes | - | MARKET, LIMIT, SL, SL-M |
| transaction_type | string | Yes | - | BUY or SELL |
| disclosed_quantity | integer | Yes | 0 | Visible quantity in depth |
| trigger_price | float | Yes | 0 | For SL/SL-M orders |
| is_amo | boolean | Yes | false | After-market order |
| slice | boolean | No | true | Auto-slice large orders (v3) |

### GET /v3/historical-candle/{instrument_key}/{unit}/{interval}/{to_date}/{from_date}
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| instrument_key | string | Yes | e.g., NSE_EQ\|INE848E01016 |
| unit | string | Yes | minutes, hours, days, weeks, months |
| interval | string | Yes | 1-300 (minutes), 1-5 (hours), 1 (others) |
| to_date | string | Yes | Format: YYYY-MM-DD (inclusive) |
| from_date | string | No | Format: YYYY-MM-DD |

**Historical Limits:**
- Minutes: Jan 2022+, max 1 month (1-15min) or 1 quarter (>15min)
- Hours: Jan 2022+, max 1 quarter
- Days: Jan 2000+, max 1 decade
- Weeks/Months: Jan 2000+, unlimited

### GET /v2/option/chain
**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| instrument_key | string | Yes | Underlying symbol (e.g., NSE_INDEX\|Nifty 50) |
| expiry_date | string | Yes | Format: YYYY-MM-DD |

### GET /v2/trade/profit-loss/charges
**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| segment | string | Yes | EQ, FO, COM, or CD |
| financial_year | string | Yes | e.g., 2324 (FY 2023-24) |
| from_date | string | No | Format: dd-mm-yyyy |
| to_date | string | No | Format: dd-mm-yyyy |

### DELETE /v2/portfolio/positions
**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| segment | string | No | Filter by segment (EQ, FO, COM, CD) |
| tag | string | No | Filter by tag |

**Note:** Max 200 positions per request, does not support Delivery in EQ segment

---

## Error Codes Reference

| Code | Description | Resolution |
|------|-------------|------------|
| UDAPI1026 | Instrument key required | Provide valid instrument_key |
| UDAPI1004 | Valid order type required | Use MARKET, LIMIT, SL, or SL-M |
| UDAPI1052 | Order quantity cannot be zero | Set quantity > 0 |
| UDAPI100074 | API accessible 5:30 AM - 12:00 AM IST | Check timing |
| UDAPI100049 | Access restricted | Use Uplink Business API |
| UDAPI1087 | Invalid symbol/instrument_key | Check format |
| UDAPI100042 | Max 500 instruments per request | Reduce count |
| UDAPI100011 | Invalid instrument key | Verify instrument exists |
| UDAPI1088 | Improper date formatting | Use correct format |
| 401 | Unauthorized | Check access token |
| 429 | Rate limit exceeded | Wait or reduce request rate |
