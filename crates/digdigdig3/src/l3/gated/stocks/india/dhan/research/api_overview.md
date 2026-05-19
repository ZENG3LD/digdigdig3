# Dhan API Overview

## Provider Information
- Full name: DhanHQ (Dhan)
- Website: https://dhanhq.co/
- Documentation: https://dhanhq.co/docs/v2/
- Category: stocks/india
- Type: Indian Stock Broker with Trading Support

## API Type
- REST: Yes (base URL: https://api.dhan.co/v2/)
- WebSocket: Yes (multiple URLs for different data types)
  - Live Market Feed: wss://api-feed.dhan.co
  - 20-Level Market Depth: wss://depth-api-feed.dhan.co/twentydepth
  - 200-Level Market Depth: wss://full-depth-api.dhan.co/twohundreddepth
- GraphQL: No
- gRPC: No
- Other protocols: None

## Base URLs
- Production: https://api.dhan.co/v2/
- Sandbox (Paper Trading): Available at https://api.dhan.co/ (sandbox environment)
- Testnet/Sandbox: Yes - fully simulated environment with 10,00,000 INR daily capital reset
- Regional endpoints: None (India-focused)
- API version: v2.0 (latest), v1.0 (legacy)

## Documentation Quality
- Official docs: https://dhanhq.co/docs/v2/
- Quality rating: Excellent
  - Well-structured with clear endpoint documentation
  - Comprehensive parameter descriptions
  - Rate limits clearly specified
  - Error codes documented in annexure
- Code examples: Yes (Python, JavaScript/Node.js, curl)
- OpenAPI/Swagger spec: Not publicly available
- SDKs available:
  - Python (official): https://github.com/dhan-oss/DhanHQ-py
  - JavaScript/Node.js (official): https://github.com/dhan-oss/DhanHQ-js
  - Go (community): https://github.com/tradewithcanvas/godhanhq
  - TypeScript (community): https://github.com/anshuopinion/dhan-ts

## Licensing & Terms
- Free tier: Yes
  - Trading APIs: Completely FREE for all Dhan users
  - Data APIs: FREE if 25+ trades completed in previous 30 days
- Paid tiers: Yes (Data APIs only)
  - Data API subscription: Rs. 499 + taxes per month (if <25 trades/month)
- Commercial use: Allowed for Dhan account holders
- Data redistribution: Prohibited (for personal trading use only)
- Terms of Service: https://dhanhq.co/
- Special Requirements (2026):
  - Static IP mandatory for all Order APIs from January 2026
  - Static IP NOT required for Sandbox environment
  - Access tokens valid for 24 hours only (SEBI compliance)
  - API key valid for 1 year

## Support Channels
- Email: Available through support portal
- Knowledge Base: https://knowledge.dhan.co/
- Support Portal: https://dhan.co/support/platforms/dhanhq-api/
- Community Forum: https://madefortrade.in/ (community discussions)
- GitHub: https://github.com/dhan-oss/
- Status page: Not publicly available
- Discord/Slack: Not mentioned

## Key Features
- **Full Trading Support**: Order placement, modification, cancellation across all segments
- **Advanced Order Types**: Super Orders (bracket + trailing SL), Forever Orders (GTT)
- **Multi-Exchange**: NSE (Equity & F&O), BSE (Equity), MCX (Commodities)
- **Deep Market Data**: 200-level market depth (NSE only)
- **Historical Data**: Up to 5 years intraday data (1m, 5m, 15m, 25m, 60m)
- **Real-time Feeds**: WebSocket with binary data format (Little Endian)
- **Options Analytics**: Full option chains with Greeks, OI, volume
- **Portfolio Management**: Holdings, positions, funds, P&L tracking
- **EDIS Support**: Electronic Delivery Instruction Slip for stock selling
- **Postback/Webhooks**: Real-time order update notifications
- **Liberal Rate Limits**: Industry-leading limits (25 orders/sec, 20,000 requests/day)

## Markets Supported
- **Exchanges**: NSE, BSE, MCX
- **Segments**:
  - NSE_EQ (NSE Equity - Cash Market)
  - NSE_FNO (NSE Futures & Options)
  - BSE_EQ (BSE Equity)
  - MCX_COMM (MCX Commodities)
- **Instrument Types**:
  - Equities (stocks, ETFs)
  - Equity Derivatives (futures, options)
  - Commodities
  - Options (NSE, BSE, MCX)

## Unique Selling Points
1. **Free Trading APIs**: No monthly charges for API access
2. **200-Level Market Depth**: Deepest orderbook data for retail traders in India
3. **Super Orders**: Advanced risk management with trailing stop loss
4. **Forever Orders**: GTT-like orders valid for 365 days
5. **Sandbox Environment**: Risk-free testing with daily capital reset
6. **Liberal Rate Limits**: 20,000 requests/day on Non-Trading APIs
7. **Comprehensive Coverage**: All major Indian exchanges (NSE, BSE, MCX)
8. **5-Year Historical Data**: Extensive intraday data availability

## Target Audience
- Retail algorithmic traders in India
- Trading platform developers
- Banks and FinTech companies building trading services
- Quantitative traders and analysts
- Individual traders with Dhan accounts

## Release History (Recent Updates - 2026)
- Static IP mandatory from January 2026 for Order APIs
- API key-based authentication introduced (1-year validity)
- Daily access token generation required
- Enhanced security in line with SEBI guidelines on algo trading
- Sandbox environment launched for risk-free testing
- Developer portal introduced for API management
