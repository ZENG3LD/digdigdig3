# Twelvedata API Overview

## Provider Information
- Full name: Twelve Data
- Website: https://twelvedata.com/
- Documentation: https://twelvedata.com/docs
- Category: stocks/us (multi-asset provider)

## API Type
- REST: Yes (base URL: https://api.twelvedata.com)
- WebSocket: Yes (URL: wss://ws.twelvedata.com)
- GraphQL: No
- gRPC: No
- Other protocols: None

## Base URLs
- Production: https://api.twelvedata.com
- Testnet/Sandbox: No dedicated sandbox (demo API key available: `apikey=demo`)
- Regional endpoints: None (single global endpoint)
- API version: v1 (current, stable)

## Documentation Quality
- Official docs: https://twelvedata.com/docs
- Quality rating: Excellent
  - Comprehensive API reference with all endpoints documented
  - Interactive API request builder available
  - WebSocket playground for testing
  - Detailed support articles at https://support.twelvedata.com/
- Code examples: Yes (languages: Python, JavaScript, R, cURL)
- OpenAPI/Swagger spec: Available
- SDKs available:
  - **Official**: Python, R
  - **Community**: C#, JavaScript, PHP, Go, TypeScript
  - **Integrations**: Excel Add-in, Google Sheets Add-on
  - **AI Integration**: MCP Server for LLM/AI assistant workflows

## Licensing & Terms
- Free tier: Yes (Basic plan with 8 API calls/minute, 800/day)
- Paid tiers: Yes (Grow, Pro, Ultra, Enterprise)
- Commercial use: Allowed (depending on plan)
- Data redistribution: Prohibited without explicit license
- Terms of Service: https://twelvedata.com/terms
- Attribution: Not explicitly required for personal use

## Support Channels
- Email: support@twelvedata.com
- Discord/Slack: Not publicly available
- GitHub: https://github.com/twelvedata (official SDKs)
- Status page: Not publicly advertised
- Support portal: https://support.twelvedata.com/ (knowledge base with extensive articles)
- Community: API documentation includes community-contributed SDKs

## Provider Characteristics

### Multi-Asset Coverage
Twelvedata is a comprehensive financial data provider covering:
- **Stocks**: US + 90+ international exchanges
- **Forex**: 200+ currency pairs (majors, minors, exotics)
- **Cryptocurrencies**: 180+ crypto exchanges, thousands of pairs
- **ETFs**: Global coverage
- **Indices**: Major global indices
- **Commodities**: Metals, energy, agriculture
- **Bonds**: Fixed income instruments
- **Mutual Funds**: Investment funds

### Data Types
- Real-time market data (Pro+ plans)
- Delayed data (free tier: varies by asset)
- Historical data (back to 1980s-90s for some assets)
- Fundamental data (company financials, earnings, dividends)
- Technical indicators (100+ built-in)
- Reference data (exchanges, symbols, calendars)
- Extended hours (US pre/post-market on Pro+ plans)

### Key Differentiators
1. **Unified API** for multiple asset classes
2. **Low latency WebSocket** streaming (~170ms average)
3. **Extensive fundamentals** (income statements, balance sheets, cash flow from 1980s)
4. **100+ technical indicators** with customizable parameters
5. **Credit-based pricing** (different endpoints cost different credits)
6. **CSV and JSON** output formats
7. **Batch requests** support (up to 120 symbols per call)
8. **Pre/Post-market data** for US equities

## API Philosophy
- Simple API key authentication (no complex HMAC signing)
- RESTful design with intuitive endpoints
- Consistent JSON response structure
- Defensive programming encouraged (null values possible)
- Credit system for fair usage across tiers
- Rate limiting with clear headers and 429 responses
