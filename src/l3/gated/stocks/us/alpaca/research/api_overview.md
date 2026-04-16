# Alpaca API Overview

## Provider Information
- Full name: Alpaca Markets
- Website: https://alpaca.markets/
- Documentation: https://docs.alpaca.markets/docs
- Category: stocks/us
- Provider Type: US Stock Broker + Market Data Provider (supports TRADING and DATA)

## API Type
- REST: Yes (base URLs below)
- WebSocket: Yes (multiple streams)
- GraphQL: No
- gRPC: No
- Other protocols: No

## Base URLs

### Trading API
- **Production (Live Trading)**: https://api.alpaca.markets
- **Paper Trading**: https://paper-api.alpaca.markets
- **API version**: v2 (primary), v1beta1 (some endpoints)

### Market Data API
- **Production**: https://data.alpaca.markets
- **Sandbox**: https://data.sandbox.alpaca.markets
- **API version**: v2 (stocks), v1beta1 (options, news), v1beta3 (crypto)

### WebSocket URLs
- **Market Data - Production**: wss://stream.data.alpaca.markets/{version}/{feed}
- **Market Data - Sandbox**: wss://stream.data.sandbox.alpaca.markets/{version}/{feed}
- **Market Data - Test Stream**: wss://stream.data.alpaca.markets/v2/test (available 24/7, use symbol "FAKEPACA")
- **Trading Updates - Live**: wss://api.alpaca.markets/stream
- **Trading Updates - Paper**: wss://paper-api.alpaca.markets/stream

### Authentication/OAuth
- **Production**: https://authx.alpaca.markets/v1
- **Sandbox**: https://authx.sandbox.alpaca.markets/v1

### Broker API (for businesses building trading apps)
- **Dashboard**: https://broker-app.alpaca.markets/
- Separate endpoints for account management, funding, compliance

## Documentation Quality
- Official docs: https://docs.alpaca.markets/docs
- Quality rating: Excellent
  - Comprehensive REST API reference with all parameters
  - WebSocket documentation with message formats
  - Clear authentication guides
  - Code examples in multiple languages
  - Interactive API explorer
- Code examples: Yes (Python, JavaScript, Go, C++, C#, Rust community libraries)
- OpenAPI/Swagger spec: Available (documented in reference section)
- SDKs available:
  - Official: Python (alpaca-py), JavaScript/TypeScript (Node.js)
  - Community: Rust, Go, C++, C#, R, Ruby
  - GitHub: https://github.com/alpacahq

## Licensing & Terms
- Free tier: Yes (IEX data feed, 200 API calls/min, paper trading unlimited)
- Paid tiers: Yes (Algo Trader Plus $99/mo for real-time SIP data feed)
- Commercial use: Allowed (commission-free trading)
- Data redistribution: Prohibited (data for personal use only)
- Terms of Service: https://alpaca.markets/terms
- Trading:
  - Paper trading: Free for all users globally
  - Live trading: Available for US residents (requires brokerage account)
  - Commission-free: Yes (no commission on stock/ETF/crypto trades)

## Support Channels
- Email: support@alpaca.markets
- Discord/Slack: Slack community available
- GitHub: https://github.com/alpacahq
- Forum: https://forum.alpaca.markets/
- Status page: Not explicitly documented
- Documentation: Comprehensive at https://docs.alpaca.markets/

## Key Features
- **Trading**: Commission-free US stocks, ETFs, options, and crypto
- **Market Data**: Real-time and historical data for stocks, options, crypto
- **Paper Trading**: Free simulated trading environment with real-time data
- **Fractional Shares**: Trade as little as $1 worth of shares for 2,000+ US equities
- **Margin Trading**: Up to 4X intraday and 2X overnight buying power
- **Options Trading**: Multiple levels (up to Level 3)
- **Crypto Trading**: 24/7 spot trading
- **Developer-First**: API-first platform designed for algorithmic trading
- **WebSocket Streaming**: Real-time market data and trading updates
- **OAuth2**: Connect API for third-party integrations

## Account Types
- Paper-Only: Free for anyone globally (email signup only)
- Live Brokerage: Individual and business accounts (US residents)
- Crypto Accounts: Specialized crypto trading accounts
- Custodial & IRA: Available for specific use cases
- Alpaca Elite: $100k+ deposit for lower margin rates (5% vs 6.5%) and free market data
