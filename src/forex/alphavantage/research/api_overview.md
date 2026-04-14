# AlphaVantage API Overview

## Provider Information
- **Full name**: Alpha Vantage
- **Website**: https://www.alphavantage.co/
- **Documentation**: https://www.alphavantage.co/documentation/
- **Category**: Multi-asset data provider (forex, stocks, crypto, commodities, economic indicators)
- **Type**: Data provider only (NO trading support)

## API Type
- **REST**: Yes (base URL: `https://www.alphavantage.co/query`)
- **WebSocket**: No
- **GraphQL**: No
- **gRPC**: No
- **Other protocols**: Model Context Protocol (MCP) server for AI integration (2026)

## Base URLs
- **Production**: `https://www.alphavantage.co/query`
- **Testnet/Sandbox**: Not applicable (uses demo API key for testing with IBM stock)
- **Regional endpoints**: None (single global endpoint)
- **API version**: No explicit versioning (function-based API)

## API Architecture
AlphaVantage uses a **function-based API** where all requests go to the same base URL with different `function` parameters:
```
https://www.alphavantage.co/query?function=FX_DAILY&from_symbol=EUR&to_symbol=USD&apikey=YOUR_KEY
```

## Documentation Quality
- **Official docs**: https://www.alphavantage.co/documentation/
- **Quality rating**: Excellent - comprehensive with clear examples
- **Code examples**: Yes (Python, JavaScript, R, Ruby examples in community)
- **OpenAPI/Swagger spec**: Not available
- **SDKs available**:
  - Python: `alpha_vantage` (PyPI)
  - Ruby: `alphavantage_ruby`
  - R: `alphavantager`
  - Elixir: `alpha_vantage`
  - Node.js: Community packages
  - Community wrappers for many languages

## Key Features
- **Multi-asset coverage**: Forex, stocks, crypto, commodities, economic indicators
- **Technical indicators**: 50+ pre-computed indicators (SMA, EMA, RSI, MACD, Bollinger Bands, etc.)
- **Fundamental data**: Company overviews, financial statements, earnings, dividends
- **Economic data**: GDP, CPI, unemployment, interest rates, treasury yields
- **News & sentiment**: AI-powered sentiment analysis on market news
- **Historical depth**: 20+ years of historical data (15+ years for options)
- **Coverage**: 200,000+ stock tickers across 20+ global exchanges
- **182 physical currencies** supported for forex
- **MCP integration**: Native support for Claude, ChatGPT, and other AI assistants (2026)

## Data Regulatory Status
- **NASDAQ-licensed data provider**
- Compliant with SEC, FINRA, and stock exchange regulations
- Real-time US market data requires premium tier (regulatory requirement)
- Free tier provides 15-minute delayed US data or end-of-day data

## Licensing & Terms
- **Free tier**: Yes (25 requests per day, 5 per minute)
- **Paid tiers**: Yes ($49.99/month - $249.99/month for standard plans)
- **Enterprise tier**: Custom pricing for unlimited requests
- **Commercial use**: Allowed with appropriate tier
- **Data redistribution**: Check Terms of Service (likely restricted for raw data)
- **Attribution**: Recommended but check specific terms
- **Terms of Service**: https://www.alphavantage.co/terms_of_service/

## Premium Features (Locked Behind Paid Tiers)
- Real-time US stock market data
- 15-minute delayed US market data
- Intraday data (TIME_SERIES_INTRADAY, FX_INTRADAY, CRYPTO_INTRADAY)
- Adjusted time series (TIME_SERIES_DAILY_ADJUSTED)
- Real-time US options data
- Historical options data (15+ years)
- VWAP and advanced technical indicators
- Full outputsize for all endpoints (free tier limited to compact = 100 data points)
- Higher rate limits (75-1200 requests per minute vs 5 per minute free)

## Support Channels
- **Email**: support@alphavantage.co (mentioned in docs)
- **Discord/Slack**: Not publicly advertised
- **GitHub**: Various community wrappers and examples
- **Status page**: Not explicitly mentioned
- **Documentation**: Comprehensive online documentation with examples
- **Premium support**: Available for paid tier customers

## Unique Selling Points
1. **Comprehensive multi-asset coverage** - Single API for stocks, forex, crypto, commodities, economics
2. **50+ technical indicators** - Pre-computed and ready to use
3. **Fundamental data** - Financial statements, earnings, analyst ratings
4. **Economic indicators** - GDP, CPI, employment, treasury yields
5. **News sentiment analysis** - AI-powered sentiment scoring
6. **20+ years historical data** - Deep historical coverage
7. **NASDAQ-licensed** - Regulatory compliant data
8. **MCP support** - Native AI assistant integration (2026)
9. **Simple API design** - Function-based, easy to learn
10. **Multiple output formats** - JSON and CSV

## Limitations
- **No WebSocket** - REST only, no real-time streaming
- **No trading** - Data provider only, no order execution
- **Rate limits** - Free tier very restrictive (25 requests/day)
- **Premium content** - Many valuable features require paid subscription
- **No sandbox** - Testing limited to demo API key with IBM stock only

## Target Use Cases
- Quantitative research and backtesting
- Portfolio tracking applications
- Financial dashboards and analytics
- Algorithmic trading (data input, not execution)
- Academic research
- AI/ML model training with financial data
- Economic data analysis
- Technical analysis tools

## Notes
- **Demo API key**: Use `apikey=demo` for testing (works with IBM stock only)
- **Response formats**: JSON (default) or CSV
- **Historical rate limit changes**: Used to be 500/day, then 100/day, now 25/day for free tier
- **Premium unlock**: Significant value unlock at premium tiers (intraday data, higher limits)
