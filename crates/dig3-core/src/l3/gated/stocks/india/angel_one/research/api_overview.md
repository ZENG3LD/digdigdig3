# Angel One SmartAPI - API Overview

## Provider Information
- **Full name**: Angel One SmartAPI (formerly Angel Broking)
- **Website**: https://www.angelone.in
- **Documentation**: https://smartapi.angelbroking.com/docs
- **Category**: stocks/india
- **Type**: Broker with full trading capabilities

## API Type
- **REST**: Yes (base URL: https://apiconnect.angelone.in)
- **WebSocket**: Yes (WebSocket V2 for real-time market data and order updates)
- **GraphQL**: No
- **gRPC**: No
- **Other protocols**: None

## Base URLs
- **Production REST API**: https://apiconnect.angelone.in
- **Login Portal**: https://smartapi.angelone.in/publisher-login
- **WebSocket**: wss://smartapisocket.angelone.in/smart-stream (WebSocket V2)
- **Testnet/Sandbox**: Not available (production only)
- **Regional endpoints**: None (India-focused)
- **API version**: Multiple endpoints use versioned paths (e.g., /rest/secure/angelbroking/*/v1/)

**Note**: Older URLs using `api.angelbroking.com` domain have been deprecated in favor of `apiconnect.angelone.in`.

## Documentation Quality
- **Official docs**: https://smartapi.angelbroking.com/docs
- **Quality rating**: Good
  - Well-structured documentation with endpoint references
  - Active community forum at https://smartapi.angelone.in/smartapi/forum
  - Regular updates and announcements
  - Some areas lack detailed examples
- **Code examples**: Yes (multiple languages)
  - Python, Java, NodeJS, Go, R, C#/.NET, PHP
- **OpenAPI/Swagger spec**: Not publicly available
- **SDKs available**: Yes (official SDKs)
  - Python: https://github.com/angel-one/smartapi-python
  - Go: https://github.com/angel-one/smartapigo
  - Java: https://github.com/angel-one/smartapi-java
  - JavaScript/NodeJS: https://github.com/angel-one/smartapi-javascript
  - .NET: https://github.com/angel-one/smartapi-dotnet
  - All SDKs are actively maintained on GitHub

## Licensing & Terms
- **Free tier**: Yes
  - No monthly fees
  - No API usage charges
  - Free for all Angel One clients
- **Paid tiers**: No separate API pricing tiers
  - Only standard brokerage charges apply (flat ₹20 per trade typically)
  - Historical data is FREE for all segments (NSE, BSE, NFO, BFO, MCX, CDS)
- **Commercial use**: Allowed for Angel One account holders
  - Requires Angel One trading account
  - Must register for SmartAPI access
- **Data redistribution**: Prohibited (for personal/application use only)
- **Terms of Service**: https://www.angelone.in/terms-and-conditions
- **SEBI Compliance**: All API usage subject to SEBI guidelines and compliance requirements

## Support Channels
- **Forum**: https://smartapi.angelone.in/smartapi/forum (very active)
- **Email**: smartapi@angelone.in
- **Customer Support**: Available through Angel One's main support channels
- **GitHub**: https://github.com/angel-one (official SDKs with issue tracking)
- **Status page**: Not publicly available
- **Community**: Active developer community on forum with official Angel One staff responses

## Key Capabilities
- **Market Data**: Real-time and historical data for Indian markets (NSE, BSE, MCX, NFO, BFO, CDS)
- **Trading**: Full order execution (equities, commodities, derivatives, mutual funds)
- **Portfolio Management**: Holdings, positions, margins, funds
- **WebSocket Streaming**: Real-time market data and order updates via WebSocket V2
- **Advanced Orders**: GTT, OCO, AMO, Bracket Orders, Cover Orders
- **120+ Indices**: Real-time OHLC data across NSE, BSE, and MCX

## Access Requirements
1. Open a trading account with Angel One
2. Register for SmartAPI access via the portal
3. Generate API Key from SmartAPI dashboard
4. Authenticate using:
   - Client Code (Angel One account ID)
   - Client PIN (account PIN)
   - TOTP (Time-based One-Time Password from QR token)

## Session Management
- Sessions remain active until 12 midnight (market close)
- JWT token and refresh token obtained via `generateSession()`
- Feed token separate (for WebSocket market data)
- Manual logout available via `terminateSession()`

## Recent Updates (2024-2026)
- Enhanced rate limits: 20 orders per second (increased from 10)
- WebSocket V2 with Depth 20 feature (20 levels of order book)
- Margin Calculator API launched (June 2025)
- Free historical data for all segments (NSE, NFO, BSE, BFO, MCX, CDS)
- Support for 120 indices across exchanges
- Up to 8,000 candles per historical data request
