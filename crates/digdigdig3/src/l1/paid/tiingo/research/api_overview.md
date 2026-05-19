# Tiingo API Overview

## Provider Information
- Full name: Tiingo Financial Data Platform
- Website: https://www.tiingo.com/
- Documentation: https://api.tiingo.com
- Category: stocks/us (multi-asset: stocks, crypto, forex)

## API Type
- REST: Yes (base URL: https://api.tiingo.com)
- WebSocket: Yes (URLs: wss://api.tiingo.com/iex, wss://api.tiingo.com/fx, wss://api.tiingo.com/crypto)
- GraphQL: No
- gRPC: No
- Other protocols: CSV export support for bulk data

## Base URLs
- Production REST: https://api.tiingo.com
- Production WebSocket IEX: wss://api.tiingo.com/iex
- Production WebSocket Forex: wss://api.tiingo.com/fx
- Production WebSocket Crypto: wss://api.tiingo.com/crypto
- Media/Downloads: https://apimedia.tiingo.com
- Testnet/Sandbox: Not available
- Regional endpoints: None
- API version: Multiple versions (v1 paths implicit in endpoints)

## Documentation Quality
- Official docs: https://api.tiingo.com
- Quality rating: Good
  - Comprehensive coverage of endpoints
  - Examples available in Python SDK
  - Some documentation pages are styled frontends (harder to scrape)
  - Active community support
- Code examples: Yes (Python, R, JavaScript/Google Sheets)
- OpenAPI/Swagger spec: Not publicly available
- SDKs available:
  - Python (official): https://github.com/hydrosquall/tiingo-python
  - R (community): riingo package
  - .NET (community): restless-tiingo
  - Google Sheets Add-on (community)

## Licensing & Terms
- Free tier: Yes
  - 5 requests per minute
  - 500 requests per day
  - 50 symbols per hour
  - Access to all API types (EOD, IEX, Crypto, Forex, Fundamentals)
  - 5 years of fundamentals history
  - WebSocket access included
- Paid tiers: Yes
  - Launch: $0 monthly minimum (flexible usage-based)
  - Grow: $100 monthly minimum
  - Enterprise: Custom pricing
  - Up to 1200 requests per minute
  - No daily caps on premium plans
  - 15+ years fundamentals history
- Commercial use: Allowed with appropriate tier
- Data redistribution: Requires Redistribution tier/license
- Terms of Service: https://www.tiingo.com/about/terms

## Support Channels
- Email: Support available through website
- Discord/Slack: Not publicly advertised
- GitHub: https://github.com/hydrosquall/tiingo-python (community SDK)
- Status page: Not publicly available
- Knowledge Base: https://www.tiingo.com/kb/
- Blog: https://blog.tiingo.com/

## Key Features
- Multi-asset coverage: US stocks, Chinese stocks, ETFs, mutual funds, crypto, forex
- High-quality institutional-grade data
- Microsecond resolution for WebSocket firehose
- Direct tier-1 bank connections for forex
- 40+ crypto exchanges aggregated
- 50+ years of historical stock data
- Fundamentals data (20+ years, 5500+ equities, 80+ indicators)
- News API with curated financial news
- IEX real-time intraday data
- End-of-day (EOD) adjusted prices
- Flexible response formats (JSON, CSV)
- WebSocket support across all data types
