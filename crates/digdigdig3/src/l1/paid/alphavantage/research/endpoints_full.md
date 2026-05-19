# AlphaVantage - Complete Endpoint Reference

**Base URL**: `https://www.alphavantage.co/query`

All endpoints use the same base URL with different `function` parameter values.

**General Parameters**:
- `apikey` (required): Your API key
- `datatype` (optional): `json` (default) or `csv`
- `outputsize` (optional for time series): `compact` (100 data points, default for free tier) or `full` (all available data, premium)

---

## Category: Core Stock Data

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `TIME_SERIES_INTRADAY` | Intraday time series (1min, 5min, 15min, 30min, 60min) | No (Premium) | Yes | 5/min free, 75-1200/min premium | Requires premium tier |
| `TIME_SERIES_DAILY` | Daily time series (open, high, low, close, volume) | Yes | Yes | 5/min free, 75-1200/min premium | Up to 20+ years history |
| `TIME_SERIES_DAILY_ADJUSTED` | Daily adjusted time series (splits, dividends) | No (Premium) | Yes | 5/min free, 75-1200/min premium | Premium only |
| `TIME_SERIES_WEEKLY` | Weekly time series | Yes | Yes | 5/min free, 75-1200/min premium | Last 20+ years |
| `TIME_SERIES_WEEKLY_ADJUSTED` | Weekly adjusted time series | Yes | Yes | 5/min free, 75-1200/min premium | Adjusted for splits/dividends |
| `TIME_SERIES_MONTHLY` | Monthly time series | Yes | Yes | 5/min free, 75-1200/min premium | Last 20+ years |
| `TIME_SERIES_MONTHLY_ADJUSTED` | Monthly adjusted time series | Yes | Yes | 5/min free, 75-1200/min premium | Adjusted for splits/dividends |
| `GLOBAL_QUOTE` | Latest price and stats for a symbol | Yes | Yes | 5/min free, 75-1200/min premium | Real-time quote (Trending) |

### TIME_SERIES_INTRADAY Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | TIME_SERIES_INTRADAY |
| symbol | string | Yes | - | Stock ticker (e.g., IBM, AAPL) |
| interval | string | Yes | - | 1min, 5min, 15min, 30min, 60min |
| adjusted | boolean | No | true | Adjust for splits |
| extended_hours | boolean | No | true | Include pre/post-market |
| month | string | No | - | YYYY-MM for specific month |
| outputsize | string | No | compact | compact (100) or full |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

### TIME_SERIES_DAILY Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | TIME_SERIES_DAILY |
| symbol | string | Yes | - | Stock ticker |
| outputsize | string | No | compact | compact (100) or full (20+ years) |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

### GLOBAL_QUOTE Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | GLOBAL_QUOTE |
| symbol | string | Yes | - | Stock ticker |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

---

## Category: Quote & Bulk Data

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `REALTIME_BULK_QUOTES` | Bulk quotes for up to 100 symbols | No (Premium) | Yes | Premium | Premium only, efficient for multiple symbols |

### REALTIME_BULK_QUOTES Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | REALTIME_BULK_QUOTES |
| symbol | string | Yes | - | Comma-separated list (max 100) |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

---

## Category: Utility & Metadata

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `SYMBOL_SEARCH` | Search for ticker symbols with match scoring | Yes | Yes | 5/min free | Best match results |
| `MARKET_STATUS` | Global market open/closed status | Yes | Yes | 5/min free | Real-time market hours |
| `LISTING_STATUS` | List of active/delisted tickers | Yes | Yes | 5/min free | US exchanges, includes IPO/delisting dates |

### SYMBOL_SEARCH Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | SYMBOL_SEARCH |
| keywords | string | Yes | - | Search query (e.g., "microsoft") |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

### MARKET_STATUS Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | MARKET_STATUS |
| apikey | string | Yes | - | Your API key |

---

## Category: Options Data (Premium)

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `REALTIME_OPTIONS` | Real-time options chains with Greeks & IV | No (Premium) | Yes | Premium | Premium only, comprehensive options data |
| `HISTORICAL_OPTIONS` | Historical options data, 15+ years | No (Premium) | Yes | Premium | Premium only, deep history |

### REALTIME_OPTIONS Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | REALTIME_OPTIONS |
| symbol | string | Yes | - | Underlying stock ticker |
| contract | string | No | - | Specific option contract |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

---

## Category: Alpha Intelligence (News & Sentiment)

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `NEWS_SENTIMENT` | Market news with AI sentiment analysis | Yes* | Yes | 5/min free | Trending feature, premium for extended access |
| `EARNINGS_CALL_TRANSCRIPT` | Full earnings call transcripts | Yes* | Yes | 5/min free | Limited free access |
| `TOP_GAINERS_LOSERS` | Top movers in the market | Yes | Yes | 5/min free | Daily top gainers, losers, most active |
| `INSIDER_TRANSACTIONS` | Insider trading activity | Yes | Yes | 5/min free | Trending feature, SEC Form 4 data |

### NEWS_SENTIMENT Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | NEWS_SENTIMENT |
| tickers | string | No | - | Comma-separated tickers to filter |
| topics | string | No | - | Topics to filter (e.g., "technology") |
| time_from | string | No | - | Start time YYYYMMDDTHHMM |
| time_to | string | No | - | End time YYYYMMDDTHHMM |
| sort | string | No | LATEST | LATEST, EARLIEST, RELEVANCE |
| limit | int | No | 50 | Max 1000 |
| apikey | string | Yes | - | Your API key |

### TOP_GAINERS_LOSERS Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | TOP_GAINERS_LOSERS |
| apikey | string | Yes | - | Your API key |

### INSIDER_TRANSACTIONS Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | INSIDER_TRANSACTIONS |
| symbol | string | No | - | Stock ticker (optional filter) |
| apikey | string | Yes | - | Your API key |

---

## Category: Fundamental Data

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `COMPANY_OVERVIEW` | Company profile and key metrics | Yes | Yes | 5/min free | Trending feature, comprehensive profile |
| `ETF_PROFILE` | ETF profile and holdings | Yes | Yes | 5/min free | ETF-specific data |
| `DIVIDENDS` | Historical dividend data | Yes | Yes | 5/min free | Cash dividends |
| `SPLITS` | Stock split history | Yes | Yes | 5/min free | All historical splits |
| `INCOME_STATEMENT` | Income statement (annual/quarterly) | Yes | Yes | 5/min free | Full income statement |
| `BALANCE_SHEET` | Balance sheet (annual/quarterly) | Yes | Yes | 5/min free | Complete balance sheet |
| `CASH_FLOW` | Cash flow statement (annual/quarterly) | Yes | Yes | 5/min free | Operating, investing, financing |
| `EARNINGS` | Earnings history and estimates | Yes | Yes | 5/min free | EPS, revenue, surprises |
| `EARNINGS_CALENDAR` | Upcoming earnings dates | Yes | Yes | 5/min free | Next 3 months |
| `IPO_CALENDAR` | Upcoming IPOs | Yes | Yes | 5/min free | Expected IPO dates |
| `SHARES_OUTSTANDING` | Historical shares outstanding | Yes | Yes | 5/min free | Share count history |

### COMPANY_OVERVIEW Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | COMPANY_OVERVIEW |
| symbol | string | Yes | - | Stock ticker |
| apikey | string | Yes | - | Your API key |

### INCOME_STATEMENT Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | INCOME_STATEMENT |
| symbol | string | Yes | - | Stock ticker |
| apikey | string | Yes | - | Your API key |

### EARNINGS Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | EARNINGS |
| symbol | string | Yes | - | Stock ticker |
| apikey | string | Yes | - | Your API key |

### EARNINGS_CALENDAR Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | EARNINGS_CALENDAR |
| symbol | string | No | - | Optional filter by ticker |
| horizon | string | No | 3month | 3month, 6month, 12month |
| apikey | string | Yes | - | Your API key |

---

## Category: Forex (FX)

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `CURRENCY_EXCHANGE_RATE` | Real-time exchange rate for currency pair | Yes | Yes | 5/min free | Trending feature, instant rate |
| `FX_INTRADAY` | Intraday FX time series (1min-60min) | No (Premium) | Yes | Premium | Premium only |
| `FX_DAILY` | Daily FX time series | Yes | Yes | 5/min free | Up to 20+ years |
| `FX_WEEKLY` | Weekly FX time series | Yes | Yes | 5/min free | Multi-year history |
| `FX_MONTHLY` | Monthly FX time series | Yes | Yes | 5/min free | Multi-year history |

**Supported**: 182 physical currencies (see coverage.md for full list)

### CURRENCY_EXCHANGE_RATE Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | CURRENCY_EXCHANGE_RATE |
| from_currency | string | Yes | - | From currency code (e.g., USD, EUR, BTC) |
| to_currency | string | Yes | - | To currency code |
| apikey | string | Yes | - | Your API key |

### FX_INTRADAY Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | FX_INTRADAY |
| from_symbol | string | Yes | - | From currency code |
| to_symbol | string | Yes | - | To currency code |
| interval | string | Yes | - | 1min, 5min, 15min, 30min, 60min |
| outputsize | string | No | compact | compact or full |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

### FX_DAILY Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | FX_DAILY |
| from_symbol | string | Yes | - | From currency code |
| to_symbol | string | Yes | - | To currency code |
| outputsize | string | No | compact | compact (100) or full |
| datatype | string | No | json | json or csv |
| apikey | string | Yes | - | Your API key |

---

## Category: Cryptocurrencies

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `CRYPTO_RATING` | Crypto Fundamental analysis & rating | Yes | Yes | 5/min free | Trending, FCAS score |
| `CRYPTO_INTRADAY` | Intraday crypto time series | No (Premium) | Yes | Premium | Premium only, various intervals |
| `DIGITAL_CURRENCY_DAILY` | Daily crypto time series | Yes | Yes | 5/min free | Historical crypto prices |
| `DIGITAL_CURRENCY_WEEKLY` | Weekly crypto time series | Yes | Yes | 5/min free | Multi-year history |
| `DIGITAL_CURRENCY_MONTHLY` | Monthly crypto time series | Yes | Yes | 5/min free | Multi-year history |

### CRYPTO_RATING Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | CRYPTO_RATING |
| symbol | string | Yes | - | Crypto symbol (e.g., BTC) |
| apikey | string | Yes | - | Your API key |

### DIGITAL_CURRENCY_DAILY Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | DIGITAL_CURRENCY_DAILY |
| symbol | string | Yes | - | Crypto symbol (e.g., BTC) |
| market | string | Yes | - | Market currency (e.g., USD, EUR) |
| apikey | string | Yes | - | Your API key |

---

## Category: Commodities

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `WTI` | Crude Oil WTI spot and historical | Yes | Yes | 5/min free | Trending, daily/monthly |
| `BRENT` | Crude Oil Brent spot and historical | Yes | Yes | 5/min free | Trending, daily/monthly |
| `NATURAL_GAS` | Natural gas prices | Yes | Yes | 5/min free | Daily/monthly |
| `COPPER` | Copper prices | Yes | Yes | 5/min free | Daily/monthly/quarterly |
| `ALUMINUM` | Aluminum prices | Yes | Yes | 5/min free | Quarterly/annual |
| `WHEAT` | Wheat prices | Yes | Yes | 5/min free | Quarterly/annual |
| `CORN` | Corn prices | Yes | Yes | 5/min free | Quarterly/annual |
| `COTTON` | Cotton prices | Yes | Yes | 5/min free | Quarterly/annual |
| `SUGAR` | Sugar prices | Yes | Yes | 5/min free | Quarterly/annual |
| `COFFEE` | Coffee prices | Yes | Yes | 5/min free | Quarterly/annual |
| `ALL_COMMODITIES` | All commodity prices index | Yes | Yes | 5/min free | Composite index |

### WTI/BRENT Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | WTI or BRENT |
| interval | string | No | monthly | daily, weekly, monthly |
| apikey | string | Yes | - | Your API key |

### General Commodity Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | COPPER, ALUMINUM, etc. |
| interval | string | No | monthly | Varies by commodity |
| apikey | string | Yes | - | Your API key |

---

## Category: Economic Indicators

| Function | Description | Free? | Auth? | Rate Limit | Notes |
|----------|-------------|-------|-------|------------|-------|
| `REAL_GDP` | US Real GDP (quarterly) | Yes | Yes | 5/min free | Quarterly data |
| `REAL_GDP_PER_CAPITA` | US Real GDP per capita | Yes | Yes | 5/min free | Annual data |
| `TREASURY_YIELD` | US Treasury yield curve | Yes | Yes | 5/min free | Trending, daily/monthly |
| `FEDERAL_FUNDS_RATE` | Federal funds rate | Yes | Yes | 5/min free | Daily/monthly/weekly |
| `CPI` | Consumer Price Index (inflation) | Yes | Yes | 5/min free | Monthly |
| `INFLATION` | US Inflation rate | Yes | Yes | 5/min free | Annual |
| `RETAIL_SALES` | US Retail sales | Yes | Yes | 5/min free | Monthly |
| `DURABLES` | Durable goods orders | Yes | Yes | 5/min free | Monthly |
| `UNEMPLOYMENT` | US Unemployment rate | Yes | Yes | 5/min free | Monthly |
| `NONFARM_PAYROLL` | US Non-farm payroll | Yes | Yes | 5/min free | Monthly |

### Economic Indicator Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | REAL_GDP, CPI, etc. |
| interval | string | No | monthly | Varies: annual, quarterly, monthly |
| apikey | string | Yes | - | Your API key |

### TREASURY_YIELD Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | TREASURY_YIELD |
| interval | string | No | monthly | daily, weekly, monthly |
| maturity | string | No | 10year | 3month, 2year, 5year, 7year, 10year, 30year |
| apikey | string | Yes | - | Your API key |

---

## Category: Technical Indicators (50+)

**All technical indicators follow the same parameter pattern**:

### Common Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| function | string | Yes | - | Indicator name (SMA, EMA, RSI, etc.) |
| symbol | string | Yes | - | Stock/FX/Crypto ticker |
| interval | string | Yes | - | 1min, 5min, 15min, 30min, 60min, daily, weekly, monthly |
| time_period | int | No | Varies | Look-back period (e.g., 20 for SMA20) |
| series_type | string | No | close | close, open, high, low |
| apikey | string | Yes | - | Your API key |

### Moving Averages

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `SMA` | Simple Moving Average | Yes | - |
| `EMA` | Exponential Moving Average | Yes | - |
| `WMA` | Weighted Moving Average | Yes | - |
| `DEMA` | Double Exponential MA | Yes | - |
| `TEMA` | Triple Exponential MA | Yes | - |
| `TRIMA` | Triangular Moving Average | Yes | - |
| `KAMA` | Kaufman Adaptive MA | Yes | - |
| `MAMA` | MESA Adaptive MA | Yes | - |
| `VWAP` | Volume Weighted Average Price | No (Premium) | Premium only |
| `T3` | T3 Moving Average | Yes | - |

### Momentum Indicators

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `MACD` | Moving Average Convergence Divergence | No (Premium) | Premium only |
| `MACDEXT` | MACD with controllable MA type | Yes | - |
| `RSI` | Relative Strength Index | Yes | - |
| `STOCH` | Stochastic Oscillator | Yes | - |
| `STOCHF` | Stochastic Fast | Yes | - |
| `STOCHRSI` | Stochastic RSI | Yes | - |
| `WILLR` | Williams %R | Yes | - |
| `ADX` | Average Directional Movement Index | Yes | - |
| `ADXR` | ADX Rating | Yes | - |
| `APO` | Absolute Price Oscillator | Yes | - |
| `PPO` | Percentage Price Oscillator | Yes | - |
| `MOM` | Momentum | Yes | - |
| `BOP` | Balance of Power | Yes | - |
| `CCI` | Commodity Channel Index | Yes | - |
| `CMO` | Chande Momentum Oscillator | Yes | - |
| `ROC` | Rate of Change | Yes | - |
| `ROCR` | Rate of Change Ratio | Yes | - |
| `AROON` | Aroon Indicator | Yes | - |
| `AROONOSC` | Aroon Oscillator | Yes | - |
| `MFI` | Money Flow Index | Yes | - |
| `TRIX` | Triple Exponential MA Oscillator | Yes | - |
| `ULTOSC` | Ultimate Oscillator | Yes | - |
| `DX` | Directional Movement Index | Yes | - |

### Volatility Indicators

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `BBANDS` | Bollinger Bands | Yes | - |
| `ATR` | Average True Range | Yes | - |
| `NATR` | Normalized ATR | Yes | - |
| `TRANGE` | True Range | Yes | - |
| `SAR` | Parabolic SAR | Yes | - |

### Volume Indicators

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `OBV` | On Balance Volume | Yes | - |
| `AD` | Accumulation/Distribution Line | Yes | - |
| `ADOSC` | Accumulation/Distribution Oscillator | Yes | - |

### Directional Movement

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `MINUS_DI` | Minus Directional Indicator | Yes | - |
| `PLUS_DI` | Plus Directional Indicator | Yes | - |
| `MINUS_DM` | Minus Directional Movement | Yes | - |
| `PLUS_DM` | Plus Directional Movement | Yes | - |

### Hilbert Transform

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `HT_TRENDLINE` | Hilbert Transform Instantaneous Trendline | Yes | - |
| `HT_SINE` | Hilbert Transform Sine Wave | Yes | - |
| `HT_TRENDMODE` | Hilbert Transform Trend vs Cycle Mode | Yes | - |
| `HT_DCPERIOD` | Hilbert Transform Dominant Cycle Period | Yes | - |
| `HT_DCPHASE` | Hilbert Transform Dominant Cycle Phase | Yes | - |
| `HT_PHASOR` | Hilbert Transform Phasor Components | Yes | - |

### Price Indicators

| Function | Description | Free? | Premium Features |
|----------|-------------|-------|------------------|
| `MIDPOINT` | Midpoint Price | Yes | - |
| `MIDPRICE` | Midpoint Price over Period | Yes | - |

---

## Error Responses

All endpoints return error information in consistent format:

### Invalid API Key
```json
{
  "Error Message": "Invalid API call. Please retry or visit the documentation for API_KEY"
}
```

### Rate Limit Exceeded (HTTP 429)
```json
{
  "Note": "Thank you for using Alpha Vantage! Our standard API call frequency is 5 calls per minute. Please visit https://www.alphavantage.co/premium/ if you would like to target a higher API call frequency."
}
```

### Invalid Parameters
```json
{
  "Error Message": "Invalid parameter 'interval'. Please retry."
}
```

---

## Notes

1. **Function-based API**: All endpoints use same base URL with different `function` parameter
2. **Demo key**: Use `apikey=demo` for testing (works with IBM stock only)
3. **Rate limits**: Strictly enforced - 5 req/min for free, 75-1200 req/min for premium
4. **Daily limit**: Free tier has 25 requests per day maximum
5. **Output formats**: JSON (default) or CSV
6. **Outputsize**: `compact` = 100 data points, `full` = complete history (premium benefits)
7. **Premium unlock**: Intraday data, adjusted series, VWAP, MACD require premium
8. **Historical depth**: 20+ years for most time series, 15+ years for options
9. **Real-time data**: US market real-time requires premium (regulatory requirement)
10. **Multiple assets**: Same technical indicators work for stocks, forex, and crypto
