# yahoo API Overview

## Provider Information
- Full name: Yahoo Finance
- Website: https://finance.yahoo.com/
- Documentation: https://finance.yahoo.com/ (No official API documentation - community-maintained)
- Category: aggregators

## API Type
- REST: Yes (base URL: https://query1.finance.yahoo.com or https://query2.finance.yahoo.com)
- WebSocket: Yes (URL: wss://streamer.finance.yahoo.com/)
- GraphQL: No
- gRPC: No
- Other protocols: None

## Base URLs
- Production: https://query1.finance.yahoo.com and https://query2.finance.yahoo.com (load balanced)
- Testnet/Sandbox: N/A (no official sandbox environment)
- Regional endpoints: None (single global endpoint with load balancing)
- API version: Multiple versions (v7, v8, v10, v11) for different endpoint types

## Documentation Quality
- Official docs: None - Yahoo shut down official API in 2017
- Quality rating: Poor (unofficial/community-maintained only)
- Code examples: Yes (through community libraries: Python, JavaScript, R, Go)
- OpenAPI/Swagger spec: Not available
- SDKs available:
  - Python: yfinance, yahooquery, yahoofinancials
  - JavaScript/TypeScript: yahoo-finance2
  - R: yfscreen, yfinancer
  - Go: go-yfinance
  - .NET: YahooFinanceAPI

## Licensing & Terms
- Free tier: Yes (unlimited but rate-limited)
- Paid tiers: Available through third-party proxies (RapidAPI)
- Commercial use: Restricted - Personal use only per Yahoo's terms
- Data redistribution: Prohibited - Terms state "intended for personal use only"
- Terms of Service: https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html

## Support Channels
- Email: N/A (no official support)
- Discord/Slack: Community-maintained library Discords/GitHub Discussions
- GitHub: Multiple community library repositories
  - https://github.com/ranaroussi/yfinance
  - https://github.com/gadicc/yahoo-finance2
  - https://github.com/dpguthrie/yahooquery
- Status page: N/A

## Important Notes
- **NO OFFICIAL API**: Yahoo Finance does not provide an official public API since 2017
- **Unofficial Access**: All current access methods rely on reverse-engineered endpoints
- **Risk of Changes**: Yahoo can change endpoints at any time without notice
- **Rate Limiting**: Aggressive rate limiting with IP-based blocks (429 errors common)
- **Personal Use Only**: Yahoo's terms prohibit commercial use and data redistribution
- **Authentication**: Requires cookie/crumb mechanism for historical data downloads
