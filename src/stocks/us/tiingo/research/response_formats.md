# Tiingo - Response Formats

All examples below are taken from official documentation and SDK source code. Field names and structures are exact.

---

## Response Format Options

All REST endpoints support:
- **JSON** (default): `format=json` or omit parameter
- **CSV**: `format=csv` for bulk downloads

---

## End-of-Day (EOD) Stock Data

### GET /tiingo/daily/{ticker} - Ticker Metadata

**Example Request:**
```
GET https://api.tiingo.com/tiingo/daily/AAPL?token=YOUR_TOKEN
```

**Response (JSON):**
```json
{
  "ticker": "AAPL",
  "name": "Apple Inc.",
  "exchangeCode": "NASDAQ",
  "startDate": "1980-12-12",
  "endDate": "2020-12-31",
  "description": "Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide."
}
```

**Fields:**
- `ticker` (string): Stock ticker symbol
- `name` (string): Company name
- `exchangeCode` (string): Exchange (NASDAQ, NYSE, AMEX, etc.)
- `startDate` (date): First available data date (YYYY-MM-DD)
- `endDate` (date): Last available data date (YYYY-MM-DD)
- `description` (string): Company description

---

### GET /tiingo/daily/{ticker}/prices - Historical Prices

**Example Request:**
```
GET https://api.tiingo.com/tiingo/daily/AAPL/prices?startDate=2020-01-01&endDate=2020-01-03&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "date": "2020-01-02T00:00:00.000Z",
    "close": 300.35,
    "high": 300.58,
    "low": 298.02,
    "open": 296.24,
    "volume": 135480400,
    "adjClose": 298.12,
    "adjHigh": 298.35,
    "adjLow": 295.81,
    "adjOpen": 294.08,
    "adjVolume": 135480400,
    "divCash": 0.0,
    "splitFactor": 1.0
  },
  {
    "date": "2020-01-03T00:00:00.000Z",
    "close": 297.43,
    "high": 300.58,
    "low": 296.50,
    "open": 297.15,
    "volume": 146322800,
    "adjClose": 295.22,
    "adjHigh": 298.35,
    "adjLow": 294.28,
    "adjOpen": 294.94,
    "adjVolume": 146322800,
    "divCash": 0.0,
    "splitFactor": 1.0
  }
]
```

**Fields:**
- `date` (datetime): ISO8601 timestamp (UTC, 00:00:00 for daily)
- **Raw OHLCV:**
  - `open` (float): Opening price
  - `high` (float): High price
  - `low` (float): Low price
  - `close` (float): Closing price
  - `volume` (integer): Trading volume (shares)
- **Adjusted OHLCV:**
  - `adjOpen` (float): Split/dividend-adjusted open
  - `adjHigh` (float): Adjusted high
  - `adjLow` (float): Adjusted low
  - `adjClose` (float): Adjusted close
  - `adjVolume` (integer): Adjusted volume
- **Corporate Actions:**
  - `divCash` (float): Dividend cash amount (0.0 if none)
  - `splitFactor` (float): Split factor (1.0 if no split)

**Note:** Adjusted prices account for splits and dividends, allowing accurate historical backtesting.

---

## IEX Intraday Data

### GET /iex/{ticker}/prices - Intraday Prices

**Example Request:**
```
GET https://api.tiingo.com/iex/AAPL/prices?startDate=2020-01-02&resampleFreq=5min&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "date": "2020-01-02T09:30:00.000Z",
    "open": 296.24,
    "high": 297.15,
    "low": 296.00,
    "close": 296.80,
    "volume": 1234567
  },
  {
    "date": "2020-01-02T09:35:00.000Z",
    "open": 296.81,
    "high": 297.50,
    "low": 296.70,
    "close": 297.35,
    "volume": 987654
  }
]
```

**Fields:**
- `date` (datetime): ISO8601 timestamp (bar start time)
- `open` (float): Opening price for interval
- `high` (float): High price for interval
- `low` (float): Low price for interval
- `close` (float): Closing price for interval
- `volume` (integer): Volume for interval

**Intervals:** 1min, 5min, 15min, 30min, 1hour, 4hour (via resampleFreq parameter)

---

## Cryptocurrency Data

### GET /tiingo/crypto - Crypto Metadata

**Example Request:**
```
GET https://api.tiingo.com/tiingo/crypto?token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "ticker": "btcusd",
    "name": "Bitcoin",
    "description": "Bitcoin to US Dollar",
    "baseCurrency": "btc",
    "quoteCurrency": "usd"
  },
  {
    "ticker": "ethusd",
    "name": "Ethereum",
    "description": "Ethereum to US Dollar",
    "baseCurrency": "eth",
    "quoteCurrency": "usd"
  }
]
```

**Fields:**
- `ticker` (string): Crypto pair ticker (e.g., "btcusd")
- `name` (string): Cryptocurrency name
- `description` (string): Pair description
- `baseCurrency` (string): Base currency (e.g., "btc")
- `quoteCurrency` (string): Quote currency (e.g., "usd")

---

### GET /tiingo/crypto/top - Top-of-Book Crypto Quotes

**Example Request:**
```
GET https://api.tiingo.com/tiingo/crypto/top?tickers=btcusd,ethusd&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "ticker": "btcusd",
    "baseCurrency": "btc",
    "quoteCurrency": "usd",
    "topOfBookData": [
      {
        "askSize": 0.5,
        "bidSize": 0.3,
        "lastSaleTimestamp": "2020-01-02T12:34:56.789012Z",
        "lastPrice": 45000.50,
        "askPrice": 45001.00,
        "quoteTimestamp": "2020-01-02T12:34:57.000000Z",
        "bidExchange": "binance",
        "lastSizeNotional": 22500.25,
        "lastExchange": "binance",
        "askExchange": "binance",
        "bidPrice": 45000.00
      }
    ]
  },
  {
    "ticker": "ethusd",
    "baseCurrency": "eth",
    "quoteCurrency": "usd",
    "topOfBookData": [
      {
        "askSize": 10.5,
        "bidSize": 8.3,
        "lastSaleTimestamp": "2020-01-02T12:34:58.123456Z",
        "lastPrice": 3200.75,
        "askPrice": 3201.00,
        "quoteTimestamp": "2020-01-02T12:34:58.500000Z",
        "bidExchange": "coinbase",
        "lastSizeNotional": 33607.875,
        "lastExchange": "coinbase",
        "askExchange": "coinbase",
        "bidPrice": 3200.50
      }
    ]
  }
]
```

**Top-level fields:**
- `ticker` (string): Crypto pair
- `baseCurrency` (string): Base currency
- `quoteCurrency` (string): Quote currency
- `topOfBookData` (array): Array of top-of-book quotes (one per exchange or aggregated)

**topOfBookData fields:**
- `bidPrice` (float): Best bid price
- `bidSize` (float): Bid size (crypto units)
- `bidExchange` (string): Exchange with best bid
- `askPrice` (float): Best ask price
- `askSize` (float): Ask size (crypto units)
- `askExchange` (string): Exchange with best ask
- `lastPrice` (float): Last trade price
- `lastSizeNotional` (float): Last trade value (in quote currency)
- `lastExchange` (string): Exchange of last trade
- `lastSaleTimestamp` (datetime): Last trade timestamp (ISO8601)
- `quoteTimestamp` (datetime): Quote timestamp (ISO8601)

---

### GET /tiingo/crypto/prices - Historical Crypto Prices

**Example Request:**
```
GET https://api.tiingo.com/tiingo/crypto/prices?tickers=btcusd&startDate=2020-01-01&endDate=2020-01-02&resampleFreq=1hour&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "ticker": "btcusd",
    "baseCurrency": "btc",
    "quoteCurrency": "usd",
    "priceData": [
      {
        "date": "2020-01-01T00:00:00.000Z",
        "open": 44500.00,
        "high": 44750.50,
        "low": 44300.25,
        "close": 44600.75,
        "volume": 123.45,
        "volumeNotional": 5500000.00,
        "tradesDone": 1234
      },
      {
        "date": "2020-01-01T01:00:00.000Z",
        "open": 44601.00,
        "high": 44850.00,
        "low": 44550.00,
        "close": 44800.00,
        "volume": 98.76,
        "volumeNotional": 4425000.00,
        "tradesDone": 987
      }
    ]
  }
]
```

**Top-level fields:**
- `ticker` (string): Crypto pair
- `baseCurrency` (string): Base currency
- `quoteCurrency` (string): Quote currency
- `priceData` (array): Array of OHLCV bars

**priceData fields:**
- `date` (datetime): ISO8601 timestamp (bar start time)
- `open` (float): Opening price
- `high` (float): High price
- `low` (float): Low price
- `close` (float): Closing price
- `volume` (float): Volume in base currency (e.g., BTC)
- `volumeNotional` (float): Volume in quote currency (e.g., USD)
- `tradesDone` (integer): Number of trades in interval

---

## Forex Data

### GET /tiingo/fx/{ticker}/top - Top-of-Book FX Quote

**Example Request:**
```
GET https://api.tiingo.com/tiingo/fx/eurusd/top?token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "ticker": "eurusd",
    "quoteTimestamp": "2020-01-02T12:34:56.789012Z",
    "bidPrice": 1.1234,
    "bidSize": 1000000.0,
    "askPrice": 1.1236,
    "askSize": 1000000.0,
    "midPrice": 1.1235
  }
]
```

**Fields:**
- `ticker` (string): FX pair ticker (e.g., "eurusd")
- `quoteTimestamp` (datetime): ISO8601 timestamp with microsecond precision
- `bidPrice` (float): Best bid price
- `bidSize` (float): Bid size (notional amount in base currency)
- `askPrice` (float): Best ask price
- `askSize` (float): Ask size (notional amount)
- `midPrice` (float): (bid + ask) / 2 (null if bid or ask is null)

---

### GET /tiingo/fx/{ticker}/prices - Historical FX Prices

**Example Request:**
```
GET https://api.tiingo.com/tiingo/fx/eurusd/prices?startDate=2020-01-01&resampleFreq=5min&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "date": "2020-01-01T00:00:00.000Z",
    "ticker": "eurusd",
    "open": 1.1230,
    "high": 1.1240,
    "low": 1.1225,
    "close": 1.1235
  },
  {
    "date": "2020-01-01T00:05:00.000Z",
    "ticker": "eurusd",
    "open": 1.1235,
    "high": 1.1245,
    "low": 1.1230,
    "close": 1.1240
  }
]
```

**Fields:**
- `date` (datetime): ISO8601 timestamp (bar start time)
- `ticker` (string): FX pair ticker
- `open` (float): Opening price
- `high` (float): High price
- `low` (float): Low price
- `close` (float): Closing price

**Note:** Volume not typically provided for forex (OTC market).

---

## Fundamentals Data

### GET /tiingo/fundamentals/definitions - Field Definitions

**Example Request:**
```
GET https://api.tiingo.com/tiingo/fundamentals/definitions?token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "dataCode": "marketCap",
    "name": "Market Capitalization",
    "statementType": "overview",
    "units": "currency",
    "description": "Total market value of company's outstanding shares"
  },
  {
    "dataCode": "peRatio",
    "name": "Price to Earnings Ratio",
    "statementType": "overview",
    "units": "ratio",
    "description": "Stock price divided by earnings per share"
  }
]
```

**Fields:**
- `dataCode` (string): Field identifier (used in API responses)
- `name` (string): Human-readable name
- `statementType` (string): Statement category (overview, income, balance, cashFlow)
- `units` (string): Units (currency, ratio, shares, etc.)
- `description` (string): Field description

**Use case:** Reference guide for interpreting fundamentals data fields.

---

### GET /tiingo/fundamentals/{ticker}/daily - Daily Fundamentals

**Example Request:**
```
GET https://api.tiingo.com/tiingo/fundamentals/AAPL/daily?startDate=2020-01-01&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "date": "2020-01-02",
    "marketCap": 1300000000000.0,
    "enterpriseVal": 1350000000000.0,
    "peRatio": 25.5,
    "pbRatio": 12.3,
    "trailingPEG1Y": 1.8
  },
  {
    "date": "2020-01-03",
    "marketCap": 1295000000000.0,
    "enterpriseVal": 1345000000000.0,
    "peRatio": 25.3,
    "pbRatio": 12.2,
    "trailingPEG1Y": 1.8
  }
]
```

**Fields (example subset, 80+ total):**
- `date` (date): Date (YYYY-MM-DD)
- `marketCap` (float): Market capitalization
- `enterpriseVal` (float): Enterprise value
- `peRatio` (float): Price-to-earnings ratio
- `pbRatio` (float): Price-to-book ratio
- `trailingPEG1Y` (float): PEG ratio (1-year trailing)

**Note:** Response includes 80+ fundamental indicators. Use /definitions endpoint for complete field list.

---

### GET /tiingo/fundamentals/{ticker}/statements - Financial Statements

**Example Request:**
```
GET https://api.tiingo.com/tiingo/fundamentals/AAPL/statements?token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "date": "2020-09-30",
    "quarter": 4,
    "year": 2020,
    "statementType": "income",
    "dataCode": "revenue",
    "value": 274515000000.0
  },
  {
    "date": "2020-09-30",
    "quarter": 4,
    "year": 2020,
    "statementType": "income",
    "dataCode": "netinc",
    "value": 57411000000.0
  },
  {
    "date": "2020-09-30",
    "quarter": 4,
    "year": 2020,
    "statementType": "balance",
    "dataCode": "assetsCurrent",
    "value": 143713000000.0
  }
]
```

**Fields:**
- `date` (date): Statement date (fiscal period end, YYYY-MM-DD)
- `quarter` (integer): Fiscal quarter (1-4, or 0 for annual)
- `year` (integer): Fiscal year
- `statementType` (string): Statement type (income, balance, cashFlow, overview)
- `dataCode` (string): Field identifier (see /definitions)
- `value` (float): Field value

**Statement Types:**
- `income`: Income statement (revenue, COGS, net income, EPS, etc.)
- `balance`: Balance sheet (assets, liabilities, equity)
- `cashFlow`: Cash flow statement (operating, investing, financing cash flows)
- `overview`: Company overview and metrics

**Parameter `asReported`:**
- `asReported=true`: Raw data from SEC filings
- `asReported=false` (default): Standardized/normalized format

---

## News Data

### GET /tiingo/news - Financial News

**Example Request:**
```
GET https://api.tiingo.com/tiingo/news?tickers=AAPL&limit=2&token=YOUR_TOKEN
```

**Response (JSON Array):**
```json
[
  {
    "id": "abc123",
    "title": "Apple Announces Record Q4 Earnings",
    "url": "https://example.com/apple-earnings",
    "description": "Apple Inc. reported record revenue for Q4...",
    "publishedDate": "2020-01-02T14:30:00Z",
    "crawlDate": "2020-01-02T14:35:12Z",
    "source": "example.com",
    "tickers": ["AAPL"],
    "tags": ["Earnings", "Technology"]
  },
  {
    "id": "def456",
    "title": "Apple Stock Rises on Strong iPhone Sales",
    "url": "https://example.com/apple-stock",
    "description": "Shares of Apple rose 3% following strong iPhone sales data...",
    "publishedDate": "2020-01-03T09:15:00Z",
    "crawlDate": "2020-01-03T09:20:45Z",
    "source": "example.com",
    "tickers": ["AAPL"],
    "tags": ["Stock Market", "iPhone"]
  }
]
```

**Fields:**
- `id` (string): Unique article ID
- `title` (string): Article title
- `url` (string): Article URL
- `description` (string): Article summary/excerpt
- `publishedDate` (datetime): Publication timestamp (ISO8601)
- `crawlDate` (datetime): When Tiingo crawled the article (ISO8601)
- `source` (string): Source domain (e.g., "example.com")
- `tickers` (array[string]): Related ticker symbols
- `tags` (array[string]): Article tags/keywords

**Pagination:**
- Use `limit` parameter (default: 100)
- Use `offset` parameter for pagination

---

## WebSocket Message Formats

### IEX WebSocket - Price Update (messageType "A")

```json
{
  "service": "iex",
  "messageType": "A",
  "data": [
    {
      "ticker": "AAPL",
      "timestamp": "2020-01-02T12:34:56.789012Z",
      "last": 150.25,
      "lastSize": 100,
      "tngoLast": 150.25,
      "prevClose": 149.50,
      "open": 149.80,
      "high": 150.50,
      "low": 149.70,
      "mid": 150.245,
      "volume": 1234567,
      "bidSize": 200,
      "bidPrice": 150.23,
      "askSize": 150,
      "askPrice": 150.26,
      "quoteTimestamp": "2020-01-02T12:34:56.789012Z",
      "lastSaleTimestamp": "2020-01-02T12:34:56.500000Z"
    }
  ]
}
```

**Top-level fields:**
- `service` (string): "iex"
- `messageType` (string): "A" = price update
- `data` (array): Array of ticker updates

**Data fields:**
- `ticker` (string): Stock symbol
- `timestamp` (datetime): Update timestamp (ISO8601, microsecond precision)
- `last` (float): Last trade price
- `lastSize` (integer): Last trade size (shares)
- `tngoLast` (float): Tiingo's last price
- `prevClose` (float): Previous close
- `open` (float): Today's open
- `high` (float): Today's high
- `low` (float): Today's low
- `mid` (float): Mid price (bid + ask) / 2
- `volume` (integer): Cumulative volume
- `bidPrice` (float): Best bid
- `bidSize` (integer): Bid size
- `askPrice` (float): Best ask
- `askSize` (integer): Ask size
- `quoteTimestamp` (datetime): Quote timestamp
- `lastSaleTimestamp` (datetime): Last sale timestamp

---

### IEX WebSocket - Heartbeat (messageType "H")

```json
{
  "service": "iex",
  "messageType": "H",
  "data": []
}
```

**Fields:**
- `service` (string): "iex"
- `messageType` (string): "H" = heartbeat
- `data` (array): Empty array

---

### Forex WebSocket - Quote Update (messageType "A")

```json
{
  "service": "fx",
  "messageType": "A",
  "data": [
    {
      "ticker": "eurusd",
      "quoteTimestamp": "2020-01-02T12:34:56.789012Z",
      "bidPrice": 1.1234,
      "bidSize": 1000000.0,
      "askPrice": 1.1236,
      "askSize": 1000000.0,
      "midPrice": 1.1235
    }
  ]
}
```

**Fields:**
- `service` (string): "fx"
- `messageType` (string): "A" = quote update
- `data` (array): Array of FX pair updates
- Data fields same as REST /fx/top response

---

### Crypto WebSocket - Quote Update (messageType "A")

```json
{
  "service": "crypto",
  "messageType": "A",
  "data": [
    {
      "ticker": "btcusd",
      "exchange": "binance",
      "quoteTimestamp": "2020-01-02T12:34:56.789Z",
      "bidPrice": 45000.50,
      "bidSize": 0.5,
      "askPrice": 45001.50,
      "askSize": 0.3,
      "midPrice": 45001.00,
      "lastPrice": 45000.75,
      "lastSize": 0.1
    }
  ]
}
```

**Fields:**
- `service` (string): "crypto"
- `messageType` (string): "A" = quote update
- `data` (array): Array of crypto updates
- Data fields similar to REST /crypto/top response

---

## Error Response Formats

### 401 Unauthorized

```json
{
  "detail": "Authentication credentials were not provided."
}
```

### 403 Forbidden

```json
{
  "detail": "You do not have permission to access this resource."
}
```

### 404 Not Found

```json
{
  "detail": "Not found."
}
```

### 429 Rate Limit Exceeded

```json
{
  "detail": "Request was throttled. Expected available in 30 seconds."
}
```

**Headers (on 429):**
```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1234567890
Retry-After: 30
```

### 500 Internal Server Error

```json
{
  "detail": "Internal server error."
}
```

---

## CSV Response Format

### Example: GET /tiingo/daily/AAPL/prices?format=csv

**CSV Output:**
```csv
date,close,high,low,open,volume,adjClose,adjHigh,adjLow,adjOpen,adjVolume,divCash,splitFactor
2020-01-02T00:00:00.000Z,300.35,300.58,298.02,296.24,135480400,298.12,298.35,295.81,294.08,135480400,0.0,1.0
2020-01-03T00:00:00.000Z,297.43,300.58,296.50,297.15,146322800,295.22,298.35,294.28,294.94,146322800,0.0,1.0
```

**Features:**
- Header row with column names
- ISO8601 timestamps
- Same fields as JSON response
- Optimized for bulk downloads and data imports

---

## Summary

- **Consistent JSON structure**: All REST endpoints return JSON arrays or objects
- **ISO8601 timestamps**: All dates/times in ISO8601 format with timezone
- **Adjusted data**: Stock prices include both raw and adjusted values
- **CSV support**: All endpoints support CSV export (format=csv)
- **WebSocket**: Unified message format (service, messageType, data)
- **Error responses**: Standard HTTP status codes with JSON error details
- **Microsecond precision**: WebSocket timestamps include microseconds
- **Field definitions**: Use /fundamentals/definitions for complete field reference
