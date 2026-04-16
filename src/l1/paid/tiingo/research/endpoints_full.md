# Tiingo - Complete Endpoint Reference

## Category: End-of-Day (EOD) Stock Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /tiingo/daily/{ticker} | Ticker metadata | Yes | Yes | 5/min, 500/day | Returns name, exchange, description, dates |
| GET | /tiingo/daily/{ticker}/prices | Historical daily prices | Yes | Yes | 5/min, 500/day | Supports date range, resampling |
| GET | /tiingo/daily/supported_tickers.zip | List of supported tickers | Yes | Yes | 5/min, 500/day | Bulk download, hosted on apimedia |

### GET /tiingo/daily/{ticker}
**Purpose:** Get metadata about a ticker symbol

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | Stock ticker symbol (e.g., AAPL) |
| token | string | No* | - | API token (can use header instead) |

**Example:**
```
GET https://api.tiingo.com/tiingo/daily/AAPL?token=YOUR_TOKEN
```

### GET /tiingo/daily/{ticker}/prices
**Purpose:** Get historical daily OHLC prices with adjustments

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | Stock ticker symbol |
| startDate | date | No | - | Start date (YYYY-MM-DD) |
| endDate | date | No | - | End date (YYYY-MM-DD) |
| resampleFreq | string | No | daily | Resample frequency: daily, weekly, monthly, annually |
| columns | string | No | all | Comma-separated list of columns to return |
| token | string | No* | - | API token (can use header instead) |
| format | string | No | json | Response format: json or csv |

**Example:**
```
GET https://api.tiingo.com/tiingo/daily/AAPL/prices?startDate=2020-01-01&endDate=2020-12-31&resampleFreq=daily&token=YOUR_TOKEN
```

**Columns Available:**
- date, open, high, low, close, volume
- adjOpen, adjHigh, adjLow, adjClose, adjVolume
- divCash, splitFactor

---

## Category: IEX Intraday Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /iex/{ticker} | IEX ticker metadata | Yes | Yes | 5/min, 500/day | IEX-specific metadata |
| GET | /iex/{ticker}/prices | Intraday IEX prices | Yes | Yes | 5/min, 500/day | Real-time and historical intraday |

### GET /iex/{ticker}/prices
**Purpose:** Get intraday price data from IEX exchange

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | Stock ticker symbol |
| startDate | datetime | No | - | Start datetime (YYYY-MM-DD or ISO8601) |
| endDate | datetime | No | - | End datetime |
| resampleFreq | string | No | - | Resample to intervals: 1min, 5min, 15min, 30min, 1hour, 4hour |
| afterHours | boolean | No | false | Include after-hours data |
| forceFill | boolean | No | true | Forward-fill missing bars |
| columns | string | No | all | Columns to return |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/iex/AAPL/prices?startDate=2020-01-01&resampleFreq=5min&token=YOUR_TOKEN
```

---

## Category: Cryptocurrency Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /tiingo/crypto | Crypto metadata | Yes | Yes | 5/min, 500/day | List supported crypto tickers |
| GET | /tiingo/crypto/top | Top-of-book quotes | Yes | Yes | 5/min, 500/day | Current bid/ask for crypto pairs |
| GET | /tiingo/crypto/prices | Historical crypto prices | Yes | Yes | 5/min, 500/day | OHLCV data, multi-exchange |

### GET /tiingo/crypto
**Purpose:** Get metadata for cryptocurrency tickers

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| tickers | array[string] | No | all | Filter by specific tickers (e.g., btcusd) |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/crypto?token=YOUR_TOKEN
```

### GET /tiingo/crypto/top
**Purpose:** Get current top-of-book (best bid/ask) for crypto pairs

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| tickers | array[string] | Yes | - | Crypto pairs (e.g., btcusd, ethusd) - as list |
| exchanges | array[string] | No | all | Filter by exchanges |
| convertCurrency | string | No | - | Convert prices to currency (e.g., usd) |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/crypto/top?tickers=btcusd,ethusd&token=YOUR_TOKEN
```

### GET /tiingo/crypto/prices
**Purpose:** Get historical cryptocurrency OHLCV data

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| tickers | array[string] | Yes | - | Crypto pairs (must be list/array) |
| baseCurrency | string | No | - | Base currency filter |
| startDate | datetime | No | - | Start datetime |
| endDate | datetime | No | - | End datetime |
| resampleFreq | string | No | - | Resample: 1min, 5min, 15min, 1hour, 1day, etc. |
| exchanges | array[string] | No | all | Filter by exchanges |
| convertCurrency | string | No | - | Convert prices to currency |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/crypto/prices?tickers=btcusd&startDate=2020-01-01&resampleFreq=1hour&token=YOUR_TOKEN
```

---

## Category: Forex (FX) Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /tiingo/fx/{ticker}/top | Top-of-book FX quote | Yes | Yes | 5/min, 500/day | Current bid/ask for currency pair |
| GET | /tiingo/fx/{ticker}/prices | Historical FX prices | Yes | Yes | 5/min, 500/day | OHLC data, intraday or daily |

### GET /tiingo/fx/{ticker}/top
**Purpose:** Get current top-of-book quote for forex pair

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | FX pair ticker (e.g., eurusd) |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/fx/eurusd/top?token=YOUR_TOKEN
```

### GET /tiingo/fx/{ticker}/prices
**Purpose:** Get historical forex OHLC prices

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | FX pair ticker |
| startDate | datetime | No | - | Start datetime |
| endDate | datetime | No | - | End datetime |
| resampleFreq | string | No | 1day | Resample: 1min, 5min, 15min, 30min, 1hour, 4hour, 1day |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/fx/eurusd/prices?startDate=2019-06-30&resampleFreq=5min&token=YOUR_TOKEN
```

---

## Category: Fundamental Data (Stocks)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /tiingo/fundamentals/definitions | Field definitions | Yes | Yes | 5/min, 500/day | Metadata about fundamentals fields |
| GET | /tiingo/fundamentals/{ticker}/daily | Daily-updated metrics | Yes | Yes | 5/min, 500/day | marketCap, EV, P/E, etc. (5yr free, 15yr paid) |
| GET | /tiingo/fundamentals/{ticker}/statements | Financial statements | Yes | Yes | 5/min, 500/day | Quarterly/annual SEC filings (5yr free, 15yr paid) |

### GET /tiingo/fundamentals/definitions
**Purpose:** Get definitions for all available fundamentals fields

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/fundamentals/definitions?token=YOUR_TOKEN
```

### GET /tiingo/fundamentals/{ticker}/daily
**Purpose:** Get daily-updated fundamental metrics

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | Stock ticker symbol |
| startDate | date | No | - | Start date |
| endDate | date | No | - | End date |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/fundamentals/AAPL/daily?startDate=2020-01-01&token=YOUR_TOKEN
```

**Metrics Available:**
- Market capitalization
- Enterprise value
- P/E ratio, P/B ratio, P/S ratio
- Dividend yield
- ROE, ROA, ROIC
- Debt/equity ratio
- And 80+ other indicators

### GET /tiingo/fundamentals/{ticker}/statements
**Purpose:** Get quarterly and annual financial statements

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ticker | string | Yes | - | Stock ticker symbol |
| startDate | date | No | - | Start date |
| endDate | date | No | - | End date |
| asReported | boolean | No | false | Return as-reported vs standardized |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/fundamentals/AAPL/statements?asReported=false&token=YOUR_TOKEN
```

**Statements Available:**
- Income statement
- Balance sheet
- Cash flow statement
- Quarterly and annual periods
- As-reported and standardized formats

---

## Category: News Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /tiingo/news | Financial news articles | Yes | Yes | 5/min, 500/day | Curated financial news feed |
| GET | /tiingo/news/bulk_download | List bulk news files | Yes | Yes | 5/min, 500/day | Available bulk downloads |
| GET | /tiingo/news/bulk_download/{file_id} | Download bulk news file | Yes | Yes | 5/min, 500/day | Download specific bulk file |

### GET /tiingo/news
**Purpose:** Get curated financial news articles

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| tickers | array[string] | No | - | Filter by tickers (e.g., AAPL, GOOGL) |
| tags | array[string] | No | - | Search by keywords/tags |
| sources | array[string] | No | - | Filter by source domains |
| startDate | datetime | No | - | Start datetime |
| endDate | datetime | No | - | End datetime |
| limit | integer | No | 100 | Max results to return |
| offset | integer | No | 0 | Offset for pagination |
| sortBy | string | No | publishedDate | Sort field |
| token | string | No* | - | API token |

**Example:**
```
GET https://api.tiingo.com/tiingo/news?tickers=AAPL,GOOGL&tags=Laptops&sources=washingtonpost.com&startDate=2017-01-01&endDate=2017-08-31&token=YOUR_TOKEN
```

---

## Category: Metadata & Lists

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /tiingo/daily/supported_tickers.zip | Supported stock tickers | Yes | Yes | 5/min, 500/day | Bulk download from apimedia |
| GET | /tiingo/crypto (metadata) | Supported crypto tickers | Yes | Yes | 5/min, 500/day | List all crypto pairs |

---

## Authentication Method

All endpoints support authentication via:

1. **Authorization Header** (Recommended):
```
Authorization: Token YOUR_API_KEY
```

2. **Query Parameter**:
```
?token=YOUR_API_KEY
```

**Note:** All endpoints marked with "Auth? Yes" require API key authentication.

---

## Response Formats

All endpoints support:
- **JSON** (default): Set `format=json` or omit parameter
- **CSV**: Set `format=csv` for optimized bulk downloads

---

## Rate Limit Headers

Response headers include:
```
X-RateLimit-Limit: 5 (requests per minute)
X-RateLimit-Remaining: 3
X-RateLimit-Reset: 1234567890 (Unix timestamp)
```

HTTP 429 returned when rate limit exceeded with `Retry-After` header.

---

## Coverage Summary

- **Stocks**: 32,000+ US equities, Chinese stocks
- **ETFs/Mutual Funds**: 33,000+
- **Crypto**: 2,100+ - 4,100+ tickers from 40+ exchanges
- **Forex**: 140+ currency pairs from tier-1 banks
- **Fundamentals**: 5,500+ equities, 80+ indicators
- **Historical Depth**: 50+ years for stocks, varies by asset class
