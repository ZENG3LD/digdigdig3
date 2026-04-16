# Zerodha Kite Connect - Account Management Endpoints

## Overview

Account management endpoints provide access to user profile, funds/margins, portfolio (holdings/positions), and account-related operations.

---

## User Profile & Authentication

### 1. Generate Session (Token Exchange)

**Method**: `POST`

**Endpoint**: `/session/token`

**URL**: `https://api.kite.trade/session/token`

**Purpose**: Complete login flow by exchanging request_token for access_token

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| api_key | string | Yes | Public API key |
| request_token | string | Yes | One-time token from login callback |
| checksum | string | Yes | SHA-256(api_key + request_token + api_secret) |

**Response**:
```json
{
  "status": "success",
  "data": {
    "user_id": "XX0000",
    "user_type": "individual",
    "email": "[email protected]",
    "user_name": "User Full Name",
    "user_shortname": "User",
    "broker": "ZERODHA",
    "exchanges": ["NSE", "BSE", "NFO", "BFO", "MCX", "CDS", "BCD", "MF"],
    "products": ["CNC", "NRML", "MIS", "MTF"],
    "order_types": ["MARKET", "LIMIT", "SL", "SL-M"],
    "avatar_url": null,
    "api_key": "your_api_key",
    "access_token": "generated_access_token",
    "public_token": "generated_public_token",
    "enctoken": "encrypted_token",
    "refresh_token": "",
    "login_time": "2026-01-26 10:30:45",
    "meta": {
      "demat_consent": "physical"
    }
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| user_id | string | Unique user identifier (e.g., "XX0000") |
| user_type | string | "individual" for retail users |
| email | string | User's registered email |
| user_name | string | User's full name |
| user_shortname | string | User's short name |
| broker | string | Always "ZERODHA" |
| exchanges | array | Enabled exchanges for the user |
| products | array | Allowed products (CNC, NRML, MIS, MTF) |
| order_types | array | Allowed order types |
| avatar_url | string | User avatar URL (usually null) |
| api_key | string | API key used |
| access_token | string | Access token for API requests (expires 6 AM) |
| public_token | string | Public session token |
| enctoken | string | Encrypted token (internal) |
| refresh_token | string | Refresh token (limited availability) |
| login_time | datetime | Login timestamp |
| meta.demat_consent | string | Demat consent status ("physical") |

**Important Notes**:
- access_token expires daily at 6 AM IST (regulatory requirement)
- request_token is single-use and expires in minutes
- Never embed api_secret in client applications

---

### 2. Get User Profile

**Method**: `GET`

**Endpoint**: `/user/profile`

**URL**: `https://api.kite.trade/user/profile`

**Purpose**: Retrieve user profile without tokens

**Authentication**: `Authorization: token api_key:access_token`

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "data": {
    "user_id": "XX0000",
    "user_type": "individual",
    "email": "[email protected]",
    "user_name": "User Full Name",
    "user_shortname": "User",
    "broker": "ZERODHA",
    "exchanges": ["NSE", "BSE", "NFO", "BFO", "MCX", "CDS", "BCD", "MF"],
    "products": ["CNC", "NRML", "MIS", "MTF"],
    "order_types": ["MARKET", "LIMIT", "SL", "SL-M"],
    "avatar_url": null,
    "meta": {
      "demat_consent": "physical"
    }
  }
}
```

**Response Fields**: Same as token exchange response (without tokens)

**Rate Limit**: 10 requests/second per API key

---

### 3. Logout / Invalidate Session

**Method**: `DELETE`

**Endpoint**: `/session/token`

**URL**: `https://api.kite.trade/session/token?api_key={api_key}&access_token={access_token}`

**Purpose**: Invalidate access_token and destroy API session

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| api_key | string | Yes | API key (query param) |
| access_token | string | Yes | Access token to invalidate (query param) |

**Response**:
```json
{
  "status": "success",
  "data": true
}
```

**Important Notes**:
- Only invalidates API session
- Does NOT log user out of Kite web/mobile apps
- Separate sessions are maintained

**Rate Limit**: 10 requests/second per API key

---

## Funds & Margins

### 1. Get All Margins

**Method**: `GET`

**Endpoint**: `/user/margins`

**URL**: `https://api.kite.trade/user/margins`

**Purpose**: Retrieve margins for all segments (equity and commodity)

**Authentication**: Required

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "data": {
    "equity": {
      "enabled": true,
      "net": 45000.50,
      "available": {
        "adhoc_margin": 0,
        "cash": 50000.00,
        "opening_balance": 50000.00,
        "live_balance": 45000.50,
        "collateral": 0,
        "intraday_payin": 0
      },
      "utilised": {
        "debits": 4999.50,
        "exposure": 0,
        "m2m_realised": 0,
        "m2m_unrealised": 0,
        "option_premium": 0,
        "payout": 0,
        "span": 0,
        "holding_sales": 0,
        "turnover": 0,
        "liquid_collateral": 0,
        "stock_collateral": 0,
        "delivery": 0
      }
    },
    "commodity": {
      "enabled": true,
      "net": 10000.00,
      "available": {
        "adhoc_margin": 0,
        "cash": 10000.00,
        "opening_balance": 10000.00,
        "live_balance": 10000.00,
        "collateral": 0,
        "intraday_payin": 0
      },
      "utilised": {
        "debits": 0,
        "exposure": 0,
        "m2m_realised": 0,
        "m2m_unrealised": 0,
        "option_premium": 0,
        "payout": 0,
        "span": 0,
        "holding_sales": 0,
        "turnover": 0,
        "liquid_collateral": 0,
        "stock_collateral": 0,
        "delivery": 0
      }
    }
  }
}
```

**Margin Structure Fields**:

| Field | Type | Description |
|-------|------|-------------|
| enabled | bool | Segment active for user |
| net | float | Net available margin (cash - utilised) |
| available.adhoc_margin | float | Additional margin provided |
| available.cash | float | Raw tradeable cash balance |
| available.opening_balance | float | Balance at market open |
| available.live_balance | float | Current real-time balance |
| available.collateral | float | Pledged collateral value |
| available.intraday_payin | float | Intraday payin amount |
| utilised.debits | float | Total debited amount |
| utilised.exposure | float | F&O exposure margin blocked |
| utilised.m2m_realised | float | Booked mark-to-market P&L |
| utilised.m2m_unrealised | float | Unbooked mark-to-market P&L |
| utilised.option_premium | float | Option premium blocked |
| utilised.payout | float | Payout amount |
| utilised.span | float | F&O SPAN margin blocked |
| utilised.holding_sales | float | Holdings sold but not yet settled |
| utilised.turnover | float | Turnover charges blocked |
| utilised.liquid_collateral | float | Liquid securities pledged |
| utilised.stock_collateral | float | Stock securities pledged |
| utilised.delivery | float | 20% of sold securities (T+2 settlement) |

**Key Concepts**:
- **net**: This is the actual available margin for trading
- **SPAN**: Standard Portfolio Analysis of Risk (exchange margin)
- **Exposure**: Additional exposure margin for F&O
- **M2M**: Mark-to-market (profit/loss on open positions)
- **Collateral**: Securities pledged as margin

**Rate Limit**: 10 requests/second per API key

---

### 2. Get Segment-Specific Margins

**Method**: `GET`

**Endpoint**: `/user/margins/{segment}`

**URL**: `https://api.kite.trade/user/margins/{segment}`

**Segments**:
- `equity` - Equity segment
- `commodity` - Commodity segment

**Purpose**: Retrieve margins for a specific segment

**Response**: Same structure as individual segment in Get All Margins

**Example Request**:
```
GET /user/margins/equity
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "enabled": true,
    "net": 45000.50,
    "available": { /* ... */ },
    "utilised": { /* ... */ }
  }
}
```

**Rate Limit**: 10 requests/second per API key

---

## Portfolio Management

### 1. Get Holdings

**Method**: `GET`

**Endpoint**: `/portfolio/holdings`

**URL**: `https://api.kite.trade/portfolio/holdings`

**Purpose**: Retrieve long-term equity holdings (delivery portfolio)

**Authentication**: Required

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "data": [
    {
      "tradingsymbol": "INFY",
      "exchange": "NSE",
      "instrument_token": 408065,
      "isin": "INE009A01021",
      "product": "CNC",
      "price": 0,
      "quantity": 100,
      "used_quantity": 0,
      "realised_quantity": 100,
      "authorised_quantity": 0,
      "t1_quantity": 0,
      "collateral_quantity": 0,
      "collateral_type": "",
      "discrepancy": false,
      "average_price": 1400.00,
      "last_price": 1450.50,
      "close_price": 1438.00,
      "pnl": 5050.00,
      "day_change": 1250.00,
      "day_change_percentage": 0.87
    }
  ]
}
```

**Holdings Fields**:

| Field | Type | Description |
|-------|------|-------------|
| tradingsymbol | string | Trading symbol |
| exchange | string | Exchange (NSE, BSE) |
| instrument_token | int | Instrument token |
| isin | string | ISIN identifier |
| product | string | Always "CNC" for holdings |
| price | float | Not used (always 0) |
| quantity | int | Total holding quantity |
| used_quantity | int | Quantity used for open orders |
| realised_quantity | int | Settled quantity |
| authorised_quantity | int | Quantity authorized for trading |
| t1_quantity | int | T+1 pending quantity |
| collateral_quantity | int | Quantity pledged as collateral |
| collateral_type | string | Type of collateral |
| discrepancy | bool | Discrepancy flag |
| average_price | float | Average buy price |
| last_price | float | Current market price |
| close_price | float | Previous day close |
| pnl | float | Total P&L (last_price - average_price) * quantity |
| day_change | float | Day's P&L change |
| day_change_percentage | float | Day's P&L change percentage |

**Important Notes**:
- Holdings contain long-term equity delivery stocks
- Instruments remain until sold or delisted
- Updated daily after market close
- Intraday positions NOT included (see Positions endpoint)

**Rate Limit**: 10 requests/second per API key

---

### 2. Get Positions

**Method**: `GET`

**Endpoint**: `/portfolio/positions`

**URL**: `https://api.kite.trade/portfolio/positions`

**Purpose**: Retrieve short to medium term positions (derivatives and intraday equity)

**Authentication**: Required

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "data": {
    "net": [
      {
        "tradingsymbol": "NIFTY26FEB20000CE",
        "exchange": "NFO",
        "instrument_token": 15199234,
        "product": "NRML",
        "quantity": 50,
        "overnight_quantity": 0,
        "multiplier": 1,
        "average_price": 120.00,
        "close_price": 115.00,
        "last_price": 122.50,
        "value": 6000.00,
        "pnl": 125.00,
        "m2m": 375.00,
        "unrealised": 375.00,
        "realised": 0,
        "buy_quantity": 50,
        "buy_price": 120.00,
        "buy_value": 6000.00,
        "buy_m2m": 375.00,
        "sell_quantity": 0,
        "sell_price": 0,
        "sell_value": 0,
        "sell_m2m": 0,
        "day_buy_quantity": 50,
        "day_buy_price": 120.00,
        "day_buy_value": 6000.00,
        "day_sell_quantity": 0,
        "day_sell_price": 0,
        "day_sell_value": 0
      }
    ],
    "day": [
      {
        "tradingsymbol": "INFY",
        "exchange": "NSE",
        "instrument_token": 408065,
        "product": "MIS",
        "quantity": 0,
        "overnight_quantity": 0,
        "multiplier": 1,
        "average_price": 0,
        "close_price": 1438.00,
        "last_price": 1450.50,
        "value": 0,
        "pnl": 0,
        "m2m": 0,
        "unrealised": 0,
        "realised": 125.00,
        "buy_quantity": 10,
        "buy_price": 1445.00,
        "buy_value": 14450.00,
        "buy_m2m": 0,
        "sell_quantity": 10,
        "sell_price": 1457.50,
        "sell_value": 14575.00,
        "sell_m2m": 0,
        "day_buy_quantity": 10,
        "day_buy_price": 1445.00,
        "day_buy_value": 14450.00,
        "day_sell_quantity": 10,
        "day_sell_price": 1457.50,
        "day_sell_value": 14575.00
      }
    ]
  }
}
```

**Response Structure**:
- **net**: Current net positions (actual portfolio state)
- **day**: Daily snapshot of buying/selling activity

**Position Fields**:

| Field | Type | Description |
|-------|------|-------------|
| tradingsymbol | string | Trading symbol |
| exchange | string | Exchange |
| instrument_token | int | Instrument token |
| product | string | CNC, NRML, MIS, MTF |
| quantity | int | Net quantity (buy - sell) |
| overnight_quantity | int | Quantity carried from previous day |
| multiplier | int | Lot size multiplier |
| average_price | float | Average entry price |
| close_price | float | Previous close |
| last_price | float | Current market price |
| value | float | Current position value |
| pnl | float | Total P&L |
| m2m | float | Mark-to-market P&L |
| unrealised | float | Unrealised P&L |
| realised | float | Realised P&L (from squared-off trades) |
| buy_quantity | int | Total buy quantity |
| buy_price | float | Average buy price |
| buy_value | float | Total buy value |
| buy_m2m | float | Buy side M2M |
| sell_quantity | int | Total sell quantity |
| sell_price | float | Average sell price |
| sell_value | float | Total sell value |
| sell_m2m | float | Sell side M2M |
| day_buy_quantity | int | Today's buy quantity |
| day_buy_price | float | Today's average buy price |
| day_buy_value | float | Today's buy value |
| day_sell_quantity | int | Today's sell quantity |
| day_sell_price | float | Today's average sell price |
| day_sell_value | float | Today's sell value |

**Important Notes**:
- Positions contain derivatives and intraday equity
- Instruments remain until sold or expiry
- MIS positions auto-squared off before market close
- Net positions show current state
- Day positions show daily activity

**Rate Limit**: 10 requests/second per API key

---

### 3. Convert Position

**Method**: `PUT`

**Endpoint**: `/portfolio/positions`

**URL**: `https://api.kite.trade/portfolio/positions`

**Purpose**: Convert margin product of an open position

**Authentication**: Required

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| exchange | string | Yes | Exchange |
| tradingsymbol | string | Yes | Trading symbol |
| transaction_type | string | Yes | BUY or SELL |
| position_type | string | Yes | "overnight" or "day" |
| quantity | int | Yes | Quantity to convert (absolute value) |
| old_product | string | Yes | Current product (CNC, NRML, MIS) |
| new_product | string | Yes | Target product (CNC, NRML, MIS) |

**Common Conversions**:
- MIS → CNC (convert intraday to delivery)
- MIS → NRML (convert intraday F&O to overnight)
- NRML → MIS (convert overnight F&O to intraday)
- CNC → MIS (convert delivery to intraday - sell only)

**Request Example**:
```http
PUT /portfolio/positions HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/x-www-form-urlencoded

exchange=NSE
&tradingsymbol=INFY
&transaction_type=BUY
&position_type=day
&quantity=10
&old_product=MIS
&new_product=CNC
```

**Response**:
```json
{
  "status": "success",
  "data": true
}
```

**Important Notes**:
- Must have sufficient margin for target product
- Can only convert open positions
- Conversion subject to exchange rules
- Some conversions restricted near market close

**Rate Limit**: 10 requests/second per API key

---

### 4. Get Holdings Auctions

**Method**: `GET`

**Endpoint**: `/portfolio/holdings/auctions`

**URL**: `https://api.kite.trade/portfolio/holdings/auctions`

**Purpose**: Retrieve list of holdings currently in auctions

**Authentication**: Required

**Parameters**: None

**Response**: Similar to holdings response with additional `auction_number` field

**Auction Fields**: Same as holdings + auction_number

**Important Notes**:
- Auctions occur when delivery obligations aren't met
- Holdings in auction cannot be traded normally
- Each auction instance has unique auction_number

**Rate Limit**: 10 requests/second per API key

---

### 5. Authorize Holdings

**Method**: `POST`

**Endpoint**: `/portfolio/holdings/authorise`

**URL**: `https://api.kite.trade/portfolio/holdings/authorise`

**Purpose**: Initiate CDSL authorization for selling holdings

**Authentication**: Required

**Note**: This is part of CDSL e-DIS (electronic Delivery Instruction Slip) process for selling holdings.

---

## Margin Calculator

### 1. Order Margins

**Method**: `POST`

**Endpoint**: `/margins/orders`

**URL**: `https://api.kite.trade/margins/orders`

**Purpose**: Calculate required margins for orders (considering existing positions)

**Authentication**: Required

**Content-Type**: `application/json`

**Query Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| mode | string | No | "compact" for simplified response |

**Request Body** (JSON array of orders):
```json
[
  {
    "exchange": "NSE",
    "tradingsymbol": "INFY",
    "transaction_type": "BUY",
    "variety": "regular",
    "product": "CNC",
    "order_type": "MARKET",
    "quantity": 10,
    "price": 0,
    "trigger_price": 0
  },
  {
    "exchange": "NFO",
    "tradingsymbol": "NIFTY26FEB20000CE",
    "transaction_type": "BUY",
    "variety": "regular",
    "product": "NRML",
    "order_type": "LIMIT",
    "quantity": 50,
    "price": 120.00,
    "trigger_price": 0
  }
]
```

**Response** (full mode):
```json
{
  "status": "success",
  "data": [
    {
      "type": "equity",
      "tradingsymbol": "INFY",
      "exchange": "NSE",
      "span": 0,
      "exposure": 0,
      "option_premium": 0,
      "additional": 0,
      "bo": 0,
      "cash": 14505.00,
      "pnl": {
        "realised": 0,
        "unrealised": 0
      },
      "leverage": 1,
      "total": 14505.00,
      "var": 0,
      "charges": {
        "transaction_tax": 14.51,
        "transaction_tax_type": "stt",
        "exchange_turnover_charge": 0.51,
        "sebi_turnover_charge": 0.14,
        "brokerage": 0,
        "stamp_duty": 1.45,
        "gst": {
          "igst": 0.09,
          "cgst": 0,
          "sgst": 0,
          "total": 0.09
        },
        "total": 16.70
      }
    },
    {
      "type": "derivatives",
      "tradingsymbol": "NIFTY26FEB20000CE",
      "exchange": "NFO",
      "span": 4500.00,
      "exposure": 1500.00,
      "option_premium": 6000.00,
      "additional": 0,
      "bo": 0,
      "cash": 0,
      "pnl": {
        "realised": 0,
        "unrealised": 0
      },
      "leverage": 5,
      "total": 12000.00,
      "var": 500.00,
      "charges": {
        "transaction_tax": 30.00,
        "transaction_tax_type": "stt",
        "exchange_turnover_charge": 2.40,
        "sebi_turnover_charge": 0.60,
        "brokerage": 20.00,
        "stamp_duty": 6.00,
        "gst": {
          "igst": 4.07,
          "cgst": 0,
          "sgst": 0,
          "total": 4.07
        },
        "total": 63.07
      }
    }
  ]
}
```

**Margin Fields**:

| Field | Type | Description |
|-------|------|-------------|
| type | string | "equity", "derivatives", "commodity" |
| tradingsymbol | string | Trading symbol |
| exchange | string | Exchange |
| span | float | SPAN margin |
| exposure | float | Exposure margin |
| option_premium | float | Option premium amount |
| additional | float | Additional margin |
| bo | float | Bracket order margin |
| cash | float | Cash required |
| pnl.realised | float | Realised P&L |
| pnl.unrealised | float | Unrealised P&L |
| leverage | float | Leverage multiplier |
| total | float | Total margin required |
| var | float | Value at Risk |
| charges | object | Breakdown of charges |

**Charges Breakdown**:

| Field | Type | Description |
|-------|------|-------------|
| transaction_tax | float | STT/CTT amount |
| transaction_tax_type | string | "stt" or "ctt" |
| exchange_turnover_charge | float | Exchange turnover charge |
| sebi_turnover_charge | float | SEBI turnover charge |
| brokerage | float | Brokerage charge |
| stamp_duty | float | Stamp duty |
| gst.igst | float | Integrated GST |
| gst.cgst | float | Central GST |
| gst.sgst | float | State GST |
| gst.total | float | Total GST |
| total | float | Total charges |

**Compact Mode Response**:
```json
{
  "status": "success",
  "data": [
    {
      "tradingsymbol": "INFY",
      "total": 14505.00
    },
    {
      "tradingsymbol": "NIFTY26FEB20000CE",
      "total": 12000.00
    }
  ]
}
```

**Rate Limit**: 10 requests/second per API key

---

### 2. Basket Margins

**Method**: `POST`

**Endpoint**: `/margins/basket`

**URL**: `https://api.kite.trade/margins/basket`

**Purpose**: Calculate margins for basket of orders (with spread benefits)

**Authentication**: Required

**Content-Type**: `application/json`

**Query Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| consider_positions | bool | No | Consider existing positions for margin benefit |
| mode | string | No | "compact" for simplified response |

**Request/Response**: Similar to Order Margins

**Key Difference**: Basket margins consider spread benefits (e.g., option strategies)

---

### 3. Virtual Contract Note

**Method**: `POST`

**Endpoint**: `/charges/orders`

**URL**: `https://api.kite.trade/charges/orders`

**Purpose**: Get detailed order-wise charge breakdowns

**Authentication**: Required

**Content-Type**: `application/json`

**Request/Response**: Similar to Order Margins, focused on charges

---

## Rate Limits Summary

| Endpoint Category | Rate Limit |
|------------------|------------|
| All account endpoints | 10 requests/second per API key |
| Margins calculation | 10 requests/second per API key |

---

## Best Practices

1. **Cache profile data**: User profile rarely changes, cache locally

2. **Monitor margins**: Check margins before placing orders to avoid rejections

3. **Position conversion timing**: Convert MIS to CNC before market close cutoff

4. **Holdings vs Positions**:
   - Holdings: Long-term delivery portfolio
   - Positions: Intraday and derivatives

5. **Margin calculation**: Always calculate margins before order placement for complex strategies

6. **Handle discrepancies**: Check `discrepancy` flag in holdings for auction risks

7. **Realised vs Unrealised P&L**:
   - Realised: Closed positions
   - Unrealised: Open positions M2M

8. **Daily margin refresh**: Margins update throughout the day, refresh before trading
