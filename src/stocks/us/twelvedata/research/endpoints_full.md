# Twelvedata - Complete Endpoint Reference

## Category: Core Market Data

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/time_series` | Historical OHLCV data | Yes | Recommended | 1 per symbol | Max 5000 bars, intervals: 1min-1month |
| GET | `/quote` | Real-time quote data | Yes | Recommended | 1 per symbol | Latest price, OHLC, volume, 52w high/low |
| GET | `/price` | Latest price only | Yes | Recommended | 1 per symbol | Single price value |
| GET | `/eod` | End of day data | Yes | Recommended | 1 per symbol | Daily closing OHLC |
| GET | `/time_series/cross` | Cross-rate time series | Yes | Recommended | 5 per symbol | Calculate exotic pairs on-the-fly |
| GET | `/exchange_rate` | Current exchange rate | Yes | Recommended | 1 per request | Currency conversion rate |
| GET | `/currency_conversion` | Convert currency amounts | Useful tier | Yes | Varies | Actual amount conversion |

### Time Series Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol (e.g., "AAPL", "EUR/USD", "BTC/USD") |
| interval | string | Yes | - | 1min, 5min, 15min, 30min, 45min, 1h, 2h, 4h, 1day, 1week, 1month |
| outputsize | int | No | 30 | Number of data points (max: 5000) |
| start_date | date | No | - | ISO 8601 format: YYYY-MM-DD |
| end_date | date | No | - | ISO 8601 format: YYYY-MM-DD |
| dp | int | No | 5 | Decimal precision (0-11) |
| timezone | string | No | Exchange | IANA timezone or "Exchange" or "UTC" |
| format | string | No | JSON | JSON or CSV |

### Quote Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol |
| interval | string | No | 1day | Time interval for calculations |
| volume_time_period | int | No | 9 | Period for volume calculations |
| dp | int | No | 5 | Decimal precision (0-11) |
| format | string | No | JSON | JSON or CSV |

## Category: Market Movers

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/market_movers/stocks` | Top gaining/losing stocks | Pro+ | Yes | 100 | Direction: gainers/losers |
| GET | `/market_movers/etf` | Top gaining/losing ETFs | Pro+ | Yes | 100 | Direction: gainers/losers |
| GET | `/market_movers/mutual_funds` | Top mutual funds movers | Pro+ | Yes | 100 | Direction: gainers/losers |
| GET | `/market_movers/forex` | Top forex movers | Pro+ | Yes | 100 | Direction: gainers/losers |
| GET | `/market_movers/crypto` | Top crypto movers | Pro+ | Yes | 100 | Direction: gainers/losers |

### Market Movers Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| direction | string | No | gainers | "gainers" or "losers" |
| outputsize | int | No | 10 | Number of results (1-50) |
| country | string | No | - | Filter by country code |

## Category: Reference Data (Catalogs)

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/stocks` | List of all stocks | Yes | No | 1 | Updated daily every 3h from 12 AM |
| GET | `/forex_pairs` | List of forex pairs | Yes | No | 1 | Physical currency pairs |
| GET | `/cryptocurrencies` | List of crypto pairs | Yes | No | 1 | Digital currency pairs |
| GET | `/etf` | List of ETFs | Yes | No | 1 | Exchange-traded funds |
| GET | `/funds` | List of mutual funds | Yes | No | 1 | Investment funds |
| GET | `/commodities` | List of commodities | Yes | No | 1 | Metals, energy, agriculture |
| GET | `/bonds` | List of bonds | Yes | No | 1 | Fixed income instruments |

### Catalog Common Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | No | - | Filter by symbol |
| exchange | string | No | - | Filter by exchange |
| mic_code | string | No | - | Market Identifier Code |
| country | string | No | - | ISO country code |
| type | string | No | - | Instrument type filter |
| format | string | No | JSON | JSON or CSV |

## Category: Discovery & Search

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/symbol_search` | Search symbols | Yes | Recommended | 1 | Supports ticker, ISIN, FIGI, Composite FIGI |
| GET | `/cross_listings` | Find cross-listings | Grow+ | Yes | 40 | All exchanges where security trades |
| GET | `/earliest_timestamp` | Get data start date | Yes | Recommended | 1 | Historical data availability |

### Symbol Search Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Search query (ticker, ISIN, FIGI, etc.) |
| outputsize | int | No | 30 | Max results (max: 120) |

## Category: Markets Information

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/exchanges` | List of exchanges | Yes | No | 1 | Name, MIC code, country, timezone |
| GET | `/exchange_schedule` | Trading hours/schedule | Ultra+ | Yes | 100 | Pre/post-market, session times |
| GET | `/cryptocurrency_exchanges` | Crypto exchange list | Yes | No | 1 | Available crypto venues |
| GET | `/market_state` | Market open/closed status | Yes | Recommended | 1 | Real-time status, time-to-open/close |
| GET | `/countries` | List of countries | Yes | No | 1 | ISO codes, capitals, currencies |
| GET | `/instrument_type` | Available asset classes | Yes | No | 1 | Types of instruments supported |

## Category: Technical Indicators (100+ Available)

### Overlap Studies
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/bbands` | Bollinger Bands | High demand | Yes | Varies | Upper, middle, lower bands |
| GET | `/ema` | Exponential Moving Average | High demand | Yes | Varies | Customizable period |
| GET | `/sma` | Simple Moving Average | High demand | Yes | Varies | Customizable period |
| GET | `/wma` | Weighted Moving Average | Yes | Yes | Varies | Customizable period |
| GET | `/dema` | Double EMA | Yes | Yes | Varies | Faster response than EMA |
| GET | `/tema` | Triple EMA | Yes | Yes | Varies | Even faster response |
| GET | `/mama` | MESA Adaptive MA | Yes | Yes | Varies | Adaptive period |
| GET | `/kama` | Kaufman Adaptive MA | Yes | Yes | Varies | Volatility-adaptive |
| GET | `/vwma` | Volume Weighted MA | Yes | Yes | Varies | Volume consideration |

### Momentum Indicators
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/rsi` | Relative Strength Index | High demand | Yes | Varies | Overbought/oversold |
| GET | `/macd` | MACD | High demand | Yes | Varies | MACD, signal, histogram |
| GET | `/stoch` | Stochastic Oscillator | High demand | Yes | Varies | %K, %D lines |
| GET | `/stochrsi` | Stochastic RSI | Yes | Yes | Varies | RSI-based stochastic |
| GET | `/cci` | Commodity Channel Index | Yes | Yes | Varies | Cycle identification |
| GET | `/adx` | Average Directional Index | High demand | Yes | Varies | Trend strength |
| GET | `/williams` | Williams %R | Yes | Yes | Varies | Overbought/oversold |
| GET | `/roc` | Rate of Change | Yes | Yes | Varies | Momentum measurement |
| GET | `/mom` | Momentum | Yes | Yes | Varies | Price momentum |
| GET | `/ppo` | Price Oscillator | Yes | Yes | Varies | Percentage-based MACD |
| GET | `/trix` | TRIX | Yes | Yes | Varies | Triple smoothed EMA |
| GET | `/ultosc` | Ultimate Oscillator | Yes | Yes | Varies | Multi-period momentum |

### Volume Indicators
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/obv` | On Balance Volume | Yes | Yes | Varies | Cumulative volume flow |
| GET | `/ad` | Accumulation/Distribution | Yes | Yes | Varies | Money flow indicator |
| GET | `/adosc` | AD Oscillator | Yes | Yes | Varies | AD momentum |
| GET | `/mfi` | Money Flow Index | Yes | Yes | Varies | Volume-weighted RSI |

### Volatility Indicators
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/atr` | Average True Range | Yes | Yes | Varies | Volatility measurement |
| GET | `/natr` | Normalized ATR | Yes | Yes | Varies | ATR as percentage |
| GET | `/tr` | True Range | Yes | Yes | Varies | Single-period range |
| GET | `/stddev` | Standard Deviation | Yes | Yes | Varies | Price volatility |

### Other Indicators
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/avgprice` | Average Price | Yes | Yes | Varies | (O+H+L+C)/4 |
| GET | `/medprice` | Median Price | Yes | Yes | Varies | (H+L)/2 |
| GET | `/typprice` | Typical Price | Yes | Yes | Varies | (H+L+C)/3 |
| GET | `/wclprice` | Weighted Close Price | Yes | Yes | Varies | (H+L+C+C)/4 |
| GET | `/sar` | Parabolic SAR | Yes | Yes | Varies | Stop and reverse |
| GET | `/percent_b` | Percent B | High demand | Yes | Varies | Position within Bollinger Bands |
| GET | `/supertrend` | SuperTrend | Yes | Yes | Varies | Trend following |

### Common Technical Indicator Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol |
| interval | string | Yes | - | Time interval |
| time_period | int | No | Varies | Lookback period (e.g., 14 for RSI) |
| series_type | string | No | close | open, high, low, close |
| outputsize | int | No | 30 | Number of data points |
| format | string | No | JSON | JSON or CSV |

## Category: Fundamental Data

### Company Information
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/logo` | Company logo | Yes | Recommended | 1 | Official logo URL |
| GET | `/profile` | Company profile | Grow+ | Yes | 10 | Details, sector, industry, CEO |
| GET | `/statistics` | Key financial metrics | High demand | Yes | Varies | P/E, EPS, market cap, etc. |
| GET | `/key_executives` | Management team | Yes | Yes | Varies | Executive names and roles |
| GET | `/market_cap` | Market capitalization | Yes | Yes | Varies | Current market cap |

### Financial Statements
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/income_statement` | Income statements | Grow+ | Yes | High demand | Revenue, expenses, net income |
| GET | `/balance_sheet` | Balance sheets | Grow+ | Yes | High demand | Assets, liabilities, equity |
| GET | `/cash_flow` | Cash flow statements | Grow+ | Yes | High demand | Operating, investing, financing |

### Earnings & Dividends
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/earnings` | Historical earnings | Yes | Yes | 20 | EPS actual, estimate |
| GET | `/earnings_calendar` | Upcoming earnings | Yes | Recommended | Varies | Scheduled earnings dates |
| GET | `/dividends` | Historical dividends | Yes | Yes | 20 | Payment dates, amounts |
| GET | `/dividends_calendar` | Upcoming dividends | Yes | Recommended | Varies | Scheduled dividend dates |

### Corporate Actions
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/splits` | Stock split history | Yes | Yes | Varies | Historical splits |
| GET | `/splits_calendar` | Upcoming splits | Yes | Recommended | Varies | Scheduled splits |
| GET | `/ipo_calendar` | IPO dates | Yes | Recommended | Varies | Upcoming IPOs |

### Analyst Coverage
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/earning_estimate` | Analyst earnings estimates | Useful | Yes | Varies | EPS forecasts |
| GET | `/revenue_estimate` | Revenue forecasts | Useful | Yes | Varies | Revenue predictions |
| GET | `/eps_trend` | EPS trends | Useful | Yes | Varies | Earnings trends |
| GET | `/eps_revisions` | Estimate revisions | Useful | Yes | Varies | Analyst revisions |
| GET | `/growth_estimates` | Growth projections | Useful | Yes | Varies | Future growth estimates |
| GET | `/recommendations` | Buy/sell ratings | High demand | Yes | Varies | Analyst recommendations |
| GET | `/price_target` | Target prices | High demand | Yes | Varies | Price targets |
| GET | `/analyst_ratings_snapshot` | Consensus ratings | Yes | Yes | Varies | Summary of ratings |

### Ownership & Holdings
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/insider-transactions` | Insider trading | Yes | Yes | Varies | Insider buys/sells |
| GET | `/institutional-holders` | Major shareholders | Yes | Yes | Varies | Institutional ownership |
| GET | `/fund-holders` | Fund ownership | Yes | Yes | Varies | Mutual fund holdings |
| GET | `/direct-holders` | Direct ownership | Yes | Yes | Varies | Direct shareholders |

### Regulatory & Tax
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/edgar-filings-archive` | SEC filings | Grow+ | Yes | Varies | 10-K, 10-Q, 8-K, etc. |
| GET | `/tax_info` | Tax information | Yes | Yes | Varies | Tax details |
| GET | `/sanctioned_entities` | Restricted entities | Yes | Yes | Varies | Sanctions list |

### Additional Fundamentals
| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/last_changes` | Recent updates | Yes | Yes | Varies | Latest data changes |

## Category: ETF Data

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/etf-all-data` | Comprehensive ETF info | High demand | Yes | Varies | All ETF data |
| GET | `/etf-performance` | Performance metrics | High demand | Yes | Varies | Returns, volatility |
| GET | `/etf-composition` | Holdings breakdown | High demand | Yes | Varies | Top holdings |
| GET | `/etf-family-list` | Fund families | Yes | Yes | Varies | ETF families |
| GET | `/etf-type-list` | ETF types | Yes | Yes | Varies | Classification types |

## Category: Mutual Fund Data

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/mf-ratings` | Fund ratings | Yes | Yes | Varies | Morningstar ratings |
| GET | `/mf-purchase-info` | Purchase details | Yes | Yes | Varies | Min investment, fees |
| GET | `/mf-sustainability` | ESG metrics | Yes | Yes | Varies | Sustainability scores |

## Category: Advanced Features

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| GET | `/batch-requests` | Batch multiple queries | Yes | Recommended | 1 per 100 symbols | Combine multiple requests |

### Batch Request Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbols | string | Yes | - | Comma-separated symbols (max 120) |
| intervals | string | No | - | Comma-separated intervals |
| methods | string | No | - | Endpoints to call for each symbol |

## Category: WebSocket Control

| Method | Endpoint | Description | Free? | Auth? | Cost (Credits) | Notes |
|--------|----------|-------------|-------|-------|----------------|-------|
| N/A | Direct WS connection | No REST endpoint needed | Pro+ | Yes | WebSocket credits | Connect directly to wss://ws.twelvedata.com |

## Response Formats

All endpoints support:
- **JSON** (default): Structured data
- **CSV**: Configurable delimiter (default: semicolon)

Set via `format` parameter: `?format=csv`

## Error Responses

All endpoints return standardized error format:

```json
{
  "code": 400,
  "message": "Detailed error description",
  "status": "error"
}
```

## Rate Limit Headers

All REST responses include:
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1234567890
```

On 429 errors:
```
Retry-After: 30
```

## Notes

1. **"High demand"** endpoints consume more credits
2. **Tier requirements** (Pro+, Grow+, Ultra+, Useful) vary by endpoint
3. All **catalog endpoints** updated daily every 3 hours from 12 AM
4. **Max 5000 data points** per request for time series
5. **Max 120 symbols** in batch requests
6. **Decimal precision** (dp parameter) ranges 0-11
7. **Null values** expected when data unavailable - defensive programming required
8. **ISIN, FIGI, Composite FIGI** supported in symbol search
9. **Timezone** parameter supports IANA identifiers, "Exchange", or "UTC"
10. **Adjustment** parameter available for price data (splits, dividends, or none)
