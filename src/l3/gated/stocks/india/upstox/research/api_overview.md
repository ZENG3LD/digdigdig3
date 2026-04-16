# Upstox API Overview

## Provider Information
- Full name: Upstox (RKSV Securities India Private Ltd)
- Website: https://upstox.com
- Documentation: https://upstox.com/developer/api-documentation/
- Category: stocks/india
- Type: Indian broker with trading and market data APIs

## API Type
- REST: Yes (base URL: https://api.upstox.com/v2, https://api-hft.upstox.com/v2)
- WebSocket: Yes (URL: wss://api.upstox.com/v2/feed/market-data-feed/protobuf and wss://api.upstox.com/v2/feed/portfolio-stream-feed)
- GraphQL: No
- gRPC: No
- Other protocols: WebSocket with Protocol Buffers (binary format)

## Base URLs
- Production REST (Standard): https://api.upstox.com/v2
- Production REST (HFT - High Frequency Trading): https://api-hft.upstox.com/v2
- Production REST (V3 endpoints): https://api.upstox.com/v3
- WebSocket Market Data: wss://api.upstox.com/v2/feed/market-data-feed/protobuf
- WebSocket Portfolio: wss://api.upstox.com/v2/feed/portfolio-stream-feed
- Sandbox: Yes (available, specific URL in sandbox environment)
- Regional endpoints: None (India-focused)
- API versions: v2 (current stable), v3 (newer endpoints with expanded features)

## Documentation Quality
- Official docs: https://upstox.com/developer/api-documentation/
- Quality rating: Good
- Code examples: Yes (Python, JavaScript/Node.js, Java, PHP, C#)
- OpenAPI/Swagger spec: Available (referenced in documentation)
- SDKs available:
  - Python: https://github.com/upstox/upstox-python
  - Node.js: https://github.com/upstox/upstox-nodejs
  - Java: https://github.com/upstox/upstox-java
  - PHP: https://github.com/upstox/upstox-php
  - .NET: https://github.com/upstox/upstox-dotnet

## Licensing & Terms
- Free tier: Yes (API creation is free, usage requires subscription)
- Paid tiers: Yes (Rs 499/month GST included subscription)
- Promotional offer: Flat Rs 10/order pricing valid till 31 March 2026
- Commercial use: Allowed (requires API subscription)
- Data redistribution: Not allowed (for personal/app use only)
- Terms of Service: https://upstox.com/terms-and-conditions/

## Support Channels
- Email: support@upstox.com
- Community Forum: https://community.upstox.com
- GitHub: https://github.com/upstox (official SDKs)
- Status page: Not publicly available
- Developer App Management: https://account.upstox.com/developer/apps
- Help Center: https://help.upstox.com

## Key Features
- Ultra-fast, low latency trading APIs
- Real-time market data via WebSocket
- Support for NSE, BSE, and MCX exchanges
- Comprehensive order types (Market, Limit, SL, SL-M, GTT, AMO)
- Historical data from year 2000 (daily) and 2022 (intraday)
- Portfolio management and P&L tracking
- Option chain data with Greeks
- OAuth 2.0 authentication
- Multi-order APIs for batch operations
- Webhook support for order updates

## Special Notes
- API services free till 31 March 2026
- Supports both Interactive API (trading) and Historical API (data)
- Designed for algorithmic trading and fintech applications
- Broker account required for trading operations
- Data provided directly from NSE, BSE, and MCX exchanges
