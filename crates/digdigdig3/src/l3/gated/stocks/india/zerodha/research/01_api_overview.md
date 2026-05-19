# Zerodha Kite Connect API Overview

## Provider Information
- **Full name**: Zerodha Kite Connect
- **Website**: https://zerodha.com
- **Documentation**: https://kite.trade/docs/connect/v3/
- **Category**: stocks/india
- **Provider Type**: Full-service broker with trading and market data capabilities

## API Type
- **REST**: Yes (base URL: https://api.kite.trade)
- **WebSocket**: Yes (URL: wss://ws.kite.trade)
- **GraphQL**: No
- **gRPC**: No
- **Other protocols**: None

## Base URLs
- **Production REST**: https://api.kite.trade
- **Production WebSocket**: wss://ws.kite.trade
- **Login URL**: https://kite.zerodha.com/connect/login?v=3&api_key={api_key}
- **Testnet/Sandbox**: Not available (must use production with small orders for testing)
- **Regional endpoints**: None (single India-based endpoint)
- **API version**: v3 (current stable version)

## Documentation Quality
- **Official docs**: https://kite.trade/docs/connect/v3/
- **Quality rating**: Excellent
- **Code examples**: Yes (Python, Java, JavaScript/TypeScript, Go, .NET, PHP)
- **OpenAPI/Swagger spec**: Not publicly available
- **SDKs available**:
  - Python (pykiteconnect) - https://github.com/zerodha/pykiteconnect
  - JavaScript/TypeScript (kiteconnectjs) - https://github.com/zerodha/kiteconnectjs
  - Java (javakiteconnect) - https://github.com/zerodha/javakiteconnect
  - Go (gokiteconnect) - https://github.com/zerodha/gokiteconnect
  - .NET (dotnetkiteconnect) - https://github.com/zerodha/dotnetkiteconnect
  - PHP (phpkiteconnect) - https://github.com/zerodha/phpkiteconnect

## Licensing & Terms
- **Free tier**: Yes (Personal API - free for individual use)
  - Allows placing orders and tracking positions, holdings, and funds
  - No WebSocket streaming
  - No historical candle data
- **Paid tiers**: Yes
  - Connect API: ₹500/month (reduced from ₹2000 in 2024)
  - Includes realtime WebSocket streaming
  - Includes historical candle data
- **Commercial use**: Requires paid Connect API subscription
- **Data redistribution**: Prohibited without explicit permission
- **Terms of Service**: https://kite.trade/terms/

## Support Channels
- **Email**: Not publicly listed (use support portal)
- **Discord/Slack**: Not available
- **GitHub**: https://github.com/zerodha (official SDKs and issue tracking)
- **Community Forum**: https://kite.trade/forum/discussions
- **Support Portal**: https://support.zerodha.com/category/trading-and-markets/general-kite/kite-api
- **Status page**: Not available

## API Architecture
- **Format**: REST-like HTTP APIs with form-encoded parameters
- **Response Type**: JSON (rarely Gzipped for large responses)
- **HTTP Status Codes**: Standard HTTP codes with accompanying JSON error data
- **Cross-Site Requests**: Not enabled - cannot be called directly from browsers
- **Authentication**: Custom OAuth-like flow (not standard OAuth 2.0)
- **Security**: api_secret must never be embedded in client applications

## Supported Exchanges
Zerodha Kite Connect provides access to all major Indian exchanges:
- **NSE** - National Stock Exchange (Equities)
- **BSE** - Bombay Stock Exchange (Equities)
- **NFO** - NSE Futures & Options
- **BFO** - BSE Futures & Options
- **MCX** - Multi Commodity Exchange
- **CDS** - Currency Derivatives Segment (NSE)
- **BCD** - BSE Currency Derivatives

## Registration Details
Zerodha is registered with SEBI as a stock broker for:
- Cash/derivatives/currency derivatives segments of NSE (INB/INF/INE231390627)
- Cash/derivatives segments of BSE (INB/INF011390623)
- Commodity derivatives segment of MCX (TM/CORP/1945)

## Key Features
- Execute orders in real time (equities, commodities, mutual funds)
- Manage user portfolios (holdings, positions)
- Stream live market data over WebSockets
- Historical candle data access
- Good Till Triggered (GTT) orders
- Basket orders (up to 20 orders per basket, 50 baskets total)
- Margin calculation
- Mutual fund orders and SIPs (via BSE STAR MF platform)
- Order postbacks (WebSocket notifications)
- Comprehensive portfolio analytics

## Target Market
India's #1 retail stock broker by volume, focused on retail and algorithmic traders in the Indian stock market.
