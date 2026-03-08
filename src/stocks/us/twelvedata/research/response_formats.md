# Twelvedata - Response Formats

All examples are from official documentation or inferred from documented field descriptions.

## Standard Error Format

### Error Response (All Endpoints)
```json
{
  "code": 400,
  "message": "Invalid parameters: symbol is required",
  "status": "error"
}
```

**Common Error Codes:**
- 400: Bad Request (invalid parameters)
- 401: Unauthorized (invalid API key)
- 403: Forbidden (insufficient tier)
- 404: Not Found (symbol/data unavailable)
- 414: URI Too Long (too many symbols)
- 429: Rate Limit Exceeded
- 500: Internal Server Error

## Core Market Data Endpoints

### GET /price
**Request:**
```
GET https://api.twelvedata.com/price?symbol=AAPL&apikey=demo
```

**Response:**
```json
{
  "symbol": "AAPL",
  "price": 150.25
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Ticker symbol |
| price | float | Current/latest price |

---

### GET /quote
**Request:**
```
GET https://api.twelvedata.com/quote?symbol=AAPL&apikey=demo
```

**Response:**
```json
{
  "symbol": "AAPL",
  "name": "Apple Inc",
  "exchange": "NASDAQ",
  "mic_code": "XNGS",
  "currency": "USD",
  "datetime": "2024-01-26",
  "timestamp": 1706284800,
  "open": 149.50,
  "high": 151.20,
  "low": 148.80,
  "close": 150.25,
  "volume": 65432100,
  "previous_close": 148.75,
  "change": 1.50,
  "percent_change": 1.01,
  "average_volume": 70123456,
  "is_market_open": true,
  "fifty_two_week": {
    "low": 125.50,
    "high": 175.80,
    "low_change": 24.75,
    "high_change": -25.55,
    "low_change_percent": 19.72,
    "high_change_percent": -14.53,
    "range": "125.500000 - 175.800000"
  }
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Ticker symbol |
| name | string | Company/instrument name |
| exchange | string | Exchange name |
| mic_code | string | Market Identifier Code |
| currency | string | Trading currency (e.g., USD) |
| datetime | string | ISO 8601 date (YYYY-MM-DD) |
| timestamp | integer | Unix timestamp (seconds) |
| open | float | Opening price |
| high | float | Highest price |
| low | float | Lowest price |
| close | float | Closing/latest price |
| volume | integer | Trading volume |
| previous_close | float | Previous day's close |
| change | float | Absolute price change |
| percent_change | float | Percentage change |
| average_volume | integer | Average volume |
| is_market_open | boolean | Market open status |
| fifty_two_week | object | 52-week statistics |

**Note:** Some fields may be null if data unavailable.

---

### GET /time_series
**Request:**
```
GET https://api.twelvedata.com/time_series?symbol=AAPL&interval=1day&outputsize=5&apikey=demo
```

**Response:**
```json
{
  "meta": {
    "symbol": "AAPL",
    "interval": "1day",
    "currency": "USD",
    "exchange_timezone": "America/New_York",
    "exchange": "NASDAQ",
    "mic_code": "XNGS",
    "type": "Common Stock"
  },
  "values": [
    {
      "datetime": "2024-01-26",
      "open": "149.50000",
      "high": "151.20000",
      "low": "148.80000",
      "close": "150.25000",
      "volume": "65432100"
    },
    {
      "datetime": "2024-01-25",
      "open": "148.20000",
      "high": "149.80000",
      "low": "147.50000",
      "close": "148.75000",
      "volume": "72543210"
    }
  ],
  "status": "ok"
}
```

**Meta Fields:**
| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Ticker symbol |
| interval | string | Time interval (1min, 1day, etc.) |
| currency | string | Trading currency |
| exchange_timezone | string | IANA timezone |
| exchange | string | Exchange name |
| mic_code | string | Market Identifier Code |
| type | string | Instrument type |

**Values Fields (array of OHLCV bars):**
| Field | Type | Description |
|-------|------|-------------|
| datetime | string | ISO 8601 timestamp |
| open | string | Opening price (as string) |
| high | string | Highest price (as string) |
| low | string | Lowest price (as string) |
| close | string | Closing price (as string) |
| volume | string | Trading volume (as string) |

**Note:** Numeric values are returned as **strings** in time_series to preserve precision.

---

### GET /eod
**Request:**
```
GET https://api.twelvedata.com/eod?symbol=AAPL&apikey=demo
```

**Response:**
```json
{
  "symbol": "AAPL",
  "exchange": "NASDAQ",
  "mic_code": "XNGS",
  "currency": "USD",
  "datetime": "2024-01-26",
  "close": "150.25"
}
```

---

### GET /exchange_rate
**Request:**
```
GET https://api.twelvedata.com/exchange_rate?symbol=EUR/USD&apikey=demo
```

**Response:**
```json
{
  "symbol": "EUR/USD",
  "rate": 1.08450,
  "timestamp": 1706284800
}
```

---

## Reference Data Endpoints

### GET /stocks
**Request:**
```
GET https://api.twelvedata.com/stocks?apikey=demo
```

**Response:**
```json
{
  "data": [
    {
      "symbol": "AAPL",
      "name": "Apple Inc",
      "currency": "USD",
      "exchange": "NASDAQ",
      "mic_code": "XNGS",
      "country": "United States",
      "type": "Common Stock"
    },
    {
      "symbol": "MSFT",
      "name": "Microsoft Corporation",
      "currency": "USD",
      "exchange": "NASDAQ",
      "mic_code": "XNGS",
      "country": "United States",
      "type": "Common Stock"
    }
  ],
  "status": "ok"
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Ticker symbol |
| name | string | Company name |
| currency | string | Trading currency |
| exchange | string | Exchange name |
| mic_code | string | Market Identifier Code |
| country | string | Country name |
| type | string | Instrument type |

---

### GET /forex_pairs
**Response:**
```json
{
  "data": [
    {
      "symbol": "EUR/USD",
      "currency_group": "Major",
      "currency_base": "Euro",
      "currency_quote": "US Dollar"
    },
    {
      "symbol": "GBP/USD",
      "currency_group": "Major",
      "currency_base": "British Pound",
      "currency_quote": "US Dollar"
    }
  ],
  "status": "ok"
}
```

---

### GET /cryptocurrencies
**Response:**
```json
{
  "data": [
    {
      "symbol": "BTC/USD",
      "available_exchanges": ["Binance", "Coinbase", "Kraken", "Bitfinex"],
      "currency_base": "Bitcoin",
      "currency_quote": "US Dollar"
    },
    {
      "symbol": "ETH/USD",
      "available_exchanges": ["Binance", "Coinbase", "Kraken"],
      "currency_base": "Ethereum",
      "currency_quote": "US Dollar"
    }
  ],
  "status": "ok"
}
```

---

### GET /symbol_search
**Request:**
```
GET https://api.twelvedata.com/symbol_search?symbol=apple&apikey=demo
```

**Response:**
```json
{
  "data": [
    {
      "symbol": "AAPL",
      "instrument_name": "Apple Inc",
      "exchange": "NASDAQ",
      "mic_code": "XNGS",
      "exchange_timezone": "America/New_York",
      "instrument_type": "Common Stock",
      "country": "United States",
      "currency": "USD"
    }
  ],
  "status": "ok"
}
```

---

### GET /exchanges
**Response:**
```json
{
  "data": [
    {
      "name": "NASDAQ",
      "code": "NASDAQ",
      "country": "United States",
      "timezone": "America/New_York"
    },
    {
      "name": "New York Stock Exchange",
      "code": "NYSE",
      "country": "United States",
      "timezone": "America/New_York"
    }
  ],
  "status": "ok"
}
```

---

### GET /market_state
**Request:**
```
GET https://api.twelvedata.com/market_state?exchange=NASDAQ&apikey=demo
```

**Response:**
```json
{
  "name": "NASDAQ",
  "code": "NASDAQ",
  "country": "United States",
  "is_market_open": true,
  "time_after_open": "02:15:30",
  "time_to_open": "00:00:00",
  "time_to_close": "04:29:30"
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| name | string | Exchange name |
| code | string | Exchange code |
| country | string | Country |
| is_market_open | boolean | Market open status |
| time_after_open | string | Time since open (HH:MM:SS) |
| time_to_open | string | Time until open (HH:MM:SS) |
| time_to_close | string | Time until close (HH:MM:SS) |

---

## Technical Indicators

### GET /rsi
**Request:**
```
GET https://api.twelvedata.com/rsi?symbol=AAPL&interval=1day&time_period=14&apikey=demo
```

**Response:**
```json
{
  "meta": {
    "symbol": "AAPL",
    "interval": "1day",
    "currency": "USD",
    "exchange_timezone": "America/New_York",
    "exchange": "NASDAQ",
    "mic_code": "XNGS",
    "type": "Common Stock",
    "indicator": {
      "name": "RSI - Relative Strength Index",
      "time_period": 14,
      "series_type": "close"
    }
  },
  "values": [
    {
      "datetime": "2024-01-26",
      "rsi": "65.432"
    },
    {
      "datetime": "2024-01-25",
      "rsi": "63.124"
    }
  ],
  "status": "ok"
}
```

---

### GET /macd
**Response:**
```json
{
  "meta": {
    "symbol": "BTC/USD",
    "interval": "30min",
    "currency_base": "Bitcoin",
    "currency_quote": "US Dollar",
    "exchange_timezone": "UTC",
    "exchange": "Binance",
    "type": "Digital Currency",
    "indicator": {
      "name": "MACD - Moving Average Convergence/Divergence",
      "fast_period": 12,
      "slow_period": 26,
      "signal_period": 9,
      "series_type": "close"
    }
  },
  "values": [
    {
      "datetime": "2024-01-26 14:30:00",
      "macd": "245.67800",
      "macd_signal": "230.12300",
      "macd_hist": "15.55500"
    }
  ],
  "status": "ok"
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| macd | string | MACD line value |
| macd_signal | string | Signal line value |
| macd_hist | string | Histogram (MACD - Signal) |

---

### GET /bbands
**Response:**
```json
{
  "meta": {
    "symbol": "AAPL",
    "interval": "1min",
    "indicator": {
      "name": "BBANDS - Bollinger Bands",
      "time_period": 20,
      "series_type": "close",
      "sd": 2,
      "ma_type": "SMA"
    }
  },
  "values": [
    {
      "datetime": "2024-01-26 14:30:00",
      "upper_band": "152.45000",
      "middle_band": "150.25000",
      "lower_band": "148.05000"
    }
  ],
  "status": "ok"
}
```

---

## Fundamental Data Endpoints

### GET /profile
**Response:**
```json
{
  "symbol": "AAPL",
  "name": "Apple Inc",
  "exchange": "NASDAQ",
  "currency": "USD",
  "country": "United States",
  "sector": "Technology",
  "industry": "Consumer Electronics",
  "address": "One Apple Park Way, Cupertino, CA 95014",
  "website": "https://www.apple.com",
  "description": "Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide.",
  "ceo": "Timothy D. Cook",
  "employees": 164000,
  "phone": "14089961010",
  "logo": "https://api.twelvedata.com/logo/aapl.com"
}
```

---

### GET /earnings
**Response:**
```json
{
  "symbol": "AAPL",
  "earnings": [
    {
      "date": "2024-01-31",
      "period": "Q1 2024",
      "eps_estimate": 2.10,
      "eps_actual": 2.18,
      "revenue_estimate": 118500000000,
      "revenue_actual": 119575000000,
      "surprise": 0.08,
      "surprise_percent": 3.81
    },
    {
      "date": "2023-10-31",
      "period": "Q4 2023",
      "eps_estimate": 1.39,
      "eps_actual": 1.46,
      "revenue_estimate": 89280000000,
      "revenue_actual": 89498000000,
      "surprise": 0.07,
      "surprise_percent": 5.04
    }
  ],
  "status": "ok"
}
```

---

### GET /dividends
**Response:**
```json
{
  "symbol": "AAPL",
  "dividends": [
    {
      "ex_date": "2024-02-09",
      "payment_date": "2024-02-16",
      "declaration_date": "2024-02-01",
      "amount": 0.24,
      "currency": "USD"
    },
    {
      "ex_date": "2023-11-10",
      "payment_date": "2023-11-16",
      "declaration_date": "2023-11-02",
      "amount": 0.24,
      "currency": "USD"
    }
  ],
  "status": "ok"
}
```

---

### GET /income_statement
**Response:**
```json
{
  "symbol": "AAPL",
  "income_statement": [
    {
      "fiscal_date": "2023-09-30",
      "period": "Annual",
      "revenue": 383285000000,
      "cost_of_revenue": 214137000000,
      "gross_profit": 169148000000,
      "operating_expenses": 54768000000,
      "operating_income": 114380000000,
      "net_income": 96995000000,
      "eps": 6.13,
      "eps_diluted": 6.12
    }
  ],
  "status": "ok"
}
```

---

### GET /balance_sheet
**Response:**
```json
{
  "symbol": "AAPL",
  "balance_sheet": [
    {
      "fiscal_date": "2023-09-30",
      "period": "Annual",
      "total_assets": 352755000000,
      "total_current_assets": 143566000000,
      "cash": 29965000000,
      "total_liabilities": 290437000000,
      "total_current_liabilities": 145308000000,
      "total_shareholders_equity": 62318000000,
      "retained_earnings": -214000000
    }
  ],
  "status": "ok"
}
```

---

### GET /cash_flow
**Response:**
```json
{
  "symbol": "AAPL",
  "cash_flow": [
    {
      "fiscal_date": "2023-09-30",
      "period": "Annual",
      "operating_cash_flow": 110543000000,
      "capital_expenditures": -10959000000,
      "free_cash_flow": 99584000000,
      "investing_cash_flow": -11130000000,
      "financing_cash_flow": -108488000000,
      "net_change_in_cash": -3513000000
    }
  ],
  "status": "ok"
}
```

---

### GET /statistics
**Response:**
```json
{
  "symbol": "AAPL",
  "valuation_metrics": {
    "market_capitalization": 2350000000000,
    "enterprise_value": 2450000000000,
    "trailing_pe": 24.25,
    "forward_pe": 22.50,
    "peg_ratio": 2.10,
    "price_to_sales": 6.15,
    "price_to_book": 37.75,
    "ev_to_revenue": 6.40,
    "ev_to_ebitda": 18.50
  },
  "financial_metrics": {
    "return_on_assets": 0.275,
    "return_on_equity": 1.560,
    "profit_margin": 0.253,
    "operating_margin": 0.298,
    "current_ratio": 0.99,
    "quick_ratio": 0.81,
    "debt_to_equity": 1.89
  }
}
```

---

### GET /recommendations
**Response:**
```json
{
  "symbol": "AAPL",
  "recommendations": [
    {
      "date": "2024-01-26",
      "strong_buy": 15,
      "buy": 20,
      "hold": 8,
      "sell": 1,
      "strong_sell": 0,
      "consensus": "Buy",
      "consensus_rating": 1.8
    }
  ],
  "status": "ok"
}
```

---

### GET /price_target
**Response:**
```json
{
  "symbol": "AAPL",
  "price_targets": [
    {
      "date": "2024-01-26",
      "average_target": 195.50,
      "high_target": 220.00,
      "low_target": 150.00,
      "number_of_analysts": 44
    }
  ],
  "status": "ok"
}
```

---

## ETF Data Endpoints

### GET /etf-composition
**Response:**
```json
{
  "symbol": "SPY",
  "name": "SPDR S&P 500 ETF Trust",
  "top_holdings": [
    {
      "symbol": "AAPL",
      "name": "Apple Inc",
      "weight": 7.15
    },
    {
      "symbol": "MSFT",
      "name": "Microsoft Corporation",
      "weight": 6.85
    },
    {
      "symbol": "AMZN",
      "name": "Amazon.com Inc",
      "weight": 3.45
    }
  ],
  "sector_allocation": [
    {
      "sector": "Technology",
      "weight": 28.50
    },
    {
      "sector": "Healthcare",
      "weight": 13.20
    },
    {
      "sector": "Financials",
      "weight": 12.80
    }
  ],
  "status": "ok"
}
```

---

## Batch Requests

### GET /batch-requests (Multiple Symbols)
**Request:**
```
GET https://api.twelvedata.com/time_series?symbol=AAPL,MSFT,TSLA&interval=1day&outputsize=2&apikey=demo
```

**Response:**
```json
{
  "AAPL": {
    "meta": {
      "symbol": "AAPL",
      "interval": "1day",
      "currency": "USD",
      "exchange": "NASDAQ",
      "type": "Common Stock"
    },
    "values": [
      {
        "datetime": "2024-01-26",
        "open": "149.50",
        "high": "151.20",
        "low": "148.80",
        "close": "150.25",
        "volume": "65432100"
      }
    ],
    "status": "ok"
  },
  "MSFT": {
    "meta": {
      "symbol": "MSFT",
      "interval": "1day",
      "currency": "USD",
      "exchange": "NASDAQ",
      "type": "Common Stock"
    },
    "values": [
      {
        "datetime": "2024-01-26",
        "open": "395.50",
        "high": "398.20",
        "low": "394.80",
        "close": "397.15",
        "volume": "25123456"
      }
    ],
    "status": "ok"
  },
  "TSLA": {
    "meta": { "..." },
    "values": [ "..." ],
    "status": "ok"
  }
}
```

**Note:** Each symbol is a top-level key in the response object.

---

## WebSocket Message Formats

### Subscribe Message (Client → Server)
```json
{
  "action": "subscribe",
  "params": {
    "symbols": "AAPL,TRP,QQQ,EUR/USD,USD/JPY,BTC/USD,ETH/BTC"
  }
}
```

---

### Price Event (Server → Client)
```json
{
  "event": "price",
  "symbol": "BTC/USD",
  "currency_base": "Bitcoin",
  "currency_quote": "US Dollar",
  "exchange": "Binance",
  "type": "Digital Currency",
  "timestamp": 1600595462,
  "price": 10964.8,
  "day_volume": 38279
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| event | string | Event type ("price") |
| symbol | string | Ticker symbol |
| currency_base | string | Base currency name |
| currency_quote | string | Quote currency name |
| exchange | string | Exchange name |
| type | string | Instrument type |
| timestamp | integer | Unix timestamp (seconds) |
| price | float | Current price |
| day_volume | integer/null | Daily volume (may be null) |

---

### Heartbeat Message (Client → Server)
**Inferred format (not explicitly documented):**
```json
{
  "action": "heartbeat"
}
```

**Frequency:** Every 10 seconds recommended.

---

### Unsubscribe Message (Client → Server)
**Inferred format:**
```json
{
  "action": "unsubscribe",
  "params": {
    "symbols": "AAPL,BTC/USD"
  }
}
```

---

### Reset Message (Client → Server)
**Inferred format:**
```json
{
  "action": "reset"
}
```

---

## CSV Format

All endpoints support CSV output via `?format=csv` parameter.

**Example:**
```
GET https://api.twelvedata.com/time_series?symbol=AAPL&interval=1day&outputsize=3&format=csv&apikey=demo
```

**Response:**
```csv
datetime;open;high;low;close;volume
2024-01-26;149.50000;151.20000;148.80000;150.25000;65432100
2024-01-25;148.20000;149.80000;147.50000;148.75000;72543210
2024-01-24;147.00000;148.50000;146.20000;148.20000;68754321
```

**Default delimiter:** Semicolon (;)
**Configurable:** Yes (via SDK or parameter)

---

## Null Value Handling

**Important:** Many fields may return `null` when data is unavailable.

**Example:**
```json
{
  "symbol": "SMALLCAP",
  "day_volume": null,
  "fifty_two_week": {
    "low": 10.50,
    "high": null
  }
}
```

**Best practice:** Always check for null before using values in calculations.

```rust
// Rust example
let volume = quote.day_volume.unwrap_or(0);
```

---

## Status Field

All responses include a `status` field:
- `"ok"`: Request successful
- `"error"`: Request failed (error details in `code` and `message` fields)

**Success:**
```json
{
  "data": [...],
  "status": "ok"
}
```

**Error:**
```json
{
  "code": 404,
  "message": "Symbol not found",
  "status": "error"
}
```

---

## Numeric Precision

Numeric values in **time_series** are returned as **strings** to preserve precision:

```json
{
  "datetime": "2024-01-26",
  "open": "149.50000",
  "close": "150.25000"
}
```

**In other endpoints**, numerics are returned as **floats/integers**:

```json
{
  "price": 150.25,
  "volume": 65432100
}
```

**Best practice:** Parse string numerics to appropriate type (Decimal/Float) based on precision requirements.

---

## Timezone Handling

Timestamps can be in three formats:
1. **Exchange local time** (default)
2. **UTC**
3. **Custom IANA timezone**

**Example with timezone parameter:**
```
GET /time_series?symbol=AAPL&interval=1h&timezone=UTC&apikey=demo
```

**Response datetime field adjusts accordingly:**
```json
{
  "meta": {
    "exchange_timezone": "UTC"
  },
  "values": [
    {
      "datetime": "2024-01-26 14:30:00"
    }
  ]
}
```

---

## Important Notes

1. **String vs Numeric**: time_series returns numeric values as strings; other endpoints use floats/integers
2. **Null values**: Always possible - implement defensive checks
3. **Status field**: Always present - check for "ok" vs "error"
4. **Batch responses**: Symbol names as top-level keys
5. **CSV format**: Semicolon delimiter by default
6. **Timestamps**: Unix seconds (not milliseconds) in most cases
7. **Decimal precision**: Configurable via `dp` parameter (0-11)
8. **Array wrapping**: Reference data endpoints return `{data: [...], status: "ok"}`
9. **Meta object**: Included in time series and indicator responses
10. **WebSocket timestamps**: Unix seconds, not milliseconds
