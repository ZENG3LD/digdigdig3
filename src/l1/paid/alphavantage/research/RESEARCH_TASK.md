# AlphaVantage API Research Task

## Mission
**EXHAUSTIVE RESEARCH** - Document EVERYTHING AlphaVantage offers.

This is a multi-asset data provider (forex, stocks, crypto, commodities, economic indicators) with NO trading support.

## Provider Details
- Provider: alphavantage
- Category: forex (primary), but also stocks, crypto, commodities
- Documentation: https://www.alphavantage.co/documentation/
- Output folder: `src/forex/alphavantage/research/`

## Required Research Files (8 files)

### 1. api_overview.md
- Provider information
- API type (REST/WebSocket/GraphQL/gRPC)
- Base URLs
- Documentation quality
- Licensing & terms
- Support channels

### 2. endpoints_full.md
**CRITICAL:** Document EVERY endpoint, grouped by category.
- Standard Market Data
- Historical Data
- Forex Specific (FX_DAILY, FX_INTRADAY, etc.)
- Stocks (TIME_SERIES_DAILY, etc.)
- Crypto (DIGITAL_CURRENCY_DAILY, etc.)
- Technical Indicators (SMA, EMA, RSI, MACD, etc.)
- Fundamental Data (OVERVIEW, EARNINGS, etc.)
- Economic Indicators (REAL_GDP, INFLATION, etc.)
- Commodities (WTI, BRENT, etc.)
- Metadata endpoints

For each endpoint document:
- Method, endpoint/function name
- Description
- Free tier access (Yes/No)
- Authentication required (Yes/No)
- Rate limits
- Parameters (all of them)
- Notes/limitations

### 3. websocket_full.md
If WebSocket NOT available: Create file with "WebSocket: Not Available" and skip.

Otherwise document:
- Connection URLs
- All available channels/topics
- Subscription format
- Message formats
- Heartbeat/ping-pong
- Connection limits
- Authentication (if applicable)

### 4. authentication.md
- Public endpoints (if any)
- API key requirements
- How to obtain API key
- API key format (query param: apikey=XXX)
- Rate limits with/without key
- OAuth (if applicable)
- Signature/HMAC (if applicable)
- Authentication examples
- Error codes

### 5. tiers_and_limits.md
**CRITICAL:** This is very important.

Document:
- Free tier (limits, data access, restrictions)
- Paid tiers (Premium, what unlocks?)
- Rate limits (requests per minute/day)
- How rate limits are measured
- Response headers for rate limiting
- Error responses (HTTP 429)
- Quota/credits system (if applicable)
- Monitoring usage

### 6. data_types.md
**CRITICAL:** Catalog EVERYTHING AlphaVantage offers.

Check boxes for available data:
- Standard market data (price, ticker, OHLC, volume)
- Historical data (minute, daily, weekly, monthly)
- Forex specific (currency pairs, intraday FX, daily FX, weekly FX, monthly FX)
- Stocks (intraday, daily, weekly, monthly, adjusted)
- Crypto (daily, weekly, monthly)
- Technical indicators (ALL of them: SMA, EMA, RSI, MACD, STOCH, ADX, etc.)
- Fundamental data (company overview, earnings, income statement, balance sheet, cash flow)
- Economic indicators (GDP, inflation, unemployment, etc.)
- Commodities (WTI, Brent, Natural Gas, etc.)
- Metadata (symbol search, market status, etc.)

Document what makes AlphaVantage special/unique.

### 7. response_formats.md
**EXACT JSON examples from official docs** - don't invent.

For EVERY important endpoint, document exact response format:
- GET FX_INTRADAY
- GET FX_DAILY
- GET FX_WEEKLY
- GET FX_MONTHLY
- GET CURRENCY_EXCHANGE_RATE
- GET TIME_SERIES_INTRADAY (stocks)
- GET TIME_SERIES_DAILY (stocks)
- GET CRYPTO_INTRADAY
- GET SMA (technical indicator example)
- GET OVERVIEW (fundamental data example)
- GET REAL_GDP (economic indicator example)
- etc.

### 8. coverage.md
Document:
- Geographic coverage
- Markets/exchanges covered
- Instrument coverage (stocks, crypto, forex, commodities)
- Forex pairs (majors, minors, exotics)
- Stock markets (US, international)
- Crypto coins supported
- Data history depth
- Granularity available (1min, 5min, 15min, 30min, 60min, daily, weekly, monthly)
- Real-time vs delayed
- Update frequency
- Data quality

## Important Notes

1. AlphaVantage uses function-based API (e.g., `?function=FX_INTRADAY&from_symbol=EUR&to_symbol=USD`)
2. Free tier has strict rate limits (5 API calls per minute, 100 per day typically)
3. Premium tier unlocks higher rate limits and intraday data
4. All endpoints use same base URL with different function parameter
5. API key passed as query parameter: `&apikey=YOUR_API_KEY`
6. NO WebSocket support (REST only)
7. Responses are JSON or CSV format
8. Very comprehensive - covers forex, stocks, crypto, technical indicators, fundamentals, economic data

## Research Tips

1. Start with https://www.alphavantage.co/documentation/
2. Document ALL function types (FX_INTRADAY, FX_DAILY, TIME_SERIES_*, CRYPTO_*, technical indicators, fundamentals, economic)
3. Note differences between free and premium tiers
4. Pay attention to rate limits - this is critical
5. Check for CSV vs JSON output options
6. Document all interval options (1min, 5min, 15min, 30min, 60min)
7. Note outputsize parameter (compact vs full)
8. Document datatype parameter (json vs csv)

## Exit Criteria

- [ ] All 8 research files created in `src/forex/alphavantage/research/`
- [ ] Every file has EXACT data from official docs (no guessing)
- [ ] All function types documented (FX, TIME_SERIES, CRYPTO, indicators, fundamentals, economic)
- [ ] All data types cataloged
- [ ] Tier/pricing clearly documented
- [ ] WebSocket documented as unavailable
- [ ] Coverage/limits understood
- [ ] Response formats from real examples

## Output

Create all 8 files in:
`c:\Users\VA PC\CODING\ML_TRADING\nemo\zengeld-terminal\crates\connectors\crates\v5\src\forex\alphavantage\research\`

Files:
1. api_overview.md
2. endpoints_full.md
3. websocket_full.md
4. authentication.md
5. tiers_and_limits.md
6. data_types.md
7. response_formats.md
8. coverage.md
