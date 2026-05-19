# yahoo - Complete Endpoint Reference

## Category: Market Data - Current Prices & Quotes

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v7/finance/quote | Get current quote data | Yes | No | ~2000/hr | Supports multiple symbols |
| GET | /v8/finance/chart/{symbol} | Real-time chart data | Yes | No | ~2000/hr | Includes pre/post market |
| GET | /v11/finance/quoteSummary/{symbol} | Comprehensive quote summary | Yes | No | ~2000/hr | Modular data retrieval |
| GET | /v6/finance/quote/marketSummary | Market overview | Yes | No | ~2000/hr | All major indices |
| GET | /v1/finance/spark | Mini chart sparkline data | Yes | No | ~2000/hr | Lightweight price history |

## Category: Historical Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v7/finance/download/{symbol} | Historical price download | Yes | Yes | ~2000/hr | Requires crumb + cookie |
| GET | /v8/finance/chart/{symbol} | Historical OHLCV data | Yes | No | ~2000/hr | Interval-based |

## Category: Options Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v7/finance/options/{symbol} | Options chain | Yes | No | ~2000/hr | All strikes/expirations |

## Category: Fundamental Data (Stocks)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v10/finance/quoteSummary/{symbol}?modules=assetProfile | Company profile | Yes | No | ~2000/hr | Officers, location, sector |
| GET | /v10/finance/quoteSummary/{symbol}?modules=incomeStatementHistory | Income statement annual | Yes | No | ~2000/hr | Historical financials |
| GET | /v10/finance/quoteSummary/{symbol}?modules=incomeStatementHistoryQuarterly | Income statement quarterly | Yes | No | ~2000/hr | Quarterly financials |
| GET | /v10/finance/quoteSummary/{symbol}?modules=balanceSheetHistory | Balance sheet annual | Yes | No | ~2000/hr | Historical balance sheets |
| GET | /v10/finance/quoteSummary/{symbol}?modules=balanceSheetHistoryQuarterly | Balance sheet quarterly | Yes | No | ~2000/hr | Quarterly balance sheets |
| GET | /v10/finance/quoteSummary/{symbol}?modules=cashflowStatementHistory | Cash flow annual | Yes | No | ~2000/hr | Historical cash flows |
| GET | /v10/finance/quoteSummary/{symbol}?modules=cashflowStatementHistoryQuarterly | Cash flow quarterly | Yes | No | ~2000/hr | Quarterly cash flows |
| GET | /v10/finance/quoteSummary/{symbol}?modules=earnings | Earnings data | Yes | No | ~2000/hr | EPS, revenue, estimates |
| GET | /v10/finance/quoteSummary/{symbol}?modules=earningsHistory | Historical earnings | Yes | No | ~2000/hr | Past earnings releases |
| GET | /v10/finance/quoteSummary/{symbol}?modules=earningsTrend | Earnings estimates | Yes | No | ~2000/hr | Future projections |
| GET | /v10/finance/quoteSummary/{symbol}?modules=financialData | Financial metrics | Yes | No | ~2000/hr | Key ratios, margins |
| GET | /v10/finance/quoteSummary/{symbol}?modules=defaultKeyStatistics | Key statistics | Yes | No | ~2000/hr | P/E, beta, market cap |
| GET | /v10/finance/quoteSummary/{symbol}?modules=calendarEvents | Corporate events | Yes | No | ~2000/hr | Earnings dates, dividends |
| GET | /v10/finance/quoteSummary/{symbol}?modules=secFilings | SEC filings | Yes | No | ~2000/hr | Links to filings |
| GET | /v10/finance/quoteSummary/{symbol}?modules=upgradeDowngradeHistory | Analyst actions | Yes | No | ~2000/hr | Upgrades/downgrades |
| GET | /v10/finance/quoteSummary/{symbol}?modules=recommendationTrend | Analyst ratings | Yes | No | ~2000/hr | Buy/hold/sell counts |

## Category: Ownership & Institutional Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v10/finance/quoteSummary/{symbol}?modules=insiderHolders | Insider holdings | Yes | No | ~2000/hr | Company insiders |
| GET | /v10/finance/quoteSummary/{symbol}?modules=insiderTransactions | Insider trades | Yes | No | ~2000/hr | Buy/sell transactions |
| GET | /v10/finance/quoteSummary/{symbol}?modules=institutionOwnership | Institutional holders | Yes | No | ~2000/hr | Major institutions |
| GET | /v10/finance/quoteSummary/{symbol}?modules=fundOwnership | Fund ownership | Yes | No | ~2000/hr | Mutual fund holders |
| GET | /v10/finance/quoteSummary/{symbol}?modules=majorDirectHolders | Major direct holders | Yes | No | ~2000/hr | Large shareholders |
| GET | /v10/finance/quoteSummary/{symbol}?modules=majorHoldersBreakdown | Ownership breakdown | Yes | No | ~2000/hr | % owned by category |
| GET | /v10/finance/quoteSummary/{symbol}?modules=netSharePurchaseActivity | Share purchase activity | Yes | No | ~2000/hr | Net buying/selling |

## Category: Fund-Specific Data (ETFs, Mutual Funds)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v10/finance/quoteSummary/{symbol}?modules=fundProfile | Fund profile | Yes | No | ~2000/hr | Fund details |
| GET | /v10/finance/quoteSummary/{symbol}?modules=fundPerformance | Fund performance | Yes | No | ~2000/hr | Returns over time |
| GET | /v10/finance/quoteSummary/{symbol}?modules=topHoldings | Top holdings | Yes | No | ~2000/hr | Largest positions |

## Category: Market Trends & Analysis

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v10/finance/quoteSummary/{symbol}?modules=indexTrend | Index trend | Yes | No | ~2000/hr | Market direction |
| GET | /v10/finance/quoteSummary/{symbol}?modules=industryTrend | Industry trend | Yes | No | ~2000/hr | Industry analysis |
| GET | /v10/finance/quoteSummary/{symbol}?modules=sectorTrend | Sector trend | Yes | No | ~2000/hr | Sector performance |
| GET | /v1/finance/trending/{region} | Trending symbols | Yes | No | ~2000/hr | Popular tickers |
| GET | /v1/finance/recommendationsBySymbol/{symbol} | Recommendations | Yes | No | ~2000/hr | Similar symbols |

## Category: Search & Discovery

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/finance/search | Search symbols | Yes | No | ~2000/hr | Fuzzy search |
| GET | /v1/finance/lookup | Symbol lookup | Yes | No | ~2000/hr | Exact match |
| GET | /v1/finance/screener/predefined | Predefined screeners | Yes | No | ~2000/hr | Market cap, gainers, etc |
| POST | /v1/finance/screener | Custom screener | Yes | No | ~2000/hr | Advanced filtering |

## Category: ESG Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v10/finance/quoteSummary/{symbol}?modules=esgScores | ESG scores | Yes | No | ~2000/hr | Environmental/Social/Governance |

## Category: Time Series Fundamentals

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /ws/fundamentals-timeseries/v1/finance/timeseries/{symbol} | Time series data | Yes | No | ~2000/hr | Historical fundamentals |

## Category: News

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/finance/search | Search with news | Yes | No | ~2000/hr | Returns news in results |

## Category: Metadata

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v10/finance/quoteSummary/{symbol}?modules=summaryProfile | Summary profile | Yes | No | ~2000/hr | Basic info |
| GET | /v10/finance/quoteSummary/{symbol}?modules=summaryDetail | Summary details | Yes | No | ~2000/hr | Key details |
| GET | /v10/finance/quoteSummary/{symbol}?modules=price | Price metadata | Yes | No | ~2000/hr | Price info |
| GET | /v10/finance/quoteSummary/{symbol}?modules=quoteType | Quote type | Yes | No | ~2000/hr | Asset classification |

## Parameters Reference

### GET /v7/finance/quote
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbols | string | Yes | - | Comma-separated ticker symbols (e.g., "AAPL,MSFT,GOOGL") |
| fields | string | No | all | Comma-separated field names to return |
| formatted | boolean | No | false | Return formatted strings |

**Example:**
```
GET https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL,MSFT
```

### GET /v8/finance/chart/{symbol}
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol (path parameter) |
| period1 | integer | No | - | Start time (Unix timestamp) |
| period2 | integer | No | - | End time (Unix timestamp) |
| interval | string | No | "1d" | 1m,2m,5m,15m,30m,60m,90m,1h,1d,5d,1wk,1mo,3mo |
| range | string | No | - | Shortcut: 1d,5d,1mo,3mo,6mo,1y,2y,5y,10y,ytd,max |
| events | string | No | - | div (dividends), split, capitalGain |
| includePrePost | boolean | No | true | Include pre/post market data |

**Interval Limitations:**
- 1m data: Last 7 days only
- <1d intervals: Last 60 days only
- 1h data: Last 730 days only

**Example:**
```
GET https://query2.finance.yahoo.com/v8/finance/chart/AAPL?period1=1699549200&period2=1731319200&interval=1d
```

### GET /v7/finance/download/{symbol}
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol (path parameter) |
| period1 | integer | Yes | - | Start time (Unix timestamp) |
| period2 | integer | Yes | - | End time (Unix timestamp) |
| interval | string | No | "1d" | 1d, 1wk, 1mo |
| events | string | No | "history" | history, div, split |
| crumb | string | Yes | - | Authentication crumb |

**Notes:**
- Requires valid cookie in request headers
- Crumb must be obtained from /v1/test/getcrumb endpoint first

**Example:**
```
GET https://query1.finance.yahoo.com/v7/finance/download/AAPL?period1=1609459200&period2=1640995200&interval=1d&events=history&crumb=XXXXXX
```

### GET /v10/finance/quoteSummary/{symbol}
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol (path parameter) |
| modules | string | Yes | - | Comma-separated module names (see modules below) |
| formatted | boolean | No | false | Return formatted strings |

**Available Modules:**
- assetProfile
- balanceSheetHistory
- balanceSheetHistoryQuarterly
- calendarEvents
- cashflowStatementHistory
- cashflowStatementHistoryQuarterly
- defaultKeyStatistics
- earnings
- earningsHistory
- earningsTrend
- esgScores
- financialData
- fundOwnership
- fundPerformance
- fundProfile
- incomeStatementHistory
- incomeStatementHistoryQuarterly
- indexTrend
- industryTrend
- insiderHolders
- insiderTransactions
- institutionOwnership
- majorDirectHolders
- majorHoldersBreakdown
- netSharePurchaseActivity
- price
- quoteType
- recommendationTrend
- secFilings
- sectorTrend
- summaryDetail
- summaryProfile
- symbol
- topHoldings
- upgradeDowngradeHistory

**Example:**
```
GET https://query2.finance.yahoo.com/v10/finance/quoteSummary/AAPL?modules=assetProfile,financialData,earnings
```

### GET /v7/finance/options/{symbol}
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol (path parameter) |
| date | integer | No | nearest | Unix timestamp of expiration date |

**Example:**
```
GET https://query1.finance.yahoo.com/v7/finance/options/AAPL
GET https://query1.finance.yahoo.com/v7/finance/options/AAPL?date=1640995200
```

### POST /v1/finance/screener
**Body Structure:**
```json
{
  "size": 250,
  "offset": 0,
  "sortField": "intradaymarketcap",
  "sortType": "DESC",
  "quoteType": "EQUITY",
  "query": {
    "operator": "AND",
    "operands": [
      {
        "operator": "GT",
        "operands": ["intradaymarketcap", 2000000000]
      },
      {
        "operator": "LT",
        "operands": ["intradaymarketcap", 100000000000]
      }
    ]
  },
  "userId": "",
  "userIdType": "guid"
}
```

**Supported Operators:**
- Comparison: GT (>), LT (<), EQ (=), BTWN (between)
- Logical: AND, OR

**Example:**
```
POST https://query2.finance.yahoo.com/v1/finance/screener
Content-Type: application/json
```

### GET /v1/finance/search
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| q | string | Yes | - | Search query |
| quotesCount | integer | No | 10 | Max quotes to return |
| newsCount | integer | No | 10 | Max news to return |
| enableFuzzyQuery | boolean | No | true | Enable fuzzy matching |

**Example:**
```
GET https://query2.finance.yahoo.com/v1/finance/search?q=apple&quotesCount=10&newsCount=5
```

## Special Notes on Symbol Formats

Yahoo Finance uses specific ticker formats for different asset types:

**Stocks (US):** AAPL, MSFT, GOOGL (standard ticker)
**Stocks (International):** 0700.HK (Hong Kong), SAP.DE (Germany)
**Cryptocurrencies:** BTC-USD, ETH-USD, XRP-USD (crypto-fiat pairs)
**Forex:** EURUSD=X, GBPUSD=X, JPYUSD=X (currency pairs with =X suffix)
**Commodities:** GC=F (Gold), SI=F (Silver), CL=F (Crude Oil) (futures with =F suffix)
**Indices:** ^GSPC (S&P 500), ^DJI (Dow), ^IXIC (Nasdaq) (indices with ^ prefix)
**Treasuries:** ^TNX (10-year), ^IRX (13-week), ^TYX (30-year) (yields with ^ prefix)

## Rate Limiting Notes

- No official rate limit documentation
- Community estimates ~2000 requests/hour per IP
- 429 errors indicate rate limit exceeded
- No rate limit headers in responses
- IP-based blocking for excessive requests
- Use delays between requests (recommended: 500ms-1s)
- Rotate IPs if making heavy requests (use proxies)
