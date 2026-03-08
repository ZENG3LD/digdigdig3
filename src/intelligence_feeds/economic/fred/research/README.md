# FRED API Research - Complete Documentation

Research completed on: 2026-01-26

## Overview

This directory contains comprehensive research documentation for the Federal Reserve Economic Data (FRED) API, a free economic data service provided by the Federal Reserve Bank of St. Louis.

## Research Files

All 8 required research files have been completed:

1. **api_overview.md** - Provider information, API architecture, documentation quality, licensing
2. **endpoints_full.md** - Complete reference of all 30 API endpoints with parameters
3. **websocket_full.md** - WebSocket availability (Not available - REST only)
4. **authentication.md** - API key authentication details and examples
5. **tiers_and_limits.md** - Pricing (Free), rate limits (120 req/min), terms of use
6. **data_types.md** - Comprehensive catalog of 840,000+ economic time series
7. **response_formats.md** - JSON/XML/CSV examples for all major endpoints
8. **coverage.md** - Geographic and data coverage details

## Key Findings

### Provider Details
- **Name**: Federal Reserve Economic Data (FRED)
- **Category**: data_feeds
- **Type**: Economic/Macro data provider (NOT a trading/exchange connector)
- **Data Sources**: 118 official sources (government agencies, central banks)
- **Total Series**: 840,000+ economic time series

### API Architecture
- **Protocol**: REST over HTTPS
- **Base URL**: https://api.stlouisfed.org
- **Endpoints**: 30 endpoints across 5 categories (Categories, Releases, Series, Sources, Tags)
- **WebSocket**: Not supported
- **Formats**: XML (default), JSON, CSV, XLSX

### Authentication
- **Method**: Simple API key in query parameter
- **Format**: `?api_key=YOUR_32_CHAR_KEY`
- **Cost**: Completely FREE (no paid tiers)
- **Registration**: Required at https://fredaccount.stlouisfed.org/

### Rate Limits
- **Free Tier**: 120 requests per minute per API key
- **Burst**: Not documented (assume none)
- **Headers**: No rate limit headers provided
- **Multiple Keys**: Allowed - each key has independent 120/min limit

### Data Types
FRED specializes in U.S. economic data:
- **National Accounts**: GDP, GNP, Personal Income, PCE
- **Prices**: CPI (320+ series), PPI (10,000+ series), PCE Price Index
- **Employment**: Unemployment, NFP, JOLTS, Labor Force data
- **Interest Rates**: Fed Funds, Treasury yields, mortgage rates (940+ series)
- **Money Supply**: M1, M2, Bank Credit, Reserves
- **Trade**: Balance of trade, imports, exports
- **Manufacturing**: Industrial production, PMI, capacity utilization
- **Housing**: Starts, permits, sales, prices (390+ house price series)
- **Regional**: 460,000+ state/MSA-level series
- **International**: Limited coverage (major economies only)

### Data NOT Available
- Individual stock prices (indices only)
- Company fundamentals
- Cryptocurrency data
- Real-time tick/intraday data
- Options/derivatives data
- Alternative data (sentiment, satellite, etc.)

### Key Features
1. **ALFRED**: Archival data with complete revision history
2. **Historical Depth**: Some series from 1700s, most from 1900s
3. **Built-in Transformations**: % change, log, frequency aggregation
4. **Regional Data**: Comprehensive state/MSA coverage
5. **Official Sources**: Government agencies only

### Terms of Use Restrictions
**IMPORTANT**: Updated June 2024 with new prohibitions:
- ❌ **AI/ML Training**: Cannot train models/LLMs with FRED data
- ❌ **Caching/Archiving**: Cannot store or redistribute FRED data
- ❌ **Wholesale Downloads**: Cannot bulk download entire database
- ✅ **Non-commercial Use**: Free for educational, personal, research use
- ⚠️ **Commercial Use**: Requires special permission from Federal Reserve

### Coverage Summary
- **U.S. Data**: Excellent (primary focus)
- **International**: Limited (major economies, basic indicators)
- **Real-time**: No (updates on release schedules)
- **Historical**: Excellent (decades to centuries)
- **Frequencies**: Daily, Weekly, Monthly, Quarterly, Annual
- **Geographic**: National, State, MSA, limited County

## Implementation Notes

### Suitable For:
- ✅ Macro economic analysis
- ✅ Research and backtesting with economic data
- ✅ Economic dashboards and visualization
- ✅ Long-term economic forecasting
- ✅ Academic research
- ✅ Policy analysis

### NOT Suitable For:
- ❌ Real-time trading signals
- ❌ High-frequency trading
- ❌ Individual stock analysis
- ❌ Cryptocurrency trading
- ❌ Intraday market data
- ❌ AI model training (prohibited)

### Connector Design Recommendations:
1. **No Trading Traits**: This is data-only, implement custom DataFeed trait
2. **Caching Strategy**: Aggressive caching (data changes infrequently)
3. **Rate Limiting**: Client-side rate limiter (120/min per key)
4. **Error Handling**: Robust handling for 400/404/429 errors
5. **Multiple Keys**: Support key rotation for higher throughput
6. **Transformations**: Leverage built-in units/frequency parameters
7. **Polling**: Use /fred/series/updates to discover recent changes

### Popular Series IDs:
```rust
// GDP
"GDP"      // Gross Domestic Product
"GDPC1"    // Real GDP

// Employment
"UNRATE"   // Unemployment Rate
"PAYEMS"   // Nonfarm Payrolls

// Inflation
"CPIAUCSL" // Consumer Price Index
"PCEPI"    // PCE Price Index

// Interest Rates
"DFF"      // Federal Funds Effective Rate
"DGS10"    // 10-Year Treasury Yield
"MORTGAGE30US" // 30-Year Mortgage Rate

// Markets
"SP500"    // S&P 500 Index
"VIXCLS"   // VIX Volatility Index

// Money Supply
"M2SL"     // M2 Money Supply

// Housing
"HOUST"    // Housing Starts
"CSUSHPISA" // Case-Shiller House Price Index

// Commodities
"DCOILWTICO" // WTI Crude Oil Price
"GOLDAMGBD228NLBM" // Gold Price
```

## API Examples

### Basic Request (GDP data):
```bash
curl "https://api.stlouisfed.org/fred/series/observations?\
series_id=GDP&\
api_key=YOUR_API_KEY&\
file_type=json&\
observation_start=2020-01-01"
```

### Search for Series:
```bash
curl "https://api.stlouisfed.org/fred/series/search?\
search_text=unemployment&\
api_key=YOUR_API_KEY&\
file_type=json&\
limit=10"
```

### Get Recently Updated Series:
```bash
curl "https://api.stlouisfed.org/fred/series/updates?\
api_key=YOUR_API_KEY&\
file_type=json&\
limit=100"
```

## Sources

Research based on official documentation and community resources:

- [FRED API Documentation](https://fred.stlouisfed.org/docs/api/fred/)
- [FRED API Terms of Use](https://fred.stlouisfed.org/docs/api/terms_of_use.html)
- [FRED API Key Management](https://fredaccount.stlouisfed.org/apikeys)
- [St. Louis Fed Web Services](https://fred.stlouisfed.org/docs/api/fred/overview.html)
- [fredr R package documentation](https://sboysel.github.io/fredr/)
- [FRED OpenAPI Specification](https://github.com/armanobosyan/FRED-OpenAPI-specification)
- [fredapi Python library](https://pypi.org/project/fredapi/)

## Next Steps

1. ✅ Research Complete (all 8 files documented)
2. ⏭️ Design connector architecture (data_feeds trait, not trading trait)
3. ⏭️ Implement FRED connector following V5 architecture
4. ⏭️ Add rate limiting and caching
5. ⏭️ Test with popular series
6. ⏭️ Document usage examples

## Notes

- FRED is a **data provider only** - no trading capabilities
- Free service with generous rate limits
- Extremely reliable (official government data)
- Perfect for economic analysis and research
- Not suitable for real-time trading applications
- Terms of Use prohibit AI training and data redistribution
