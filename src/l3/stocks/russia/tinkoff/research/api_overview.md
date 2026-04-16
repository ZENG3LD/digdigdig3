# Tinkoff Invest API Overview

## Provider Information
- **Full name**: Tinkoff Investments (T-Bank Investments)
- **Website**: https://www.tinkoff.ru/invest/
- **Documentation**: https://tinkoff.github.io/investAPI/
- **Category**: stocks/russia
- **Type**: Russian broker with full trading support

## API Type
- **REST**: Yes (legacy OpenAPI v1, mostly deprecated)
  - Base URL: https://api-invest.tinkoff.ru/openapi
- **WebSocket**: Yes
  - URL: wss://invest-public-api.tinkoff.ru/ws/
- **GraphQL**: No
- **gRPC**: Yes (PRIMARY protocol)
  - Production: invest-public-api.tinkoff.ru:443
  - Sandbox: sandbox-invest-public-api.tinkoff.ru:443
- **Other protocols**: gRPC-web for browser clients

## Base URLs

### gRPC Endpoints (Primary)
- **Production**: `invest-public-api.tinkoff.ru:443`
- **Sandbox**: `sandbox-invest-public-api.tinkoff.ru:443`
- **API version**: v2 (Protocol Buffers v3)

### REST Endpoints (Legacy/Proxy)
- **Production REST**: `https://invest-public-api.tbank.ru/rest/`
- **Format**: `https://invest-public-api.tbank.ru/rest/tinkoff.public.invest.api.contract.v1.{ServiceName}/{MethodName}`
- **Example**: `https://invest-public-api.tbank.ru/rest/tinkoff.public.invest.api.contract.v1.MarketDataService/GetCandles`

### WebSocket Endpoint
- **Production**: `wss://invest-public-api.tinkoff.ru/ws/`

## Documentation Quality
- **Official docs**: https://tinkoff.github.io/investAPI/
- **Quality rating**: Excellent
  - Comprehensive proto contracts
  - Detailed method documentation
  - Extensive examples
  - Multi-language SDK support
- **Code examples**: Yes
  - Languages: Python, Java, C#, Go, JavaScript, Rust, PHP, Ruby, Haskell, Swift, Dart
- **OpenAPI/Swagger spec**: Yes (https://tinkoff.github.io/investAPI/swagger-ui/)
- **Proto contracts**: Available at https://github.com/Tinkoff/investAPI/tree/main/src/docs/contracts
- **SDKs available**:
  - **Official**: Python, Java, C#, Go
  - **Unofficial**: Node.js, Rust, PHP, Ruby, Haskell, Swift, Dart, C++

## Licensing & Terms
- **Free tier**: Yes (completely free)
- **Paid tiers**: No API pricing (free for all Tinkoff Investments clients)
- **Commercial use**: Allowed (requires registration for public software)
  - Contact: al.a.volkov@tinkoff.ru for dedicated appname and technical support
- **Data redistribution**: Not explicitly allowed (check Terms of Service)
- **Terms of Service**: Requires Tinkoff Investments account

## Support Channels
- **Email**: al.a.volkov@tinkoff.ru (for public software developers)
- **GitHub**: https://github.com/Tinkoff/investAPI (issues, discussions)
- **Documentation**: https://tinkoff.github.io/investAPI/
- **Status page**: Not publicly available

## Key Features
- Full trading support (stocks, bonds, ETFs, futures, options, currencies)
- Real-time market data streaming
- Historical data (candles from 5 seconds to monthly, depth up to 10 years)
- Portfolio and positions tracking
- Stop orders (take-profit, stop-loss, stop-limit)
- Sandbox environment for testing
- Bidirectional streaming (gRPC)
- Multiple token types (readonly, full-access, account-specific, sandbox)

## Technical Characteristics
- **Protocol**: gRPC (Protocol Buffers v3)
- **Package**: `tinkoff.public.invest.api.contract.v1`
- **Authentication**: Bearer token in metadata
- **Rate limiting**: Dynamic (based on trading activity)
- **Peak capacity**: 20,000 requests/second (platform-wide)
- **Token lifespan**: 3 months from last use
- **Historical data**: From 1970-01-01 (Unix epoch)

## Geographic Restrictions
- **Primary market**: Russia (Moscow Exchange - MOEX, RTS)
- **International access**: Yes (10,000+ securities from 30 countries)
- **VPN detection**: Not documented
- **Geo-fencing**: Not documented

## Coverage
- **Russian stocks**: ~1,900 shares (as of 2022)
- **Russian bonds**: ~655 bonds
- **ETFs**: ~105 ETFs
- **Futures**: ~284 futures contracts
- **Currencies**: ~21 currency pairs
- **Options**: Available (with underlying asset tracking)
- **International**: 10,000+ securities from 30 countries

## Special Notes
- Tinkoff rebranded to T-Bank in 2024, but API remains under Tinkoff Invest branding
- Requires Tinkoff Investments brokerage account
- Trading limits: Orders over 6,000,000 RUB require additional confirmation (not available via API)
- Sandbox token must be used exclusively with sandbox services
- Tokens display only once during generation (cannot be viewed later)
- 2FA enabled by default for granular tokens (90-day lifespan)
