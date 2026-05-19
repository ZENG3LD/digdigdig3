# Fyers API Overview

## Provider Information
- Full name: Fyers Securities Private Limited
- Website: https://fyers.in/
- Documentation: https://myapi.fyers.in/docsv3
- Category: stocks/india
- Type: India Broker (NSE, BSE, MCX)
- Specialization: F&O (Futures & Options), Equity, Commodities, Currency Derivatives

## API Type
- REST: Yes (base URL: https://api.fyers.in)
- WebSocket: Yes (multiple endpoints)
  - Data WebSocket: wss://api-t1.fyers.in/socket/v3/dataSock
  - Order WebSocket: wss://api-t1.fyers.in/socket/v3/orderSock
  - TBT WebSocket: wss://rtsocket-api.fyers.in/versova
- GraphQL: No
- gRPC: No
- Other protocols: None

## Base URLs
- Production (REST): https://api.fyers.in
- Production (Data REST): https://api-t1.fyers.in
- Testnet/Sandbox: Not available
- Regional endpoints: None
- API versions:
  - API v2: /api/v2 (legacy)
  - Data v2: /data-rest/v2 (legacy)
  - Data v3: /data (current, released January 2026)
  - API v3: Current version (v3.0.0)

## Documentation Quality
- Official docs: https://myapi.fyers.in/docsv3
- Quality rating: Good
  - Comprehensive REST endpoint documentation
  - WebSocket documentation available
  - Multiple official SDKs
  - Community support and examples
  - Some information requires browsing support portal
- Code examples: Yes
  - Languages: Python, JavaScript/Node.js, .NET/C#, Go
  - GitHub sample code: https://github.com/FyersDev/fyers-api-sample-code
  - Community examples available
- OpenAPI/Swagger spec: Not available
- SDKs available:
  - Python: fyers-apiv3 (PyPI, released January 20, 2026)
  - JavaScript/Node.js: fyers-api-v3 (npm), extra-fyers (npm)
  - .NET/C#: Available
  - Go: Multiple community implementations
  - All SDKs support both REST and WebSocket

## Licensing & Terms
- Free tier: Yes
- Paid tiers: Yes (for API Bridge)
- Commercial use: Allowed (requires Fyers trading account)
- Trading account requirements:
  - Must have active Fyers trading account
  - Must enable External 2FA TOTP for API access
  - Demat and trading account required
- API pricing:
  - Basic API: FREE (no subscription fee)
  - Fyers API Bridge: Rs 500/month or Rs 3,500/year
- Data redistribution: Not allowed (personal use only)
- Terms of Service: https://fyers.in/terms-and-conditions/

## Support Channels
- Email: support@fyers.in
- Community forum: https://fyers.in/community/
  - https://fyers.in/community/fyers-api-rha0riqv/ (API section)
  - https://fyers.in/community/questions-5gz5j8db/ (Q&A)
  - https://fyers.in/community/api-algo-trading-bihtdkgq/ (Algo Trading)
- Discord/Slack: Not available
- GitHub: https://github.com/FyersDev/
- Support Portal: https://support.fyers.in/portal/en/kb/fyers-api-integrations
- Status page: https://status.fyers.in/
- API Dashboard: https://myapi.fyers.in/dashboard/

## Market Coverage
- Exchanges: NSE (National Stock Exchange), BSE (Bombay Stock Exchange), MCX (Multi Commodity Exchange), NCDEX
- Approved Segments:
  - CM (Capital Market) - Equity
  - FO (Futures & Options) - Derivatives
  - CD (Currency Derivatives)
  - COMM (Commodities)
- Asset Classes:
  - Equities (NSE, BSE)
  - Futures & Options (Index and Stock F&O)
  - Commodities (MCX, NCDEX)
  - Currency Derivatives (NSE)
  - Mutual Funds

## Key Features
- Real-time market data (tick-by-tick)
- Order placement and management (single, basket, multi-leg)
- WebSocket streaming for live data
- Historical data (OHLC candles)
- Portfolio management (holdings, positions, funds)
- E-DIS (Electronic Delivery of Securities)
- Market status and symbol master data
- Order execution speed: Under 50 milliseconds
- Multi-leg strategies support (spreads, straddles, etc.)
- Bracket Orders (BO) and Cover Orders (CO)

## API Improvements in V3 (2026)
- Improved tick update speed
- Lite mode for targeted LTP (Last Traded Price) updates
- Real-time symbol-specific updates
- Real-time market depth changes
- Increased subscription capacity (up to 5,000 symbols via WebSocket)
- Daily rate limit increased 10x (from 10,000 to 100,000 requests/day)
- Strengthened error handling callbacks
- Enhanced WebSocket stability with auto-reconnection
