# Zerodha Kite Connect - Market Data Endpoints

## Overview

Kite Connect provides comprehensive market data access for all Indian exchanges (NSE, BSE, NFO, BFO, MCX, CDS, BCD) through REST endpoints and WebSocket streaming.

---

## REST Endpoints

### 1. Instruments List

**Endpoint**: `GET /instruments` or `GET /instruments/{exchange}`

**Purpose**: Returns gzipped CSV dump of all tradable instruments across exchanges

**Base URL**: `https://api.kite.trade/instruments`

**Exchange-Specific URLs**:
- All exchanges: `https://api.kite.trade/instruments`
- NSE only: `https://api.kite.trade/instruments/NSE`
- BSE only: `https://api.kite.trade/instruments/BSE`
- NFO only: `https://api.kite.trade/instruments/NFO`
- MCX only: `https://api.kite.trade/instruments/MCX`
- BFO only: `https://api.kite.trade/instruments/BFO`
- CDS only: `https://api.kite.trade/instruments/CDS`
- BCD only: `https://api.kite.trade/instruments/BCD`

**Parameters**:
- None (exchange is part of URL path if filtering)

**Response Format**: CSV (not JSON)

**Response Encoding**: Gzipped

**CSV Columns**:
```
instrument_token,exchange_token,tradingsymbol,name,last_price,expiry,strike,tick_size,lot_size,instrument_type,segment,exchange
```

**CSV Fields Description**:

| Field | Type | Description |
|-------|------|-------------|
| instrument_token | int | Unique identifier for the instrument |
| exchange_token | int | Exchange-specific token |
| tradingsymbol | string | Exchange trading symbol |
| name | string | Company/instrument name |
| last_price | float | Last recorded price (NOT real-time) |
| expiry | date | Expiry date (empty for equities) |
| strike | float | Strike price (options only) |
| tick_size | float | Minimum price movement |
| lot_size | int | Minimum trading quantity |
| instrument_type | string | EQ, FUT, CE, PE, etc. |
| segment | string | Exchange segment |
| exchange | string | NSE, BSE, NFO, etc. |

**Important Notes**:
- The dump is generated once daily (not real-time)
- `last_price` is snapshot from dump generation, not live
- For futures/options, `instrument_token` is flushed on expiry
- Must cache tokens for historical expired contracts

**Example CSV Rows**:
```csv
instrument_token,exchange_token,tradingsymbol,name,last_price,expiry,strike,tick_size,lot_size,instrument_type,segment,exchange
408065,1594,INFY,INFOSYS LIMITED,1450.50,,0.0,0.05,1,EQ,NSE,NSE
15199234,59372,NIFTY26FEB5020000CE,NIFTY,120.00,2026-02-26,20000.0,0.05,50,CE,NFO-OPT,NFO
```

**Rate Limit**: Standard rate limit (10 req/sec per API key)

**Authentication**: Required

---

### 2. Full Market Quotes

**Endpoint**: `GET /quote`

**Purpose**: Get full market quotes with OHLC, OI, bid/ask, and market depth

**URL**: `https://api.kite.trade/quote?i=EXCHANGE:TRADINGSYMBOL`

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| i | string (repeated) | Yes | Instrument in format `exchange:tradingsymbol` |

**Limits**:
- **Maximum instruments per request**: 500
- **Rate limit**: 1 request/second

**Query Format** (multiple instruments):
```
GET /quote?i=NSE:INFY&i=NSE:RELIANCE&i=NFO:NIFTY26FEB20000CE
```

**Response Format**: JSON

**Response Structure**:
```json
{
  "status": "success",
  "data": {
    "NSE:INFY": {
      "instrument_token": 408065,
      "timestamp": "2026-01-26T15:30:00+05:30",
      "last_trade_time": "2026-01-26T15:29:58+05:30",
      "last_price": 1450.50,
      "last_quantity": 10,
      "buy_quantity": 125430,
      "sell_quantity": 98750,
      "volume": 1234567,
      "average_price": 1448.75,
      "net_change": 12.50,
      "oi": 0,
      "oi_day_high": 0,
      "oi_day_low": 0,
      "lower_circuit_limit": 1305.45,
      "upper_circuit_limit": 1595.55,
      "ohlc": {
        "open": 1440.00,
        "high": 1455.00,
        "low": 1438.00,
        "close": 1438.00
      },
      "depth": {
        "buy": [
          {
            "price": 1450.45,
            "quantity": 250,
            "orders": 5
          },
          {
            "price": 1450.40,
            "quantity": 180,
            "orders": 3
          },
          {
            "price": 1450.35,
            "quantity": 320,
            "orders": 7
          },
          {
            "price": 1450.30,
            "quantity": 150,
            "orders": 2
          },
          {
            "price": 1450.25,
            "quantity": 200,
            "orders": 4
          }
        ],
        "sell": [
          {
            "price": 1450.50,
            "quantity": 300,
            "orders": 6
          },
          {
            "price": 1450.55,
            "quantity": 220,
            "orders": 4
          },
          {
            "price": 1450.60,
            "quantity": 180,
            "orders": 3
          },
          {
            "price": 1450.65,
            "quantity": 250,
            "orders": 5
          },
          {
            "price": 1450.70,
            "quantity": 190,
            "orders": 4
          }
        ]
      }
    }
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| instrument_token | int | Unique instrument identifier |
| timestamp | datetime | Quote timestamp |
| last_trade_time | datetime | Last trade execution time |
| last_price | float | Last traded price |
| last_quantity | int | Last trade quantity |
| buy_quantity | int | Total buy quantity at all bid levels |
| sell_quantity | int | Total sell quantity at all ask levels |
| volume | int | Total volume traded today |
| average_price | float | Volume-weighted average price |
| net_change | float | Change from previous close |
| oi | int | Open Interest (derivatives only) |
| oi_day_high | int | Day's high OI |
| oi_day_low | int | Day's low OI |
| lower_circuit_limit | float | Lower circuit limit |
| upper_circuit_limit | float | Upper circuit limit |
| ohlc.open | float | Day's opening price |
| ohlc.high | float | Day's highest price |
| ohlc.low | float | Day's lowest price |
| ohlc.close | float | Previous day's closing price |
| depth.buy | array | 5 levels of bid market depth |
| depth.sell | array | 5 levels of ask market depth |

**Market Depth Structure**:
- **5 levels** on each side (buy/sell)
- Each level contains: price, quantity, orders

**Important Notes**:
- If no data available for a key, the key will be absent from response
- OI fields are 0 for equities (only applicable to derivatives)
- Market depth provides Level 2 orderbook data

**Authentication**: Required

---

### 3. OHLC Quotes

**Endpoint**: `GET /quote/ohlc`

**Purpose**: Get OHLC and LTP data (lighter than full quote)

**URL**: `https://api.kite.trade/quote/ohlc?i=EXCHANGE:TRADINGSYMBOL`

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| i | string (repeated) | Yes | Instrument in format `exchange:tradingsymbol` |

**Limits**:
- **Maximum instruments per request**: 1000
- **Rate limit**: 1 request/second

**Response Structure**:
```json
{
  "status": "success",
  "data": {
    "NSE:INFY": {
      "instrument_token": 408065,
      "last_price": 1450.50,
      "ohlc": {
        "open": 1440.00,
        "high": 1455.00,
        "low": 1438.00,
        "close": 1438.00
      }
    },
    "NSE:RELIANCE": {
      "instrument_token": 738561,
      "last_price": 2650.75,
      "ohlc": {
        "open": 2640.00,
        "high": 2658.00,
        "low": 2635.00,
        "close": 2642.00
      }
    }
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| instrument_token | int | Unique instrument identifier |
| last_price | float | Last traded price |
| ohlc.open | float | Day's opening price |
| ohlc.high | float | Day's highest price |
| ohlc.low | float | Day's lowest price |
| ohlc.close | float | Previous day's closing price |

**Authentication**: Required

---

### 4. LTP (Last Traded Price) Quotes

**Endpoint**: `GET /quote/ltp`

**Purpose**: Get only the last traded price (lightest quote)

**URL**: `https://api.kite.trade/quote/ltp?i=EXCHANGE:TRADINGSYMBOL`

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| i | string (repeated) | Yes | Instrument in format `exchange:tradingsymbol` |

**Limits**:
- **Maximum instruments per request**: 1000
- **Rate limit**: 1 request/second

**Response Structure**:
```json
{
  "status": "success",
  "data": {
    "NSE:INFY": {
      "instrument_token": 408065,
      "last_price": 1450.50
    },
    "NSE:RELIANCE": {
      "instrument_token": 738561,
      "last_price": 2650.75
    },
    "NFO:NIFTY26FEB20000CE": {
      "instrument_token": 15199234,
      "last_price": 120.00
    }
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| instrument_token | int | Unique instrument identifier |
| last_price | float | Last traded price |

**Authentication**: Required

---

### 5. Historical Candle Data

**Endpoint**: `GET /instruments/historical/{instrument_token}/{interval}`

**Purpose**: Retrieve historical OHLC candle data

**URL Format**: `https://api.kite.trade/instruments/historical/{instrument_token}/{interval}`

**URI Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| instrument_token | int | Yes | Instrument identifier from instruments list |
| interval | string | Yes | Candle interval |

**Supported Intervals**:
- `minute` - 1-minute candles
- `3minute` - 3-minute candles
- `5minute` - 5-minute candles
- `10minute` - 10-minute candles
- `15minute` - 15-minute candles
- `30minute` - 30-minute candles
- `60minute` - 60-minute (1-hour) candles
- `day` - Daily candles

**Query Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| from | datetime | Yes | Start date/time in `yyyy-mm-dd HH:MM:SS` format |
| to | datetime | Yes | End date/time in `yyyy-mm-dd HH:MM:SS` format |
| continuous | int | No | Pass `1` for continuous data (NFO/MCX futures only) |
| oi | int | No | Pass `1` to include Open Interest data |

**Rate Limit**: 3 requests/second

**Example Request**:
```
GET /instruments/historical/408065/day?from=2025-01-01 00:00:00&to=2026-01-25 23:59:59
```

**Example Request with OI**:
```
GET /instruments/historical/15199234/15minute?from=2026-01-20 09:15:00&to=2026-01-26 15:30:00&oi=1
```

**Response Structure** (without OI):
```json
{
  "status": "success",
  "data": {
    "candles": [
      ["2026-01-20T09:15:00+0530", 1440.00, 1445.00, 1438.00, 1442.50, 123456],
      ["2026-01-20T09:30:00+0530", 1442.50, 1448.00, 1441.00, 1446.00, 145678],
      ["2026-01-20T09:45:00+0530", 1446.00, 1450.00, 1444.00, 1448.50, 167890]
    ]
  }
}
```

**Candle Array Structure** (without OI):
```
[timestamp, open, high, low, close, volume]
```

**Response Structure** (with OI):
```json
{
  "status": "success",
  "data": {
    "candles": [
      ["2026-01-20T09:15:00+0530", 120.00, 122.50, 119.00, 121.00, 5000, 125000],
      ["2026-01-20T09:30:00+0530", 121.00, 123.00, 120.50, 122.50, 6200, 127500],
      ["2026-01-20T09:45:00+0530", 122.50, 124.00, 121.00, 123.00, 5800, 130000]
    ]
  }
}
```

**Candle Array Structure** (with OI):
```
[timestamp, open, high, low, close, volume, oi]
```

**Field Descriptions**:

| Index | Field | Type | Description |
|-------|-------|------|-------------|
| 0 | timestamp | datetime | Candle timestamp (ISO 8601 format with timezone) |
| 1 | open | float | Opening price |
| 2 | high | float | Highest price |
| 3 | low | float | Lowest price |
| 4 | close | float | Closing price |
| 5 | volume | int | Volume traded |
| 6 | oi | int | Open Interest (if `oi=1` param used) |

**Important Notes**:
- Exchanges flush `instrument_token` for futures/options on expiry
- Must cache tokens for historical expired contracts
- `continuous=1` works ONLY with NFO and MCX futures contracts
- Continuous data provides seamless historical data across contract rollovers
- Can request specific granular time ranges (e.g., 15-minute windows)

**Authentication**: Required

---

## Endpoint Comparison

| Endpoint | Max Instruments | Rate Limit | Data Included | Use Case |
|----------|----------------|------------|---------------|----------|
| /quote | 500 | 1 req/sec | Full (OHLC, depth, OI) | Detailed quotes with orderbook |
| /quote/ohlc | 1000 | 1 req/sec | OHLC + LTP | Basic price data |
| /quote/ltp | 1000 | 1 req/sec | LTP only | Quick price checks |
| /instruments/historical | 1 | 3 req/sec | Historical candles | Backtesting, charting |
| /instruments | All | 10 req/sec | Instrument list | Symbol discovery |

---

## Instrument Identification

### Format
All market data endpoints use one of two identification methods:

1. **Exchange:TradingSymbol** (for REST quotes)
   - Format: `{EXCHANGE}:{TRADINGSYMBOL}`
   - Examples:
     - `NSE:INFY` - Infosys equity on NSE
     - `BSE:INFY` - Infosys equity on BSE
     - `NFO:NIFTY26FEB20000CE` - Nifty Call Option
     - `MCX:GOLDPETAL26FEBFUT` - Gold Petal Futures

2. **Instrument Token** (for historical data and WebSocket)
   - Format: Integer identifier
   - Example: `408065` (Infosys NSE)
   - Obtained from `/instruments` CSV dump

---

## Data Availability

### Equities (NSE/BSE)
- Real-time quotes: Yes
- Historical data: Yes (years of history)
- Market depth: Yes (5 levels)
- Open Interest: No (not applicable)

### Derivatives (NFO/BFO/CDS/BCD)
- Real-time quotes: Yes
- Historical data: Yes (limited to contract life + cached tokens)
- Market depth: Yes (5 levels)
- Open Interest: Yes

### Commodities (MCX)
- Real-time quotes: Yes
- Historical data: Yes (continuous data for futures)
- Market depth: Yes (5 levels)
- Open Interest: Yes

---

## Free Tier vs Paid Tier

| Feature | Free (Personal API) | Paid (Connect API - ₹500/mo) |
|---------|---------------------|------------------------------|
| Instruments list | ✅ Yes | ✅ Yes |
| Quote (LTP, OHLC, Full) | ✅ Yes | ✅ Yes |
| Historical candles | ❌ No | ✅ Yes |
| WebSocket streaming | ❌ No | ✅ Yes |
| Rate limits | Same | Same |

---

## Error Handling

### Missing Data
If there is no data available for a given instrument, the key will be **absent from the response** (not null).

```json
{
  "status": "success",
  "data": {
    "NSE:INFY": {
      "instrument_token": 408065,
      "last_price": 1450.50
    }
    // NSE:INVALID would be missing entirely if invalid
  }
}
```

### Invalid Instrument
Returns partial response with only valid instruments.

### Rate Limit Exceeded
```json
{
  "status": "error",
  "message": "Too many requests",
  "error_type": "NetworkException"
}
```
HTTP Status: 429

---

## Best Practices

1. **Use appropriate endpoint for use case**:
   - Need orderbook? → `/quote`
   - Need OHLC only? → `/quote/ohlc`
   - Need just price? → `/quote/ltp`

2. **Batch requests**:
   - Request multiple instruments in single call
   - Respect max instrument limits (500 or 1000)

3. **Cache instrument list**:
   - Download `/instruments` dump daily
   - Store locally to avoid repeated calls
   - Update tokens before expiry for derivatives

4. **Use WebSocket for real-time**:
   - REST polling is rate-limited
   - WebSocket provides push-based updates
   - More efficient for live data

5. **Handle missing data gracefully**:
   - Check for key existence before accessing
   - Don't assume all fields always present

6. **Respect rate limits**:
   - Implement exponential backoff on 429
   - Track request counts client-side
   - Use WebSocket to reduce REST calls
